-- 매체 자동 발견 (discoverer) 워커용 스키마

-- discoverer 실행 로그
CREATE TABLE IF NOT EXISTS discovery_runs (
    id BIGSERIAL PRIMARY KEY,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    status TEXT NOT NULL,                    -- 'running' | 'success' | 'failed'
    rss_urls_discovered INT NOT NULL DEFAULT 0,
    rss_urls_failed INT NOT NULL DEFAULT 0,
    new_sources_added INT NOT NULL DEFAULT 0,
    error_message TEXT,
    notes JSONB                              -- {"discovered":[{"name":"...","rss":"..."}], "added":[...]}
);
CREATE INDEX IF NOT EXISTS idx_discovery_runs_started ON discovery_runs(started_at DESC);

-- sources 에 발견 메타 추가
ALTER TABLE sources ADD COLUMN IF NOT EXISTS discovered_at TIMESTAMPTZ;
                                                   -- discoverer 가 자동 추가한 매체면 시각 기록
ALTER TABLE sources ADD COLUMN IF NOT EXISTS discovery_source TEXT;
                                                   -- 'manual' | 'bigkinds' | 'rss-autodiscover' | 'sources.json'
ALTER TABLE sources ADD COLUMN IF NOT EXISTS rss_url_checked_at TIMESTAMPTZ;
                                                   -- 마지막으로 RSS auto-discovery 시도한 시각
