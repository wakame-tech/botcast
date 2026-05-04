set dotenv-load := true

# API サーバーを起動
api:
    cargo run -p api

# Worker を起動
worker:
    RUST_BACKTRACE=1 cargo run -p worker

# 型チェック
check:
    cargo check

# OpenAPI サーバーコード生成（crates/openapi-gen）
gen_server:
    openapi-generator-cli generate \
        -i spec/tsp-output/schema/openapi.yaml \
        -g rust-axum \
        -o ./crates/openapi-gen

# MCP サーバーコード生成
gen_mcp:
    openapi-mcp-generator \
        --input spec/tsp-output/schema/openapi.yaml \
        --base-url http://localhost:3002 \
        --output ./mcp \
        --force

# TypeSpec → OpenAPI 生成（spec/）
gen_spec:
    cd spec && npm run compile

# すべてのコード生成を実行
gen: gen_spec gen_server gen_mcp

# ローカル開発用 Docker サービスを起動
up:
    docker compose -f compose-dev.yaml up -d

# ローカル開発用 Docker サービスを停止
down:
    docker compose -f compose-dev.yaml down

# バージョン確認
version:
    @grep '^version' crates/api/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/'
