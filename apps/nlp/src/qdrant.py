"""Qdrant 클라이언트 + 컬렉션 관리"""
from __future__ import annotations

import os
from uuid import UUID

from qdrant_client import QdrantClient
from qdrant_client.models import (
    Distance,
    PointStruct,
    VectorParams,
)

QDRANT_URL = os.environ["QDRANT_URL"].strip()
# 인라인 주석/공백 방어 + 빈 값은 None
_raw_key = (os.environ.get("QDRANT_API_KEY") or "").strip()
QDRANT_API_KEY: str | None = _raw_key if _raw_key and not _raw_key.startswith("#") else None
COLLECTION = "articles"


client = QdrantClient(url=QDRANT_URL, api_key=QDRANT_API_KEY, timeout=30)


def ensure_collection(dim: int) -> None:
    """컬렉션 없으면 생성. 차원 다르면 경고만 (강제 삭제 X)"""
    existing = {c.name for c in client.get_collections().collections}
    if COLLECTION not in existing:
        client.create_collection(
            collection_name=COLLECTION,
            vectors_config=VectorParams(size=dim, distance=Distance.COSINE),
        )
        print(f"[qdrant] 컬렉션 생성: {COLLECTION} (dim={dim})")
    else:
        info = client.get_collection(COLLECTION)
        existing_dim = info.config.params.vectors.size
        if existing_dim != dim:
            print(
                f"[qdrant] ⚠️  컬렉션 차원 불일치: 기존 {existing_dim}, 신규 {dim}. "
                f"모델 변경했으면 컬렉션 재생성 필요"
            )


def upsert(point_id: UUID, vector: list[float], payload: dict) -> None:
    client.upsert(
        collection_name=COLLECTION,
        points=[PointStruct(id=str(point_id), vector=vector, payload=payload)],
    )


def search(vector: list[float], limit: int = 10) -> list[dict]:
    """의미검색"""
    result = client.search(
        collection_name=COLLECTION,
        query_vector=vector,
        limit=limit,
    )
    return [
        {"id": p.id, "score": p.score, "payload": p.payload} for p in result
    ]
