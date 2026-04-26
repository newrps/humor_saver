//! DB 헬퍼 — sqlx 쿼리 wrapper

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Source {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub url: String,
    pub rss_url: Option<String>,
    pub enabled: bool,
}

pub async fn list_enabled_sources(pool: &PgPool) -> Result<Vec<Source>> {
    let rows = sqlx::query_as::<_, Source>(
        "SELECT id, name, category, url, rss_url, enabled
           FROM sources
          WHERE enabled = TRUE
          ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug)]
pub struct NewArticle {
    pub source_id: i32,
    pub external_id: Option<String>,
    pub url: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub language: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// UPSERT - 신규 INSERT 시 1, 기존 건은 0 반환
pub async fn upsert_article(pool: &PgPool, a: &NewArticle) -> Result<i64> {
    let row: (bool,) = sqlx::query_as(
        r#"
        INSERT INTO articles
            (source_id, external_id, url, title, summary, content,
             author, category, published_at, image_url, language, tags)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, COALESCE($11, 'ko'), $12)
        ON CONFLICT (source_id, url) DO UPDATE
            SET title = EXCLUDED.title,
                summary = COALESCE(EXCLUDED.summary, articles.summary),
                content = COALESCE(EXCLUDED.content, articles.content),
                published_at = COALESCE(EXCLUDED.published_at, articles.published_at),
                image_url = COALESCE(EXCLUDED.image_url, articles.image_url),
                tags = COALESCE(EXCLUDED.tags, articles.tags)
        RETURNING (xmax = 0) AS inserted
        "#,
    )
    .bind(a.source_id)
    .bind(&a.external_id)
    .bind(&a.url)
    .bind(&a.title)
    .bind(&a.summary)
    .bind(&a.content)
    .bind(&a.author)
    .bind(&a.category)
    .bind(a.published_at)
    .bind(&a.image_url)
    .bind(&a.language)
    .bind(&a.tags)
    .fetch_one(pool)
    .await?;
    Ok(if row.0 { 1 } else { 0 })
}

/// 매체 수집 결과 반영 — last_collected_at, last_success_at, consecutive_errors
pub async fn touch_source_after_collect(pool: &PgPool, source_id: i32, success: bool) -> Result<()> {
    if success {
        sqlx::query(
            r#"UPDATE sources
                  SET last_collected_at = NOW(),
                      last_success_at = NOW(),
                      consecutive_errors = 0
                WHERE id = $1"#,
        )
        .bind(source_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"UPDATE sources
                  SET last_collected_at = NOW(),
                      consecutive_errors = consecutive_errors + 1
                WHERE id = $1"#,
        )
        .bind(source_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn start_run(pool: &PgPool, source_id: Option<i32>) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO collection_runs (source_id, status) VALUES ($1, 'running') RETURNING id"#,
    )
    .bind(source_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn finish_run(
    pool: &PgPool,
    run_id: i64,
    new_articles: i64,
    duration_ms: i32,
    error: Option<String>,
) -> Result<()> {
    sqlx::query(
        r#"UPDATE collection_runs
              SET finished_at = NOW(),
                  status = $2,
                  new_articles = $3,
                  duration_ms = $4,
                  error_message = $5
            WHERE id = $1"#,
    )
    .bind(run_id)
    .bind(if error.is_some() { "failed" } else { "success" })
    .bind(new_articles)
    .bind(duration_ms)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}
