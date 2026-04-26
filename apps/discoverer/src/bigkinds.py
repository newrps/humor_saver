"""빅카인즈 매체 디렉토리 클라이언트

빅카인즈 API:
  - https://www.bigkinds.or.kr/v2/news/openApi.do
  - https://tools.kinds.or.kr (관리자 페이지)

매체 목록 정확한 endpoint 는 API key 발급 후 문서 참고. 임시로
검색 API 응답에 매체 정보 포함되니 거기서 추출하는 방식도 가능.

지금은 placeholder. API key 받은 뒤 실제 endpoint 로 교체.
"""
from __future__ import annotations

import os

import httpx

BIGKINDS_API_KEY = (os.environ.get("BIGKINDS_API_KEY") or "").strip()
BIGKINDS_BASE = "https://tools.kinds.or.kr"


def is_enabled() -> bool:
    return bool(BIGKINDS_API_KEY)


def fetch_provider_list() -> list[dict]:
    """
    빅카인즈에 등록된 매체 목록 가져오기.
    실제 endpoint 는 API 신청·승인 후 가이드 문서 확인.
    응답 형식 예시 (실제와 다를 수 있음):
        [{"provider_code":"01100101","provider_name":"경향신문","url":"..."}]
    """
    if not is_enabled():
        return []

    # TODO: 실제 빅카인즈 endpoint 로 교체
    # body = {"access_key": BIGKINDS_API_KEY}
    # r = httpx.post(f"{BIGKINDS_BASE}/v3/openApi/providerList", json=body, timeout=30)
    # r.raise_for_status()
    # return r.json().get("return_object", {}).get("providers", [])

    # 현재는 빈 리스트 (API key 받기 전)
    return []
