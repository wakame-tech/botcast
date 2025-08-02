# Development Commands

## Backend (botcast-worker/)

### Running Services
```bash
# Run the main worker service (default)
just
# or explicitly:
RUST_BACKTRACE=1 cargo run -p worker

# Run the API server only
just api
# or:
cargo run -p api
```

### Development Tasks
```bash
# Type checking
cargo check
just check

# Generate OpenAPI server and client code
just generate
# or individually:
just generate_openapi_server
just generate_openapi_client

# Run tests
cargo test
```

### Database Management
```bash
# SeaORM migrations (requires sea-orm-cli)
sea-orm-cli migrate up -u "DATABASE_URL"
```

## Frontend (web/)

### Running Development Server
```bash
# Development server with hot reload
npm run dev
```

### Code Quality
```bash
# Type checking, linting, and formatting
npm run check

# Generate TypeScript types from OpenAPI spec
npm run generate

# Linting and formatting (Biome)
npm run lint
npm run format
```

### Building
```bash
# Build for production
npm run build
```

## Important Notes
- Use `just api` instead of raw `cargo run` for proper .env loading
- Run database migrations before starting API server
- Use proper working directories for commands (botcast-worker/ for Rust, web/ for frontend)
- Regenerate OpenAPI code after spec changes with `just generate` + `npm run generate`