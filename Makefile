# news-tracker 배포 Makefile
# PC에서 빌드 → NAS로 이미지 전송 → 재시작
#
# 사용법:
#   make deploy           # 전체 (5개 서비스 빌드 + 전송 + 재시작)
#   make deploy-api       # api만
#   make deploy-collector # collector만
#   make logs             # NAS 실시간 로그
#   make ssh              # NAS 셸

# ── 설정 (환경에 맞게 수정) ──────────────────────────────────────
# 내부망. 외부 접속: NAS_HOST=newrps@newrps.synology.me make deploy
NAS_HOST    ?= newrps@192.168.123.110
NAS_PORT    ?= 56822
NAS_PATH    ?= /volume1/docker/news_saver
PROJECT     ?= news-tracker

SSH_OPTS    := -p $(NAS_PORT)
SSH         := ssh $(SSH_OPTS) $(NAS_HOST)
SCP         := scp -P $(NAS_PORT)

# 빌드 가능한 서비스 (image pull 만 하는 postgres/qdrant/tei 제외)
BUILD_SERVICES := api collector discoverer nlp web

# ── 메인 타겟 ────────────────────────────────────────────────────
.PHONY: deploy
deploy: build push restart
	@echo "✅ 배포 완료"

.PHONY: build
build: $(addprefix build-,$(BUILD_SERVICES))

.PHONY: push
push: $(addprefix push-,$(BUILD_SERVICES))

.PHONY: restart
restart:
	@echo "→ NAS에서 docker compose up -d"
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose up -d'

# ── 서비스별 빌드 ────────────────────────────────────────────────
build-%:
	@echo "→ 빌드: $*"
	docker build -t $(PROJECT)-$*:latest ./apps/$*

push-%:
	@echo "→ 전송: $*"
	docker save $(PROJECT)-$*:latest | gzip | $(SSH) 'gunzip | /usr/local/bin/docker load'

# ── 단일 서비스 deploy 단축키 ────────────────────────────────────
deploy-%: build-% push-%
	@echo "→ 재시작: $*"
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose up -d $*'
	@echo "✅ $* 배포 완료"

# ── 유틸리티 ─────────────────────────────────────────────────────
.PHONY: logs
logs:
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose logs -f --tail=100'

.PHONY: logs-%
logs-%:
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose logs -f --tail=100 $*'

.PHONY: ps
ps:
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose ps'

.PHONY: ssh
ssh:
	@$(SSH)

.PHONY: down
down:
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose down'

.PHONY: pull-images
pull-images:
	@echo "→ NAS에서 외부 이미지 pull (postgres, qdrant, tei)"
	@$(SSH) 'cd $(NAS_PATH) && /usr/local/bin/docker compose pull postgres qdrant tei'

# ── 첫 배포용: 설정 파일 동기화 (이미지 X) ───────────────────────
.PHONY: sync-config
sync-config:
	@echo "→ docker-compose.yml + migrations + sources.json 동기화"
	@rsync -avz -e "ssh $(SSH_OPTS)" \
		--exclude 'apps/*/target' \
		--exclude 'apps/web/node_modules' \
		--exclude 'data/postgres' \
		--exclude 'data/qdrant' \
		--exclude 'data/tei-cache' \
		./docker-compose.yml ./migrations ./data/sources.json $(NAS_HOST):$(NAS_PATH)/

.PHONY: help
help:
	@echo "사용 가능한 명령:"
	@echo "  make deploy            전체 배포 (빌드+전송+재시작)"
	@echo "  make deploy-api        api만 배포"
	@echo "  make deploy-collector  collector만 배포"
	@echo "  make deploy-nlp        nlp만 배포"
	@echo "  make deploy-discoverer discoverer만 배포"
	@echo "  make deploy-web        web만 배포"
	@echo "  make pull-images       postgres/qdrant/tei 이미지만 pull (NAS에서)"
	@echo "  make sync-config       docker-compose.yml + migrations 만 NAS에 전송"
	@echo "  make logs              실시간 로그 (전체)"
	@echo "  make logs-api          api 로그만"
	@echo "  make ps                컨테이너 상태"
	@echo "  make down              모든 컨테이너 중지"
	@echo "  make ssh               NAS SSH 접속"
