# Hermes Game Operator Makefile
# Common commands for development and deployment

.PHONY: help install build test dev clean lint format

help: ## Show this help message
	@echo "Hermes Game Operator - Available Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

install: ## Install dependencies
	npm install
	cd src-tauri && cargo fetch

build: ## Build frontend
	npm run build

build-tauri: ## Build Tauri application
	npm run tauri build

test: ## Run unit tests
	npm run test

test-e2e: ## Run E2E tests
	npm run test:e2e

dev: ## Start development server
	npm run tauri dev

dev-web: ## Start web development server
	npm run dev

lint: ## Run linter
	npm run lint

format: ## Format code
	npm run format || true
	cd src-tauri && cargo fmt

clean: ## Clean build artifacts
	rm -rf dist/
	rm -rf dist-web/
	rm -rf node_modules/.cache/
	cd src-tauri && cargo clean

check: ## Check code quality
	npm run typecheck
	cd src-tauri && cargo check

release: ## Create a release (usage: make release VERSION=x.x.x)
	./scripts/release.sh $(VERSION)

env-check: ## Check development environment
	./scripts/check-env.sh

setup: ## Setup development environment
	./scripts/setup-dev.sh

docker-build: ## Build Docker image
	docker build -t hermes-game-operator:latest .

docker-run: ## Run Docker container
	docker run -p 1420:1420 hermes-game-operator:latest

git-status: ## Show git status
	git status

git-log: ## Show recent commits
	git log --oneline -10

count: ## Count lines of code
	@echo "TypeScript/Vue:"
	@find src -name "*.ts" -o -name "*.vue" | xargs wc -l | tail -1
	@echo ""
	@echo "Rust:"
	@find src-tauri/src -name "*.rs" | xargs wc -l | tail -1
	@echo ""
	@echo "Tests:"
	@find src/tests -name "*.ts" | xargs wc -l 2>/dev/null | tail -1 || echo "No tests"

.DEFAULT_GOAL := help