//! news-tracker API (axum)
//!
//! 엔드포인트:
//!   GET /health
//!   GET /articles?limit=20&offset=0&source_id=&q=    풀텍스트 또는 최근 기사
//!   GET /articles/:id
//!   GET /search?q=&limit=10                          의미 검색 (Qdrant)
//!   GET /sources                                     매체 목록
//!   GET /sources/:id/stats                           매체별 통계
//!   GET /trends/keywords?days=7&limit=20             키워드 트렌드

use std::env;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

mod db;
mod embedding;
mod error;
mod handlers;
mod qdrant;

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub qdrant: qdrant::QdrantClient,
    pub tei_url: String,
}

pub type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=warn,sqlx::query=warn".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")?;
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://qdrant:6333".into());
    let qdrant_key = env::var("QDRANT_API_KEY").ok().filter(|s| !s.trim().is_empty());
    let tei_url = env::var("TEI_URL").unwrap_or_else(|_| "http://tei:80".into());
    let port: u16 = env::var("API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    info!("PostgreSQL 연결...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    info!("Qdrant 연결...");
    let qdrant = qdrant::QdrantClient::new(qdrant_url.clone(), qdrant_key);

    let state: SharedState = Arc::new(AppState {
        pool,
        qdrant,
        tei_url,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(handlers::routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("✅ API 시작: {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
