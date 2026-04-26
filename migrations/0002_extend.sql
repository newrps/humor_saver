-- news-tracker 스키마 확장
-- 0001_init.sql 적용 후 실행됨 (자동: 컨테이너 첫 부팅 시)

-- ─── articles 확장 ──────────────────────────────────────
ALTER TABLE articles ADD COLUMN IF NOT EXISTS image_url TEXT;
ALTER TABLE articles ADD COLUMN IF NOT EXISTS image_path TEXT;
                                                       -- NAS 로컬 경로 (예: 2026/04/25/12345.webp)
ALTER TABLE articles ADD COLUMN IF NOT EXISTS image_status TEXT NOT NULL DEFAULT 'pending';
                                                       -- pending|success|failed|skipped|none
ALTER TABLE articles ADD COLUMN IF NOT EXISTS language CHAR(2) NOT NULL DEFAULT 'ko';
ALTER TABLE articles ADD COLUMN IF NOT EXISTS hash TEXT;
                                                       -- 본문 sha256 일부 (16자) - 통신사 재게재 감지
ALTER TABLE articles ADD COLUMN IF NOT EXISTS embedding_model TEXT;
                                                       -- bge-m3 / e5-base 등. 모델 변경 시 재계산 판단
ALTER TABLE articles ADD COLUMN IF NOT EXISTS embedding_status TEXT NOT NULL DEFAULT 'pending';
                                                       -- pending|success|failed|skipped
ALTER TABLE articles ADD COLUMN IF NOT EXISTS entities JSONB;
                                                       -- NER: {"persons":[], "orgs":[], "locs":[]}
ALTER TABLE articles ADD COLUMN IF NOT EXISTS tags TEXT[];

CREATE INDEX IF NOT EXISTS idx_articles_hash ON articles(hash) WHERE hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_articles_pending_image ON articles(image_status) WHERE image_status = 'pending';
CREATE INDEX IF NOT EXISTS idx_articles_pending_embed ON articles(embedding_status) WHERE embedding_status = 'pending';
CREATE INDEX IF NOT EXISTS idx_articles_entities_gin ON articles USING GIN(entities);
CREATE INDEX IF NOT EXISTS idx_articles_tags_gin ON articles USING GIN(tags);

-- 기존 nlp_processed 는 키워드 추출 단계만 의미하도록 유지
-- (image/embedding 은 별도 status 컬럼으로 분리)

-- ─── sources 확장 ───────────────────────────────────────
ALTER TABLE sources ADD COLUMN IF NOT EXISTS last_collected_at TIMESTAMPTZ;
ALTER TABLE sources ADD COLUMN IF NOT EXISTS last_success_at TIMESTAMPTZ;
ALTER TABLE sources ADD COLUMN IF NOT EXISTS consecutive_errors INT NOT NULL DEFAULT 0;
ALTER TABLE sources ADD COLUMN IF NOT EXISTS language CHAR(2) NOT NULL DEFAULT 'ko';
ALTER TABLE sources ADD COLUMN IF NOT EXISTS priority INT NOT NULL DEFAULT 5;
ALTER TABLE sources ADD COLUMN IF NOT EXISTS political_lean TEXT;
                                                       -- 'left' | 'center-left' | 'center' | 'center-right' | 'right' | NULL

CREATE INDEX IF NOT EXISTS idx_sources_priority ON sources(priority, id) WHERE enabled = TRUE;

-- ─── keywords 확장 ──────────────────────────────────────
ALTER TABLE keywords ADD COLUMN IF NOT EXISTS total_count BIGINT NOT NULL DEFAULT 0;
ALTER TABLE keywords ADD COLUMN IF NOT EXISTS is_blocked BOOLEAN NOT NULL DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_keywords_blocked ON keywords(is_blocked);

-- 차단어 시드 (분석에 노이즈가 되는 일반 단어)
INSERT INTO keywords (word, pos, is_blocked) VALUES
    ('기자', 'NNG', TRUE),
    ('오늘', 'NNG', TRUE),
    ('어제', 'NNG', TRUE),
    ('내일', 'NNG', TRUE),
    ('한편', 'NNG', TRUE),
    ('따르면', 'VV', TRUE),
    ('관련', 'NNG', TRUE),
    ('통해', 'VV', TRUE),
    ('대해', 'VV', TRUE),
    ('위해', 'VV', TRUE),
    ('지난', 'VA', TRUE),
    ('이번', 'NNG', TRUE)
ON CONFLICT (word) DO NOTHING;

-- ─── collection_runs 확장 ───────────────────────────────
ALTER TABLE collection_runs ADD COLUMN IF NOT EXISTS articles_seen INT NOT NULL DEFAULT 0;
ALTER TABLE collection_runs ADD COLUMN IF NOT EXISTS duration_ms INT;

-- ─── 트리거: keywords.total_count 자동 갱신 ─────────────
CREATE OR REPLACE FUNCTION trg_update_keyword_count() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE keywords SET total_count = total_count + 1 WHERE id = NEW.keyword_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE keywords SET total_count = total_count - 1 WHERE id = OLD.keyword_id;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_keyword_count ON article_keywords;
CREATE TRIGGER trg_keyword_count
    AFTER INSERT OR DELETE ON article_keywords
    FOR EACH ROW EXECUTE FUNCTION trg_update_keyword_count();

-- ─── 뷰: 매체별 수집 통계 (대시보드용) ──────────────────
CREATE OR REPLACE VIEW v_source_stats AS
SELECT
    s.id,
    s.name,
    s.category,
    s.enabled,
    s.consecutive_errors,
    s.last_collected_at,
    s.last_success_at,
    COUNT(a.id)::BIGINT AS total_articles,
    COUNT(a.id) FILTER (WHERE a.collected_at > NOW() - INTERVAL '24 hours')::BIGINT AS articles_24h,
    COUNT(a.id) FILTER (WHERE a.collected_at > NOW() - INTERVAL '7 days')::BIGINT AS articles_7d,
    MAX(a.published_at) AS latest_article_at
FROM sources s
LEFT JOIN articles a ON a.source_id = s.id
GROUP BY s.id;
