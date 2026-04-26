use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ArticleRow {
    pub id: i64,
    pub source_id: i32,
    pub source_name: Option<String>,
    pub source_category: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub collected_at: DateTime<Utc>,
    pub embedding_status: String,
    pub language: Option<String>,
    pub translated_title: Option<String>,
    pub translated_summary: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ArticleDetailRow {
    pub id: i64,
    pub source_id: i32,
    pub source_name: Option<String>,
    pub source_category: Option<String>,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: String,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub collected_at: DateTime<Utc>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SourceRow {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub url: String,
    pub region: Option<String>,
    pub field: Option<String>,
    pub enabled: bool,
    pub last_success_at: Option<DateTime<Utc>>,
    pub consecutive_errors: i32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SourceStatsRow {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub total_articles: i64,
    pub articles_24h: i64,
    pub articles_7d: i64,
    pub latest_article_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct KeywordTrendRow {
    pub word: String,
    pub article_count: i64,
}
