"""번역 — 외국어 → 한국어. NLLB-200 (로컬, 무료, 외부 API X).

NLLB 컨테이너 (apps/nllb) 가 떠있어야 동작.
"""
from __future__ import annotations

import os

import httpx
from tenacity import retry, stop_after_attempt, wait_exponential

NLLB_URL = (os.environ.get("NLLB_URL") or "http://nllb:8000").rstrip("/")
MODEL = "nllb-200-distilled-600M"
ENABLED = True   # NLLB 컨테이너 사용 가능 가정. 안 떠있으면 호출 시 실패.

_client = httpx.Client(timeout=httpx.Timeout(60.0))


@retry(stop=stop_after_attempt(3), wait=wait_exponential(min=2, max=10), reraise=True)
def _call(text: str, source: str) -> str:
    resp = _client.post(
        f"{NLLB_URL}/translate",
        json={"text": text, "source": source, "target": "ko"},
    )
    resp.raise_for_status()
    return resp.json()["translation"]


def translate(title: str, summary: str | None, source_lang: str) -> tuple[str, str | None]:
    """
    (title, summary, source_lang) → (translated_title, translated_summary)
    """
    t_title = _call(title, source_lang)
    t_summary = None
    if summary:
        # 너무 길면 자르기 (NLLB max_length 512 토큰)
        s = summary[:1500]
        try:
            t_summary = _call(s, source_lang)
        except Exception:
            # 요약 실패해도 제목은 살림
            t_summary = None
    return t_title, t_summary
