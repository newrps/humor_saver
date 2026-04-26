//! news-tracker collector
//!
//! 1. 시작 시 sources.json 을 DB(`sources` 테이블)에 sync
//! 2. 부팅 직후 1회 + cron 주기로 수집 실행
//! 3. RSS 가 있는 매체는 RSS, 없는 매체는 (TODO: 빅카인즈 API)
//! 4. UPSERT 로 중복 방지 — articles.url 또는 external_id 기준

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

mod collectors;
mod db;
mod sources;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx::query=warn".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let cron_expr = env::var("COLLECT_CRON").unwrap_or_else(|_| "0 0 */1 * * *".into());
    //                                                       ↑ croner 6-필드 (초 분 시 일 월 요일)
    let sources_path = env::var("SOURCES_PATH").unwrap_or_else(|_| "/app/sources.json".into());

    info!("PostgreSQL 연결: {}", redact(&database_url));
    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?,
    );

    info!("매체 목록 sync: {}", sources_path);
    sources::sync_sources(&pool, &sources_path).await?;

    info!("초기 수집 1회 실행");
    if let Err(e) = collectors::run_all(&pool).await {
        error!("초기 수집 실패: {:#}", e);
    }

    info!("스케줄러 등록 (cron = {})", cron_expr);
    let mut scheduler = JobScheduler::new().await?;
    let pool_for_cron = pool.clone();
    let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
        let pool = pool_for_cron.clone();
        Box::pin(async move {
            info!("⏰ cron tick - 수집 시작");
            if let Err(e) = collectors::run_all(&pool).await {
                error!("수집 실패: {:#}", e);
            }
        })
    })?;
    scheduler.add(job).await?;
    scheduler.start().await?;

    info!("✅ collector 가동중. 종료: Ctrl+C");
    tokio::signal::ctrl_c().await?;
    info!("종료 신호 받음");
    scheduler.shutdown().await?;
    Ok(())
}

fn redact(url: &str) -> String {
    // postgresql://user:pass@host/db → postgresql://user:***@host/db
    if let Ok(mut u) = url::Url::parse(url) {
        if u.password().is_some() {
            let _ = u.set_password(Some("***"));
        }
        u.to_string()
    } else {
        url.to_string()
    }
}
