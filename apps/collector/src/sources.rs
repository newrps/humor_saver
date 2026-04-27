//! sources.json → PostgreSQL `sources` 테이블 sync

use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

#[derive(Debug, Deserialize)]
struct SourcesFile {
    sources: Vec<SourceEntry>,
}

#[derive(Debug, Deserialize)]
struct SourceEntry {
    name: String,
    category: String,
    url: String,
    #[serde(default)]
    rss: Option<String>,
    #[serde(default)]
    region: Option<String>,
    #[serde(default)]
    field: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

pub async fn sync_sources(pool: &PgPool, path: impl AsRef<Path>) -> Result<()> {
    let raw = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("sources 파일 읽기 실패: {:?}", path.as_ref()))?;
    let file: SourcesFile = serde_json::from_str(&raw)?;

    let mut inserted = 0_u64;
    let mut updated = 0_u64;
    for s in &file.sources {
        let res = sqlx::query(
            r#"
            INSERT INTO sources (name, category, url, rss_url, region, field, language, enabled)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, 'ko'), TRUE)
            ON CONFLICT (name) DO UPDATE
                SET category = EXCLUDED.category,
                    url = EXCLUDED.url,
                    rss_url = EXCLUDED.rss_url,
                    region = EXCLUDED.region,
                    field = EXCLUDED.field,
                    language = EXCLUDED.language
            RETURNING (xmax = 0) AS inserted
            "#,
        )
        .bind(&s.name)
        .bind(&s.category)
        .bind(&s.url)
        .bind(&s.rss)
        .bind(&s.region)
        .bind(&s.field)
        .bind(&s.language)
        .fetch_one(pool)
        .await?;
        let was_inserted: bool = res.try_get("inserted").unwrap_or(false);
        if was_inserted {
            inserted += 1;
        } else {
            updated += 1;
        }
    }
    info!(
        "매체 sync 완료: 신규 {}, 업데이트 {} (총 {}개)",
        inserted,
        updated,
        file.sources.len()
    );
    Ok(())
}

// 헬퍼 trait import
use sqlx::Row;
