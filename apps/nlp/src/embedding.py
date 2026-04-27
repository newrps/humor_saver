"""TEI (HuggingFace text-embeddings-inference) 클라이언트"""
from __future__ import annotations

import os

import httpx
from tenacity import retry, stop_after_attempt, wait_exponential

TEI_URL = os.environ["TEI_URL"].rstrip("/")
EMBEDDING_MODEL = os.environ.get("EMBEDDING_MODEL", "unknown")

_client = httpx.Client(timeout=httpx.Timeout(30.0))


@retry(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=2, max=10),
    reraise=True,
)
def embed(text: str) -> list[float]:
    """단일 텍스트 → 임베딩 벡터"""
    if not text or not text.strip():
        raise ValueError("빈 텍스트")
    resp = _client.post(f"{TEI_URL}/embed", json={"inputs": text, "truncate": True})
    resp.raise_for_status()
    data = resp.json()
    # TEI 응답: [[float, ...]]  (inputs가 리스트면 여러 개)
    if not data or not isinstance(data, list):
        raise RuntimeError(f"TEI 비정상 응답: {data!r}")
    return data[0] if isinstance(data[0], list) else data


def get_dimension() -> int:
    """첫 임베딩 호출로 차원 확인"""
    return len(embed("dimension probe"))
