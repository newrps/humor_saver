"""Discoverer 메인 — 일일 1회 실행

A. RSS auto-discovery: 깨진/없는 RSS URL 자동 복구
B. 빅카인즈 매체 디렉토리: 신규 매체 자동 추가 (API key 필요)

cron: .env 의 DISCOVERER_TIME (HH:MM, 기본 03:00)
"""
from __future__ import annotations

import os
import time
import traceback
from datetime import datetime

import schedule

import db
import bigkinds
from rss_discovery import find_rss_url


DISCOVERER_TIME = os.environ.get("DISCOVERER_TIME", "03:00")


def now() -> str:
    return datetime.now().strftime("%Y-%m-%d %H:%M:%S")


def discover_rss() -> dict:
    """A. 기존 매체 RSS 자동 발견·복구"""
    targets = db.fetch_rss_discovery_targets()
    print(f"[{now()}] [A.RSS] 대상 매체 {len(targets)}개")

    discovered = []
    failed = []
    for src in targets:
        homepage = src["url"]
        print(f"[{now()}] [A.RSS] {src['name']} ← {homepage}")
        try:
            rss_url = find_rss_url(homepage)
            if rss_url:
                db.update_source_rss(src["id"], rss_url)
                print(f"[{now()}] [A.RSS]   ✓ {rss_url}")
                discovered.append({"name": src["name"], "rss": rss_url})
            else:
                db.update_source_rss(src["id"], None)  # checked_at 만 갱신
                print(f"[{now()}] [A.RSS]   ✗ 발견 실패")
                failed.append(src["name"])
        except Exception as e:
            print(f"[{now()}] [A.RSS]   ✗ 오류: {e}")
            failed.append(src["name"])
    return {"discovered": discovered, "failed": failed}


def discover_bigkinds() -> dict:
    """B. 빅카인즈에서 신규 매체 발견"""
    if not bigkinds.is_enabled():
        print(f"[{now()}] [B.빅카인즈] BIGKINDS_API_KEY 없음 → 스킵")
        return {"added": [], "skipped": True}

    providers = bigkinds.fetch_provider_list()
    print(f"[{now()}] [B.빅카인즈] 등록 매체 {len(providers)}개 받음")

    existing = db.existing_source_names()
    added = []
    for p in providers:
        name = p.get("provider_name") or p.get("name")
        if not name or name in existing:
            continue
        try:
            new_id = db.insert_source(
                name=name,
                category="general_daily",  # 빅카인즈 카테고리 매핑은 추후 보강
                url=p.get("url", ""),
                discovery_source="bigkinds",
            )
            if new_id:
                print(f"[{now()}] [B.빅카인즈]   + {name}")
                added.append(name)
        except Exception as e:
            print(f"[{now()}] [B.빅카인즈]   ✗ {name} 추가 실패: {e}")

    # 새로 추가된 매체에 대해서도 RSS auto-discovery
    if added:
        print(f"[{now()}] [B.빅카인즈] 신규 매체 {len(added)}개 RSS 자동 발견 시도...")
        # 다음 라운드의 discover_rss() 가 자동으로 잡아감 (rss_url IS NULL 조건)
    return {"added": added}


def run_discovery() -> None:
    print(f"\n[{now()}] === Discovery 시작 ===")
    run_id = db.start_run()
    try:
        a_result = discover_rss()
        b_result = discover_bigkinds()
        # 신규 매체에 대해 즉시 RSS 발견 한 번 더
        if b_result.get("added"):
            a_result_2 = discover_rss()
            a_result["discovered"].extend(a_result_2["discovered"])
            a_result["failed"].extend(a_result_2["failed"])

        db.finish_run(
            run_id,
            rss_discovered=len(a_result["discovered"]),
            rss_failed=len(a_result["failed"]),
            new_sources=len(b_result.get("added", [])),
            notes={"rss": a_result, "bigkinds": b_result},
        )
        print(f"[{now()}] === 완료 ===")
        print(f"   RSS 발견: {len(a_result['discovered'])}개")
        print(f"   RSS 실패: {len(a_result['failed'])}개")
        print(f"   신규 매체: {len(b_result.get('added', []))}개")
    except Exception:
        err = traceback.format_exc()
        print(err)
        db.finish_run(run_id, error=err[:1000])


def main() -> None:
    print(f"[{now()}] Discoverer 시작 (매일 {DISCOVERER_TIME} 실행)")

    # 즉시 1회 실행 옵션
    if os.environ.get("DISCOVERER_RUN_ON_BOOT", "false").lower() == "true":
        run_discovery()

    schedule.every().day.at(DISCOVERER_TIME).do(run_discovery)

    while True:
        schedule.run_pending()
        time.sleep(60)


if __name__ == "__main__":
    main()
