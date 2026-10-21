APP_NAME=has-ltd

.PHONY: setup frontend backend dev

setup:
	@echo "Project scaffold created"

frontend:
	./scripts/dev.sh frontend

backend:
	./scripts/dev.sh backend

dev:
	./scripts/dev.sh all

