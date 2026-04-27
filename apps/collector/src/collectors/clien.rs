//! 클리앙 HTML 스크래퍼
//!
//! 클리앙 RSS 가 사라져서 (404) HTML 직접 파싱.
//! 패턴:
//!   <a class="list_subject" href="/service/board/park/19183252?...">
//!     <span class="subject_fixed" data-role="list-title-text" title="제목 텍스트">

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::time::Duration;
use tracing::info;

use crate::db::{self, NewArticle, Source};

const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36";

pub async fn collect(pool: &PgPool, src: &Source) -> Result<i64> {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()?;

    let resp = client.get(&src.url).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}", resp.status()));
    }
    let html = resp.text().await?;

    let entries = parse(&html);
    let total = entries.len();
    let mut new_count = 0_i64;

    for (path, title) in entries {
        let url = format!("https://www.clien.net{path}");
        // 외부 ID = 게시글 번호
        let external_id = path
            .trim_start_matches("/service/board/park/")
            .trim_start_matches("/service/board/news/")
            .split(&['?', '#'][..])
            .next()
            .map(|s| s.to_string());

        let inserted = db::upsert_article(
            pool,
            &NewArticle {
                source_id: src.id,
                external_id,
                url,
                title,
                summary: None,
                content: None,
                author: None,
                category: None,
                published_at: None, // 목록 페이지엔 날짜 없음
                image_url: None,
                language: Some(src.language.trim().to_string()),
                tags: None,
            },
        )
        .await?;
        new_count += inserted;
    }

    info!("[{}] HTML {}건 신규 (전체 {}건)", src.name, new_count, total);
    Ok(new_count)
}

/// 페이지 HTML → [(path, title)] 추출
fn parse(html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    // <a class="list_subject" href="..." 와 그 뒤의 <span ... title="..."> 페어 찾기
    let mut cursor = 0;
    while let Some(a_pos) = html[cursor..].find("class=\"list_subject\"") {
        let abs = cursor + a_pos;
        // 이 <a> 의 href 추출
        let Some(href_start) = html[abs..].find("href=\"") else { break; };
        let href_open = abs + href_start + 6;
        let Some(href_end_rel) = html[href_open..].find('"') else { break; };
        let path = html[href_open..href_open + href_end_rel].to_string();

        // 게시글이 아닌 (공지/규칙) 건너뛰기 — board path 가 park 또는 news 가 아닌 것
        let is_post = path.contains("/board/park/") || path.contains("/board/news/");

        // 그 <a> 안에 있는 첫 <span ... title="..."> 가 제목
        let scan_start = href_open + href_end_rel;
        let Some(close_pos) = html[scan_start..].find("</a>") else { break; };
        let a_block = &html[scan_start..scan_start + close_pos];

        if is_post {
            if let Some(title) = extract_title(a_block) {
                out.push((path, title));
            }
        }
        cursor = scan_start + close_pos + 4;
    }
    out
}

/// <a> 블록 안에서 title="..." 추출
fn extract_title(a_block: &str) -> Option<String> {
    let title_pos = a_block.find("title=\"")?;
    let start = title_pos + 7;
    let end = a_block[start..].find('"')?;
    let raw = &a_block[start..start + end];
    let t = html_unescape(raw).trim().to_string();
    if t.is_empty() { None } else { Some(t) }
}

/// 최소한의 HTML entity 디코드
fn html_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&#34;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
}
