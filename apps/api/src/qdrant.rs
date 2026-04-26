//! Qdrant 검색 (REST API 직접 호출 — 의존성 최소화)

use serde::{Deserialize, Serialize};

const COLLECTION: &str = "articles";

pub struct QdrantClient {
    base: String,
    api_key: Option<String>,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct SearchReq<'a> {
    vector: &'a [f32],
    limit: usize,
    with_payload: bool,
}

#[derive(Deserialize, Debug)]
pub struct SearchHit {
    pub id: serde_json::Value, // UUID 또는 int
    pub score: f32,
    pub payload: serde_json::Value,
}

#[derive(Deserialize)]
struct SearchResp {
    result: Vec<SearchHit>,
}

impl QdrantClient {
    pub fn new(base: String, api_key: Option<String>) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            api_key,
            http: reqwest::Client::new(),
        }
    }

    pub async fn search(&self, vector: &[f32], limit: usize) -> anyhow::Result<Vec<SearchHit>> {
        let url = format!("{}/collections/{}/points/search", self.base, COLLECTION);
        let mut req = self.http.post(&url).json(&SearchReq {
            vector,
            limit,
            with_payload: true,
        });
        if let Some(key) = &self.api_key {
            req = req.header("api-key", key);
        }
        let resp = req.send().await?.error_for_status()?;
        let body: SearchResp = resp.json().await?;
        Ok(body.result)
    }
}
