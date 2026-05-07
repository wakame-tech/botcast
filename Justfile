# botcast monorepo root Justfile

# Start all local services (SurrealDB + Dify sandbox)
up:
    docker compose -f compose-dev.yaml up -d

# Stop all local services
down:
    docker compose -f compose-dev.yaml down

# --- worker ---

# Run worker (default)
worker:
    cargo run -p worker

# Run API server
api:
    cargo run -p worker -- api

# Check worker
check-worker:
    cargo check -p worker

# --- botcast-cms ---


# Test botcast-cms (Rust)
test-cms:
    cargo test --workspace --manifest-path botcast-cms/cms/Cargo.toml

# --- All ---

# Check both workspaces
check:
    just check-worker
    just check-cms
