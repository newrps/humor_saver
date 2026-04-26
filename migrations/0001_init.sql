-- news-tracker 초기 스키마
-- PostgreSQL 16+

-- 매체 (sources.json 의 캐시. collector 가 부팅 시 sync)
CREATE TABLE IF NOT EXISTS sources (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    url TEXT NOT NULL,
    rss_url TEXT,
    region TEXT,
    field TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 뉴스 기사
CREATE TABLE IF NOT EXISTS articles (
    id BIGSERIAL PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES sources(id),
    external_id TEXT,                     -- 빅카인즈 news_id 또는 RSS guid
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    content TEXT,                         -- 본문 (선택, 저작권 주의)
    author TEXT,
    category TEXT,                        -- 매체 자체 카테고리
    published_at TIMESTAMPTZ,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- NLP 처리 상태
    nlp_processed BOOLEAN NOT NULL DEFAULT FALSE,
    nlp_processed_at TIMESTAMPTZ,
    embedding_id UUID,                    -- Qdrant 의 point id (UUID)

    UNIQUE (source_id, url),
    UNIQUE (source_id, external_id)
);

CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_collected ON articles(collected_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_source ON articles(source_id);
CREATE INDEX IF NOT EXISTS idx_articles_pending_nlp ON articles(nlp_processed) WHERE nlp_processed = FALSE;

-- PostgreSQL 풀텍스트 검색 (벡터 검색 보완용 키워드 매칭)
ALTER TABLE articles ADD COLUMN IF NOT EXISTS tsv tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(summary, '')), 'B') ||
        setweight(to_tsvector('simple', coalesce(content, '')), 'C')
    ) STORED;
CREATE INDEX IF NOT EXISTS idx_articles_tsv ON articles USING GIN(tsv);

-- 키워드 (Mecab 형태소 + TF-IDF 등으로 추출)
CREATE TABLE IF NOT EXISTS keywords (
    id SERIAL PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,
    pos TEXT,                             -- 품사 (NNP, NNG 등)
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_keywords_word ON keywords(word);

-- 기사 ↔ 키워드 (가중치 포함)
CREATE TABLE IF NOT EXISTS article_keywords (
    article_id BIGINT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    keyword_id INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    score REAL NOT NULL DEFAULT 1.0,      -- TF-IDF 등 가중치
    PRIMARY KEY (article_id, keyword_id)
);
CREATE INDEX IF NOT EXISTS idx_article_keywords_keyword ON article_keywords(keyword_id);

-- 일별 키워드 트렌드 (집계 테이블 - 대시보드용)
CREATE TABLE IF NOT EXISTS keyword_trends_daily (
    day DATE NOT NULL,
    keyword_id INTEGER NOT NULL REFERENCES keywords(id) ON DELETE CASCADE,
    article_count INTEGER NOT NULL,
    score_sum REAL NOT NULL,
    PRIMARY KEY (day, keyword_id)
);
CREATE INDEX IF NOT EXISTS idx_keyword_trends_day ON keyword_trends_daily(day DESC);

-- 수집 작업 로그 (운영 모니터링용)
CREATE TABLE IF NOT EXISTS collection_runs (
    id BIGSERIAL PRIMARY KEY,
    source_id INTEGER REFERENCES sources(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    status TEXT NOT NULL,                 -- 'running' | 'success' | 'failed'
    new_articles INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);
CREATE INDEX IF NOT EXISTS idx_collection_runs_started ON collection_runs(started_at DESC);
