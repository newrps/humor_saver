use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::*;
use crate::embedding;
use crate::error::{AppError, AppResult};
use crate::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/health", get(health))
        .route("/articles", get(list_articles))
        .route("/articles/:id", get(get_article))
        .route("/search", get(semantic_search))
        .route("/sources", get(list_sources))
        .route("/sources/:id/stats", get(source_stats))
        .route("/trends/keywords", get(trends_keywords))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// ───── 기사 목록 (풀텍스트/매체 필터) ────────────────
#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    source_id: Option<i32>,
    q: Option<String>,
}
fn default_limit() -> i64 { 20 }

async fn list_articles(
    State(state): State<SharedState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    let limit = q.limit.clamp(1, 100);
    let offset = q.offset.max(0);

    let rows: Vec<ArticleRow> = if let Some(query) = q.q.as_deref().filter(|s| !s.trim().is_empty()) {
        // PostgreSQL 풀텍스트 검색
        sqlx::query_as(r#"
            SELECT a.id, a.source_id, s.name AS source_name, s.category AS source_category,
                   a.title, a.summary, a.url, a.image_url, a.author,
                   a.published_at, a.collected_at, a.embedding_status,
                   a.language, a.translated_title, a.translated_summary
              FROM articles a
              JOIN sources s ON s.id = a.source_id
             WHERE a.tsv @@ plainto_tsquery('simple', $1)
               AND ($2::int IS NULL OR a.source_id = $2)
             ORDER BY a.published_at DESC NULLS LAST, a.collected_at DESC
             LIMIT $3 OFFSET $4
        "#)
        .bind(query)
        .bind(q.source_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool).await?
    } else {
        sqlx::query_as(r#"
            SELECT a.id, a.source_id, s.name AS source_name, s.category AS source_category,
                   a.title, a.summary, a.url, a.image_url, a.author,
                   a.published_at, a.collected_at, a.embedding_status,
                   a.language, a.translated_title, a.translated_summary
              FROM articles a
              JOIN sources s ON s.id = a.source_id
             WHERE ($1::int IS NULL OR a.source_id = $1)
             ORDER BY a.published_at DESC NULLS LAST, a.collected_at DESC
             LIMIT $2 OFFSET $3
        "#)
        .bind(q.source_id).bind(limit).bind(offset)
        .fetch_all(&state.pool).await?
    };

    Ok(Json(json!({ "items": rows, "limit": limit, "offset": offset })))
}

// ───── 기사 상세 ────────────────────────────────────
async fn get_article(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ArticleDetailRow>> {
    let row: Option<ArticleDetailRow> = sqlx::query_as(r#"
        SELECT a.id, a.source_id, s.name AS source_name, s.category AS source_category,
               a.title, a.summary, a.content, a.url, a.image_url, a.author,
               a.category, a.published_at, a.collected_at, a.tags
          FROM articles a
          JOIN sources s ON s.id = a.source_id
         WHERE a.id = $1
    "#)
    .bind(id).fetch_optional(&state.pool).await?;
    row.map(Json).ok_or(AppError::NotFound)
}

// ───── 의미 검색 (Qdrant) ───────────────────────────
#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_search_limit")]
    limit: usize,
}
fn default_search_limit() -> usize { 10 }

#[derive(Serialize)]
pub struct SearchHitOut {
    pub article_id: i64,
    pub score: f32,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub source_name: Option<String>,
    pub published_at: Option<String>,
}

async fn semantic_search(
    State(state): State<SharedState>,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<Value>> {
    let query = q.q.trim();
    if query.is_empty() {
        return Err(AppError::BadRequest("q required".into()));
    }
    let limit = q.limit.clamp(1, 50);

    // 1. query → 임베딩
    let vec = embedding::embed(&state.tei_url, query)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("embedding failed: {e}")))?;

    // 2. Qdrant 검색
    let hits = state.qdrant.search(&vec, limit)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("qdrant search failed: {e}")))?;

    // 3. article_id 모아서 PostgreSQL에서 메타 가져오기
    let article_ids: Vec<i64> = hits.iter()
        .filter_map(|h| h.payload.get("article_id").and_then(|v| v.as_i64()))
        .collect();

    if article_ids.is_empty() {
        return Ok(Json(json!({ "items": [], "query": query })));
    }

    let rows: Vec<(i64, String, Option<String>, String, Option<String>, Option<String>, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(r#"
        SELECT a.id, a.title, a.summary, a.url, a.image_url, s.name, a.published_at
          FROM articles a
          JOIN sources s ON s.id = a.source_id
         WHERE a.id = ANY($1)
    "#)
    .bind(&article_ids)
    .fetch_all(&state.pool).await?;

    let row_map: std::collections::HashMap<i64, _> = rows.into_iter().map(|r| (r.0, r)).collect();

    // hits 순서대로 출력 (점수 순)
    let items: Vec<SearchHitOut> = hits.into_iter().filter_map(|h| {
        let aid = h.payload.get("article_id").and_then(|v| v.as_i64())?;
        let row = row_map.get(&aid)?;
        Some(SearchHitOut {
            article_id: aid,
            score: h.score,
            title: row.1.clone(),
            summary: row.2.clone(),
            url: row.3.clone(),
            image_url: row.4.clone(),
            source_name: row.5.clone(),
            published_at: row.6.map(|d| d.to_rfc3339()),
        })
    }).collect();

    Ok(Json(json!({ "items": items, "query": query })))
}

// ───── 매체 ─────────────────────────────────────────
async fn list_sources(State(state): State<SharedState>) -> AppResult<Json<Vec<SourceRow>>> {
    let rows: Vec<SourceRow> = sqlx::query_as(r#"
        SELECT id, name, category, url, region, field, enabled,
               last_success_at, consecutive_errors
          FROM sources
         ORDER BY category, name
    "#).fetch_all(&state.pool).await?;
    Ok(Json(rows))
}

async fn source_stats(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> AppResult<Json<SourceStatsRow>> {
    let row: Option<SourceStatsRow> = sqlx::query_as(r#"
        SELECT id, name, category, total_articles, articles_24h, articles_7d,
               latest_article_at, last_success_at
          FROM v_source_stats
         WHERE id = $1
    "#).bind(id).fetch_optional(&state.pool).await?;
    row.map(Json).ok_or(AppError::NotFound)
}

// ───── 키워드 트렌드 ────────────────────────────────
#[derive(Deserialize)]
pub struct TrendsQuery {
    #[serde(default = "default_days")]
    days: i32,
    #[serde(default = "default_trend_limit")]
    limit: i64,
}
fn default_days() -> i32 { 7 }
fn default_trend_limit() -> i64 { 20 }

async fn trends_keywords(
    State(state): State<SharedState>,
    Query(q): Query<TrendsQuery>,
) -> AppResult<Json<Value>> {
    let days = q.days.clamp(1, 30);
    let limit = q.limit.clamp(1, 100);

    // 일별 집계 테이블이 비어있을 수 있으니 article_keywords 직접 집계
    let rows: Vec<KeywordTrendRow> = sqlx::query_as(r#"
        SELECT k.word, count(DISTINCT a.id)::bigint AS article_count
          FROM article_keywords ak
          JOIN keywords k ON k.id = ak.keyword_id
          JOIN articles a ON a.id = ak.article_id
         WHERE NOT k.is_blocked
           AND a.collected_at > NOW() - ($1::int || ' days')::interval
         GROUP BY k.word
         ORDER BY article_count DESC
         LIMIT $2
    "#)
    .bind(days)
    .bind(limit)
    .fetch_all(&state.pool).await?;

    Ok(Json(json!({ "items": rows, "days": days })))
}
