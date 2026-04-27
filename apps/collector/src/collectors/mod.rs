//! 수집기 진입점 — 활성 매체 순회 + 각 매체에 맞는 수집기 호출

use anyhow::Result;
use sqlx::PgPool;
use std::time::Instant;
use tracing::{info, warn};

use crate::db;

pub mod clien;
pub mod rss;

/// 모든 활성 매체에 대해 수집 1라운드 실행
pub async fn run_all(pool: &PgPool) -> Result<()> {
    let sources = db::list_enabled_sources(pool).await?;
    info!("수집 대상 매체 {}개", sources.len());

    let mut total_new = 0_i64;
    for src in &sources {
        let run_id = db::start_run(pool, Some(src.id)).await?;
        let started = Instant::now();
        let result = if src.url.contains("clien.net") {
            clien::collect(pool, src).await
        } else if src.rss_url.is_some() {
            rss::collect(pool, src).await
        } else {
            warn!("[{}] RSS URL / 매체별 크롤러 없음 → 스킵", src.name);
            Ok(0)
        };
        let duration_ms = started.elapsed().as_millis() as i32;

        match result {
            Ok(n) => {
                total_new += n;
                db::finish_run(pool, run_id, n, duration_ms, None).await.ok();
                db::touch_source_after_collect(pool, src.id, true).await.ok();
            }
            Err(e) => {
                warn!("[{}] 수집 실패: {:#}", src.name, e);
                db::finish_run(pool, run_id, 0, duration_ms, Some(format!("{:#}", e))).await.ok();
                db::touch_source_after_collect(pool, src.id, false).await.ok();
            }
        }
    }
    info!("✅ 1라운드 완료 - 신규 {}건", total_new);
    Ok(())
}
