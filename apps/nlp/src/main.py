"""NLP 워커 메인 루프

흐름:
  1. embedding_status='pending' 인 articles 가져오기
  2. 각 기사:
     - kiwipiepy 로 키워드 추출 → article_keywords 저장
     - TEI 로 임베딩 생성
     - Qdrant 에 저장
     - articles UPDATE: embedding_id, embedding_model, embedding_status='success', nlp_processed=TRUE
  3. NLP_INTERVAL_SEC 대기 후 반복
"""
from __future__ import annotations

import os
import sys
import time
import traceback
from datetime import datetime
from uuid import uuid4

import db
import embedding
import keywords as kw_extract
from qdrant import COLLECTION, ensure_collection, upsert as qdrant_upsert


INTERVAL = int(os.environ.get("NLP_INTERVAL_SEC", "300"))
BATCH_SIZE = int(os.environ.get("NLP_BATCH_SIZE", "20"))
MODEL = embedding.EMBEDDING_MODEL


def now() -> str:
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def process_one(article: dict, blocked: set[str]) -> bool:
    """기사 1건 처리. 성공 True / 실패 False"""
    aid = article["id"]
    title = article.get("title") or ""
    summary = article.get("summary") or ""
    content = article.get("content") or ""

    # 키워드 추출용 텍스트 (가장 정보 많은 것)
    kw_text = " ".join(filter(None, [title, summary, content]))[:5000]

    # 키워드 추출·저장
    try:
        keywords = kw_extract.extract(kw_text, blocked=blocked, top_k=30)
        db.save_keywords(aid, keywords)
    except Exception as e:
        print(f"[{now()}] [kw FAIL] article_id={aid}: {e}")
        # 키워드 실패해도 임베딩은 진행

    # 임베딩 생성용 텍스트 (제목·요약·본문 우선순위, 길이 제한)
    embed_text = " ".join(filter(None, [title, summary, content]))[:8000]
    if not embed_text.strip():
        print(f"[{now()}] [skip] article_id={aid}: 빈 텍스트")
        db.mark_embedding_failed(aid, "empty text")
        return False

    try:
        vec = embedding.embed(embed_text)
    except Exception as e:
        print(f"[{now()}] [embed FAIL] article_id={aid}: {e}")
        db.mark_embedding_failed(aid, str(e))
        return False

    # Qdrant 저장
    point_id = uuid4()
    payload = {
        "article_id": aid,
        "source_id": article["source_id"],
        "title": title,
        "summary": summary[:500] if summary else None,
        "published_at": article["published_at"].isoformat() if article.get("published_at") else None,
    }
    try:
        qdrant_upsert(point_id, vec, payload)
    except Exception as e:
        print(f"[{now()}] [qdrant FAIL] article_id={aid}: {e}")
        db.mark_embedding_failed(aid, f"qdrant: {e}")
        return False

    db.mark_embedding_success(aid, point_id, MODEL)
    return True


def round_once() -> int:
    """한 번 처리 라운드. 처리한 건수 반환."""
    blocked = db.get_blocked_words()
    pending = db.fetch_pending_articles(limit=BATCH_SIZE)
    if not pending:
        return 0

    print(f"[{now()}] 처리 시작: {len(pending)}건 (모델={MODEL})")
    success = 0
    for art in pending:
        if process_one(art, blocked):
            success += 1
    print(f"[{now()}] 처리 완료: {success}/{len(pending)} 성공")
    return success


def main() -> None:
    print(f"[{now()}] NLP 워커 시작 (interval={INTERVAL}s, batch={BATCH_SIZE})")

    # TEI 응답 대기 + 차원 확인 + 컬렉션 보장
    print(f"[{now()}] TEI 차원 확인...")
    while True:
        try:
            dim = embedding.get_dimension()
            print(f"[{now()}] 임베딩 차원: {dim}")
            break
        except Exception as e:
            print(f"[{now()}] TEI 응답 대기 중... ({e})")
            time.sleep(5)

    ensure_collection(dim)

    while True:
        try:
            processed = round_once()
        except Exception:
            print(f"[{now()}] 라운드 오류:")
            traceback.print_exc(file=sys.stdout)
            processed = 0
        # 큐 비었으면 INTERVAL 대기, 아직 남았으면 바로 다음 배치
        sleep_for = INTERVAL if processed == 0 else 2
        time.sleep(sleep_for)


if __name__ == "__main__":
    main()
