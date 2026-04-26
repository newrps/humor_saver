"""PostgreSQL 헬퍼"""
from __future__ import annotations

import os
from contextlib import contextmanager
from typing import Iterator, Optional

import psycopg
from psycopg.rows import dict_row

DATABASE_URL = os.environ["DATABASE_URL"]


@contextmanager
def conn() -> Iterator[psycopg.Connection]:
    with psycopg.connect(DATABASE_URL, autocommit=False, row_factory=dict_row) as c:
        yield c


# ─── A. RSS auto-discovery 대상 ────────────────────────
def fetch_rss_discovery_targets() -> list[dict]:
    """
    RSS URL 갱신/발견이 필요한 매체:
    - rss_url 이 없거나
    - 연속 실패 5회 이상이거나
    - 마지막 성공이 7일 이전
    그리고 마지막 RSS 체크가 24시간 전 (또는 한 번도 안 한 매체)
    """
    with conn() as c, c.cursor() as cur:
        cur.execute(
            """
            SELECT id, name, url, rss_url, consecutive_errors, last_success_at
              FROM sources
             WHERE enabled = TRUE
               AND (rss_url IS NULL
                    OR consecutive_errors >= 5
                    OR (last_success_at IS NULL OR last_success_at < NOW() - INTERVAL '7 days'))
               AND (rss_url_checked_at IS NULL OR rss_url_checked_at < NOW() - INTERVAL '20 hours')
             ORDER BY consecutive_errors DESC, id
            """
        )
        return cur.fetchall()


def update_source_rss(source_id: int, rss_url: Optional[str]) -> None:
    with conn() as c, c.cursor() as cur:
        if rss_url:
            cur.execute(
                """
                UPDATE sources
                   SET rss_url = %s,
                       rss_url_checked_at = NOW(),
                       consecutive_errors = 0
                 WHERE id = %s
                """,
                (rss_url, source_id),
            )
        else:
            # 발견 실패 — checked_at 만 갱신 (다음 24h 동안 재시도 안 함)
            cur.execute(
                "UPDATE sources SET rss_url_checked_at = NOW() WHERE id = %s",
                (source_id,),
            )
        c.commit()


# ─── B. 빅카인즈 매체 추가 ─────────────────────────────
def existing_source_names() -> set[str]:
    with conn() as c, c.cursor() as cur:
        cur.execute("SELECT name FROM sources")
        return {r["name"] for r in cur.fetchall()}


def insert_source(
    name: str, category: str, url: str, *, rss_url: Optional[str] = None,
    region: Optional[str] = None, language: str = "ko",
    discovery_source: str = "manual",
) -> int:
    with conn() as c, c.cursor() as cur:
        cur.execute(
            """
            INSERT INTO sources
                (name, category, url, rss_url, region, language, enabled,
                 discovered_at, discovery_source)
            VALUES (%s, %s, %s, %s, %s, %s, TRUE, NOW(), %s)
            ON CONFLICT (name) DO NOTHING
            RETURNING id
            """,
            (name, category, url, rss_url, region, language, discovery_source),
        )
        row = cur.fetchone()
        c.commit()
        return row["id"] if row else 0


# ─── 실행 로그 ─────────────────────────────────────────
def start_run() -> int:
    with conn() as c, c.cursor() as cur:
        cur.execute(
            "INSERT INTO discovery_runs (status) VALUES ('running') RETURNING id"
        )
        rid = cur.fetchone()["id"]
        c.commit()
        return rid


def finish_run(
    run_id: int,
    *,
    rss_discovered: int = 0,
    rss_failed: int = 0,
    new_sources: int = 0,
    notes: dict | None = None,
    error: str | None = None,
) -> None:
    import json
    with conn() as c, c.cursor() as cur:
        cur.execute(
            """
            UPDATE discovery_runs
               SET finished_at = NOW(),
                   status = %s,
                   rss_urls_discovered = %s,
                   rss_urls_failed = %s,
                   new_sources_added = %s,
                   notes = %s,
                   error_message = %s
             WHERE id = %s
            """,
            (
                "failed" if error else "success",
                rss_discovered, rss_failed, new_sources,
                json.dumps(notes or {}, ensure_ascii=False),
                error, run_id,
            ),
        )
        c.commit()
