.PHONY: infra/up
infra/up:
	docker compose up -d api database
	HOST=localhost make migration/up

.PHONY: infra/down
infra/down:
	docker compose down -v

.PHONY: dev
dev:
	make infra/up
	docker compose exec api sh

.PHONY: test
test:
	make infra/up
	docker compose exec -T api cargo build
	docker compose exec -T api cargo test -- --test-threads=1
	make clean/docker

.PHONY: migration/up
migration/up:
	docker run -t --network=host -v "$(shell pwd)/db:/db" ghcr.io/amacneil/dbmate:1.16 --url postgres://root:db_password@$(HOST):5432/billing?sslmode=disable --wait --wait-timeout 60s --no-dump-schema up