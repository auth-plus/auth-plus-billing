.PHONY: infra/up
infra/up:
	docker-compose up -d database database-migration

.PHONY: infra/down
infra/down:
	docker-compose down

.PHONY: dev
dev:
	make infra/up
	docker-compose up -d api
	docker-compose exec api sh

.PHONY: test
test:
	make infra/up
	docker-compose up -d api
	docker-compose exec -T api cargo build
	docker-compose exec -T api cargo test
	make clean/docker

.PHONY: clean/docker
clean/docker:
	make infra/down
	docker container prune -f
	docker volume prune -f
	docker image prune -f
	docker network prune -f
	rm -rf db/schema.sql
	rm -f db/schema.sql