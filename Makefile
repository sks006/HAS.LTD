APP_NAME=has-ltd

.PHONY: setup frontend backend dev

setup:
	@echo "Project scaffold created"

frontend:
	cd frontend && npm install && npm run dev

backend:
	cd backend && cargo run -p api --bin api

dev:
	@echo "Run 'make frontend' and 'make backend' in separate terminals"
