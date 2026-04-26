"""한국어 형태소 분석 키워드 추출 (Kiwi)"""
from __future__ import annotations

from collections import Counter
from kiwipiepy import Kiwi


_kiwi = Kiwi()

# 추출 대상 품사 (명사·고유명사 위주)
_TARGET_TAGS = {
    "NNG",    # 일반명사
    "NNP",    # 고유명사
    "NNB",    # 의존명사
    "SL",     # 외국어 (영어 단어)
}

# 너무 짧은 단어 제외
_MIN_LEN = 2


def extract(text: str, blocked: set[str], top_k: int = 30) -> list[tuple[str, str, float]]:
    """
    text → [(word, pos, score), ...] 상위 top_k개

    score: 빈도 기반 (단순 카운트). 추후 TF-IDF 로 보강 가능.
    """
    if not text or not text.strip():
        return []

    tokens = _kiwi.tokenize(text)

    # 필터: 품사 + 길이 + 차단어
    filtered = []
    for t in tokens:
        if t.tag not in _TARGET_TAGS:
            continue
        word = t.form.strip()
        if len(word) < _MIN_LEN:
            continue
        if word in blocked:
            continue
        # 숫자만으로 된 토큰 제외
        if word.replace(".", "").replace(",", "").isdigit():
            continue
        filtered.append((word, t.tag))

    # 빈도 카운트
    counter: Counter = Counter()
    pos_map: dict[str, str] = {}
    for word, pos in filtered:
        counter[word] += 1
        pos_map[word] = pos

    # 상위 top_k
    most_common = counter.most_common(top_k)
    if not most_common:
        return []

    max_count = most_common[0][1]
    return [(word, pos_map[word], count / max_count) for word, count in most_common]
