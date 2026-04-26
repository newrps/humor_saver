"""PostgreSQL 헬퍼 - psycopg3"""
from __future__ import annotations

import os
from contextlib import contextmanager
from typing import Iterator
from uuid import UUID

import psycopg
from psycopg.rows import dict_row


DATABASE_URL = os.environ["DATABASE_URL"]


@contextmanager
def conn() -> Iterator[psycopg.Connection]:
    with psycopg.connect(DATABASE_URL, autocommit=False, row_factory=dict_row) as c:
        yield c


def fetch_pending_articles(limit: int = 20) -> list[dict]:
    """임베딩 처리 대기 중인 기사. 제목·요약 합쳐서 의미있는 것만."""
    with conn() as c, c.cursor() as cur:
        cur.execute(
            """
            SELECT id, source_id, title, summary, content, published_at
              FROM articles
             WHERE embedding_status = 'pending'
               AND title IS NOT NULL
             ORDER BY collected_at ASC
             LIMIT %s
            """,
            (limit,),
        )
        return cur.fetchall()


def mark_embedding_success(article_id: int, embedding_id: UUID, model: str) -> None:
    with conn() as c, c.cursor() as cur:
        cur.execute(
            """
            UPDATE articles
               SET embedding_id = %s,
                   embedding_model = %s,
                   embedding_status = 'success',
                   nlp_processed = TRUE,
                   nlp_processed_at = NOW()
             WHERE id = %s
            """,
            (embedding_id, model, article_id),
        )
        c.commit()


def mark_embedding_failed(article_id: int, error: str) -> None:
    with conn() as c, c.cursor() as cur:
        cur.execute(
            "UPDATE articles SET embedding_status = 'failed' WHERE id = %s",
            (article_id,),
        )
        c.commit()


def save_keywords(article_id: int, keywords: list[tuple[str, str, float]]) -> None:
    """
    keywords: [(word, pos, score), ...]
    keywords 테이블에 word UPSERT, article_keywords에 score 저장
    """
    if not keywords:
        return
    with conn() as c, c.cursor() as cur:
        for word, pos, score in keywords:
            cur.execute(
                """
                INSERT INTO keywords (word, pos)
                VALUES (%s, %s)
                ON CONFLICT (word) DO UPDATE SET pos = COALESCE(keywords.pos, EXCLUDED.pos)
                RETURNING id
                """,
                (word, pos),
            )
            kw_id = cur.fetchone()["id"]
            cur.execute(
                """
                INSERT INTO article_keywords (article_id, keyword_id, score)
                VALUES (%s, %s, %s)
                ON CONFLICT (article_id, keyword_id) DO UPDATE SET score = EXCLUDED.score
                """,
                (article_id, kw_id, score),
            )
        c.commit()


def get_blocked_words() -> set[str]:
    with conn() as c, c.cursor() as cur:
        cur.execute("SELECT word FROM keywords WHERE is_blocked = TRUE")
        return {r["word"] for r in cur.fetchall()}
