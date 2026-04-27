FROM python:3.12-slim

ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    HF_HOME=/data \
    TRANSFORMERS_CACHE=/data \
    TZ=Asia/Seoul

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc g++ && \
    rm -rf /var/lib/apt/lists/*

# CPU 전용 PyTorch (용량 ↓ 1.5GB → 200MB)
RUN pip install --no-cache-dir \
        torch==2.5.1+cpu \
        --index-url https://download.pytorch.org/whl/cpu

RUN pip install --no-cache-dir \
        fastapi==0.115.5 \
        "uvicorn[standard]==0.32.0" \
        transformers==4.46.0 \
        sentencepiece==0.2.0 \
        protobuf==5.28.3

COPY server.py .

EXPOSE 8000
CMD ["uvicorn", "server:app", "--host", "0.0.0.0", "--port", "8000"]
