"""RSS auto-discovery — 매체 홈페이지 HTML 의 <link rel='alternate'> 추출"""
from __future__ import annotations

from urllib.parse import urljoin, urlparse

import httpx
from bs4 import BeautifulSoup

USER_AGENT = "news-tracker-discoverer/0.1 (+https://github.com/newrps/news_saver)"

RSS_TYPES = {
    "application/rss+xml",
    "application/atom+xml",
    "application/xml",
    "text/xml",
}


def find_rss_url(homepage_url: str, timeout: float = 15.0) -> str | None:
    """
    매체 홈페이지에서 RSS feed URL 자동 발견.
    1. HTML <head> 의 <link rel="alternate" type="application/rss+xml">
    2. 흔한 RSS 경로 추측 (/rss, /rss.xml, /feed, /feed.xml, /rss/all 등)
    """
    url = homepage_url
    headers = {"User-Agent": USER_AGENT}

    # 1) HTML head 의 link rel
    try:
        with httpx.Client(timeout=timeout, follow_redirects=True, headers=headers) as cli:
            resp = cli.get(url)
            resp.raise_for_status()
            content_type = resp.headers.get("content-type", "")
            if "html" not in content_type.lower():
                return None
            soup = BeautifulSoup(resp.text, "lxml")
            for link in soup.find_all("link", rel=lambda v: v and "alternate" in (v if isinstance(v, list) else [v])):
                lt = (link.get("type") or "").lower()
                href = link.get("href")
                if href and lt in RSS_TYPES:
                    full = urljoin(url, href)
                    if _validate_rss(full, headers, timeout):
                        return full
    except Exception:
        pass

    # 2) 흔한 RSS 경로 추측
    parsed = urlparse(url)
    base = f"{parsed.scheme}://{parsed.netloc}"
    candidates = [
        "/rss", "/rss.xml", "/feed", "/feed.xml", "/atom.xml",
        "/rss/all", "/rss/total", "/rss/news", "/rss/allArticle.xml",
        "/api/rss", "/api/v3/rss/site/total",
    ]
    for path in candidates:
        full = base + path
        if _validate_rss(full, headers, timeout=8.0):
            return full

    return None


def _validate_rss(url: str, headers: dict, timeout: float) -> bool:
    """URL 이 실제 RSS/Atom XML 인지 GET 헤더+첫 1KB 로 확인"""
    try:
        with httpx.Client(timeout=timeout, follow_redirects=True, headers=headers) as cli:
            resp = cli.get(url)
            if resp.status_code != 200:
                return False
            ct = resp.headers.get("content-type", "").lower()
            body = resp.text[:2048].lower()
            # XML 시그니처 또는 RSS/Atom 루트 태그 확인
            if "xml" in ct or "rss" in ct or "atom" in ct:
                return True
            if "<rss" in body or "<feed" in body or "<?xml" in body:
                return True
            return False
    except Exception:
        return False
