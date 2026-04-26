//! RSS / Atom 피드 수집기

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use feed_rs::parser;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{debug, info};

use crate::db::{self, NewArticle, Source};

// 일부 한국 매체가 봇 UA 차단 → 브라우저 호환 UA 사용
const USER_AGENT: &str =
    "Mozilla/5.0 (compatible; news-tracker/0.1; +https://github.com/newrps/news_saver) AppleWebKit/537.36";

pub async fn collect(pool: &PgPool, src: &Source) -> Result<i64> {
    let rss_url = src
        .rss_url
        .as_deref()
        .ok_or_else(|| anyhow!("rss_url 없음"))?;

    debug!("[{}] GET {}", src.name, rss_url);
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()?;

    // status 코드 무시 (일부 매체가 200 대신 404로 보내면서도 정상 RSS 반환 — bloter, IT조선 등)
    // 본문이 진짜 RSS면 파서가 처리, 아니면 파서가 에러
    let resp = client.get(rss_url).send().await?;
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if bytes.is_empty() {
        return Err(anyhow!("HTTP {} + 빈 본문", status));
    }

    let feed = parser::parse(&bytes[..])
        .map_err(|e| anyhow!("RSS 파싱 실패: {e}"))?;

    let total = feed.entries.len();
    let mut new_count = 0_i64;
    for entry in feed.entries {
        let url = entry
            .links
            .first()
            .map(|l| l.href.clone())
            .unwrap_or_default();
        if url.is_empty() {
            continue;
        }
        let title = entry
            .title
            .as_ref()
            .map(|t| t.content.trim().to_string())
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let summary = entry.summary.as_ref().map(|s| strip_html(&s.content));

        // content:encoded 또는 기본 content
        let content = entry
            .content
            .as_ref()
            .and_then(|c| c.body.as_ref())
            .map(|s| strip_html(s));

        let author = entry.authors.first().map(|a| a.name.clone());
        let published_at: Option<DateTime<Utc>> = entry.published.or(entry.updated);
        let external_id = Some(entry.id.clone());

        // 이미지 URL 추출 - media 우선, 없으면 content/summary 안의 첫 <img src=...>
        let image_url = entry
            .media
            .iter()
            .flat_map(|m| m.content.iter())
            .find_map(|mc| mc.url.as_ref().map(|u| u.to_string()))
            .or_else(|| {
                let html = entry
                    .content
                    .as_ref()
                    .and_then(|c| c.body.as_ref())
                    .or_else(|| entry.summary.as_ref().map(|s| &s.content));
                html.and_then(|h| extract_first_img(h))
            });

        // 카테고리/태그 (RSS 의 categories)
        let tags: Option<Vec<String>> = if entry.categories.is_empty() {
            None
        } else {
            Some(entry.categories.iter().map(|c| c.term.clone()).collect())
        };

        let inserted = db::upsert_article(
            pool,
            &NewArticle {
                source_id: src.id,
                external_id,
                url,
                title,
                summary,
                content,
                author,
                category: None,
                published_at,
                image_url,
                language: None, // 매체 기본값 'ko' 사용
                tags,
            },
        )
        .await?;
        new_count += inserted;
    }

    info!("[{}] RSS {}건 신규 (전체 {}건)", src.name, new_count, total);
    Ok(new_count)
}

/// HTML 안에서 첫 번째 <img src="..."> 추출 (단순 파서)
fn extract_first_img(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let img_pos = lower.find("<img")?;
    let after = &html[img_pos..];
    let src_pos = after.to_lowercase().find("src=")?;
    let after_src = &after[src_pos + 4..];
    let quote = after_src.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after_src[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

/// 매우 단순한 HTML 태그 제거 (요약문 정리용)
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
