"""번역 — 외국어 → 한국어. Claude API 사용.

ANTHROPIC_API_KEY 가 없으면 모두 'skipped' 처리 (graceful).
"""
from __future__ import annotations

import os

import httpx
from tenacity import retry, stop_after_attempt, wait_exponential

API_KEY = (os.environ.get("ANTHROPIC_API_KEY") or "").strip()
MODEL = os.environ.get("CLAUDE_MODEL", "claude-haiku-4-5-20251001")
ENABLED = bool(API_KEY)

_client = httpx.Client(timeout=httpx.Timeout(30.0))


@retry(stop=stop_after_attempt(3), wait=wait_exponential(min=2, max=10), reraise=True)
def translate(title: str, summary: str | None, source_lang: str) -> tuple[str, str | None]:
    """
    (title, summary, source_lang) → (translated_title, translated_summary)

    실패 시 예외. 호출자가 'failed' 처리.
    """
    if not ENABLED:
        raise RuntimeError("ANTHROPIC_API_KEY not set")

    body_text = f"제목: {title}"
    if summary:
        body_text += f"\n요약: {summary[:1000]}"

    prompt = (
        f"다음 {source_lang} 기사를 한국어로 자연스럽게 번역하세요. "
        f"제목과 요약만 출력. 설명·인사 금지.\n\n"
        f"형식:\n"
        f"제목: <번역된 제목>\n"
        f"요약: <번역된 요약 또는 빈 줄>\n\n"
        f"---\n{body_text}"
    )

    resp = _client.post(
        "https://api.anthropic.com/v1/messages",
        headers={
            "x-api-key": API_KEY,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
        json={
            "model": MODEL,
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": prompt}],
        },
    )
    resp.raise_for_status()
    data = resp.json()

    text = ""
    for block in data.get("content", []):
        if block.get("type") == "text":
            text += block.get("text", "")

    # 파싱: 제목: ..., 요약: ...
    out_title = ""
    out_summary = None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("제목:"):
            out_title = line[len("제목:"):].strip()
        elif line.startswith("요약:"):
            out_summary = line[len("요약:"):].strip() or None

    if not out_title:
        # 파싱 실패 시 본문 첫 줄을 제목으로
        out_title = text.strip().splitlines()[0] if text.strip() else title
    return out_title, out_summary
