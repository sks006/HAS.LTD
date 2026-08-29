#!/bin/bash
# scripts/dev.sh

set -e

TARGET=${1:-all}

run_backend() {
    echo "=================================================="
    echo "🚀 Starting Backend with cargo watch..."
    echo "=================================================="
    # Watch specific folders to prevent unnecessary rebuilds
    cargo watch \
        -w backend/api/src \
        -w backend/domain/src \
        -w backend/infrastructure/src \
        -x "run -p api --bin api"
}

run_frontend() {
    echo "=================================================="
    echo "💻 Starting Frontend Development Server..."
    echo "=================================================="
    cd frontend && npm run dev
}

if [ "$TARGET" = "backend" ]; then
    run_backend
elif [ "$TARGET" = "frontend" ]; then
    run_frontend
elif [ "$TARGET" = "all" ]; then
    # Trap exit signal to kill background processes cleanly
    trap 'kill 0' EXIT
    
    run_backend &
    BACKEND_PID=$!
    
    # Wait a second before starting frontend
    sleep 1
    
    run_frontend &
    FRONTEND_PID=$!
    
    wait
else
    echo "Usage: $0 [backend|frontend|all]"
    exit 1
fi
