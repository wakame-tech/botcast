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

# botcast-cms の spec から生成したクライアント (cms/crates/client) を crates/cms-client に取り込み直す。
# botcast-cms は private なので git 依存にせず、生成物をコピーして持つ。取り込んだ commit は SOURCE_REV に残す
sync-cms-client cms="../botcast-cms":
    rsync -a --delete {{cms}}/cms/crates/client/ crates/cms-client/
    git -C {{cms}} rev-parse HEAD > crates/cms-client/SOURCE_REV


# Test botcast-cms (Rust)
test-cms:
    cargo test --workspace --manifest-path botcast-cms/cms/Cargo.toml

# --- All ---

# Check both workspaces
check:
    just check-worker
    just check-cms
