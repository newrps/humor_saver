//! TEI HTTP 클라이언트 — query 문자열을 임베딩으로 변환

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbedRequest<'a> {
    inputs: &'a str,
}

pub async fn embed(tei_url: &str, text: &str) -> anyhow::Result<Vec<f32>> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/embed", tei_url.trim_end_matches('/')))
        .json(&EmbedRequest { inputs: text })
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    // TEI 응답: [[f32, ...]]
    let arr = body
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("TEI 비정상 응답: {body}"))?;
    Ok(arr
        .iter()
        .filter_map(|x| x.as_f64().map(|f| f as f32))
        .collect())
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct _Unused {}
