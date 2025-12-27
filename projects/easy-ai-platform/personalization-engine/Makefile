.PHONY: dev build test clean docker-up docker-down

# Development
dev:
	cargo run

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt && cargo clippy

clean:
	cargo clean

# Docker
docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

# Database
db-init:
	docker-compose exec mysql mysql -uroot -proot_password personalization < scripts/init-mysql.sql

# Gorse
gorse-dashboard:
	@echo "Open http://localhost:8088 in browser"
