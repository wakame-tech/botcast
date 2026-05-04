# botcast monorepo root Justfile

# Start all local services (SurrealDB + Dify sandbox)
up:
    docker compose -f compose-dev.yaml up -d

# Stop all local services
down:
    docker compose -f compose-dev.yaml down

# --- botcast-worker ---

# Run worker (default)
worker:
    just -f botcast-worker/Justfile

# Run API server
api:
    just -f botcast-worker/Justfile api

# Check botcast-worker
check-worker:
    just -f botcast-worker/Justfile check

# --- botcast-cms ---

# Check botcast-cms (Rust)
check-cms:
    cargo check --workspace --manifest-path botcast-cms/cms/Cargo.toml

# Test botcast-cms (Rust)
test-cms:
    cargo test --workspace --manifest-path botcast-cms/cms/Cargo.toml

# --- All ---

# Check both workspaces
check:
    just check-worker
    just check-cms
