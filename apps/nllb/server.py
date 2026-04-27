"""NLLB-200 번역 서버.

POST /translate {text, source, target} → {translation}

source/target 은 ISO 짧은 코드 (en, ko, ja, zh) 또는 NLLB 형식 (eng_Latn 등) 둘 다 지원.
"""
from __future__ import annotations

import os
import logging

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from transformers import AutoModelForSeq2SeqLM, AutoTokenizer

logging.basicConfig(level=logging.INFO)
log = logging.getLogger("nllb")

MODEL_NAME = os.environ.get("NLLB_MODEL", "facebook/nllb-200-distilled-600M")

# ISO short code → NLLB Flores-200 코드 매핑
LANG_MAP = {
    "ko": "kor_Hang",
    "en": "eng_Latn",
    "ja": "jpn_Jpan",
    "zh": "zho_Hans",
    "zh-cn": "zho_Hans",
    "zh-tw": "zho_Hant",
    "es": "spa_Latn",
    "fr": "fra_Latn",
    "de": "deu_Latn",
    "ru": "rus_Cyrl",
    "vi": "vie_Latn",
    "th": "tha_Thai",
    "id": "ind_Latn",
    "ar": "arb_Arab",
    "hi": "hin_Deva",
    "pt": "por_Latn",
    "it": "ita_Latn",
    "tr": "tur_Latn",
}


def to_nllb(code: str) -> str:
    code = (code or "").lower().strip()
    if "_" in code:           # 이미 NLLB 형식
        return code
    return LANG_MAP.get(code, code)


log.info(f"모델 로드: {MODEL_NAME}")
tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME)
model = AutoModelForSeq2SeqLM.from_pretrained(MODEL_NAME)
log.info("모델 로드 완료")


app = FastAPI(title="NLLB-200 번역 서버")


class TranslateReq(BaseModel):
    text: str
    source: str
    target: str = "ko"


class TranslateResp(BaseModel):
    translation: str
    source: str
    target: str
    model: str = MODEL_NAME


@app.get("/health")
def health():
    return {"status": "ok", "model": MODEL_NAME}


@app.post("/translate", response_model=TranslateResp)
def translate(req: TranslateReq):
    text = (req.text or "").strip()
    if not text:
        raise HTTPException(400, "empty text")
    src = to_nllb(req.source)
    tgt = to_nllb(req.target)

    tokenizer.src_lang = src
    inputs = tokenizer(text, return_tensors="pt", max_length=512, truncation=True)
    forced = tokenizer.convert_tokens_to_ids(tgt)
    if forced is None:
        raise HTTPException(400, f"unknown target lang: {req.target} → {tgt}")

    out = model.generate(
        **inputs,
        forced_bos_token_id=forced,
        max_length=512,
        num_beams=2,
    )
    translation = tokenizer.decode(out[0], skip_special_tokens=True)
    return TranslateResp(translation=translation, source=src, target=tgt)
