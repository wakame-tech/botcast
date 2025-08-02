# Codebase Structure

## Root Directory Structure
```
botcast/
├── botcast-worker/     # Rust backend workspace
├── web/               # React frontend
├── doc/               # Documentation + OpenAPI spec
├── .github/           # GitHub workflows
└── CLAUDE.md         # Project instructions
```

## Backend Structure (botcast-worker/crates/)

### Core Crates
- **`api/`**: REST API controllers and Axum server
- **`worker/`**: Background task processing and worker services
- **`repos/`**: Database entities, repositories, and SeaORM integration
- **`script_runtime/`**: Custom JavaScript-like runtime for podcast scripts
- **`audio_generator/`**: Audio synthesis and FFmpeg processing

### Generated Code
- **`openapi_gen/`**: Generated Axum server code from OpenAPI spec
- **`openapi_client/`**: Generated Rust client code

### Supporting Crates
- **`script_llm/`**: LLM integration for script generation
- **`readable_text/`**: HTML to markdown conversion utilities

## Frontend Structure (web/src/)

### Core Directories
- **`routes/`**: File-based routing structure (TanStack Router)
- **`components/`**: Reusable UI components organized by domain
- **`lib/`**: Utilities, API client configuration, and generated types
- **`hooks/`**: Custom React hooks

## Key Files
- **`doc/spec.yml`**: OpenAPI specification (source of truth for API)
- **`botcast-worker/Justfile`**: Backend development commands
- **`web/package.json`**: Frontend scripts and dependencies
- **`migration/`**: Database migration files (SeaORM)

## Development Workflow
1. API changes → Update `doc/spec.yml`
2. Regenerate code → `just generate` (Rust) + `npm run generate` (TS)
3. Database changes → Create and run SeaORM migrations
4. Code quality → `cargo check` + `npm run check`