-- 외국어 기사 → 한국어 번역 컬럼 추가
ALTER TABLE articles ADD COLUMN IF NOT EXISTS translated_title TEXT;
ALTER TABLE articles ADD COLUMN IF NOT EXISTS translated_summary TEXT;
ALTER TABLE articles ADD COLUMN IF NOT EXISTS translation_model TEXT;
                                                  -- 'claude-haiku' | 'deepl' | 'skipped'
ALTER TABLE articles ADD COLUMN IF NOT EXISTS translation_status TEXT NOT NULL DEFAULT 'pending';
                                                  -- pending | success | failed | skipped
ALTER TABLE articles ADD COLUMN IF NOT EXISTS translated_at TIMESTAMPTZ;

-- 한국어(ko)는 번역 불필요 → 자동 스킵 표시
UPDATE articles SET translation_status = 'skipped' WHERE language = 'ko';

-- 신규 한국어 기사도 자동 skipped 가 되도록 트리거
CREATE OR REPLACE FUNCTION trg_skip_ko_translation() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.language = 'ko' THEN
        NEW.translation_status = 'skipped';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS skip_ko_translation ON articles;
CREATE TRIGGER skip_ko_translation
    BEFORE INSERT ON articles
    FOR EACH ROW EXECUTE FUNCTION trg_skip_ko_translation();

-- 번역 대기 인덱스 (NLP 워커 큐)
CREATE INDEX IF NOT EXISTS idx_articles_pending_translation
    ON articles(translation_status) WHERE translation_status = 'pending';
