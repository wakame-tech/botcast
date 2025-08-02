# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Response Language

**すべてのレスポンスは日本語で行うこと。**

## Project Overview

Botcast is a podcast generation system with a Rust backend and TypeScript/React frontend. The system consists of three main components:

1. **botcast-worker**: Rust backend with OpenAPI-generated server and worker services
2. **web**: React frontend with TypeScript and TanStack Router
3. **doc**: Documentation site built with Deno

## Development Commands

### Rust Backend (botcast-worker/)

```bash
# Run the main worker service
just
# or explicitly:
RUST_BACKTRACE=1 cargo run -p worker

# Run the API server only
just api
# or:
cargo run -p api

# Type checking
cargo check
just check

# Generate OpenAPI server and client
just generate
# or individually:
just generate_openapi_server
just generate_openapi_client

# Run tests
cargo test
```

### Frontend (web/)

```bash
# Development server
npm run dev

# Type checking, linting, and formatting
npm run check

# Build for production
npm run build

# Generate TypeScript types from OpenAPI spec
npm run generate

# Linting and formatting
npm run lint
npm run format
```

### Documentation (doc/)

Uses Deno for building the documentation site.

## Architecture

### Backend Architecture

- **Workspace structure**: Multi-crate workspace with specialized crates
- **API layer**: OpenAPI-first design with generated Axum server (`crates/api/`)
- **Worker layer**: Background task processing (`crates/worker/`)
- **Data layer**: SeaORM with PostgreSQL (`crates/repos/`)
- **Storage**: R2 (Cloudflare) object storage integration
- **Audio processing**: FFmpeg integration and VoiceVox TTS (`crates/audio_generator/`)
- **Script runtime**: Custom JavaScript-like runtime for podcast scripts (`crates/script_runtime/`)

### Frontend Architecture

- **React 18** with TypeScript
- **TanStack Router** for routing with file-based routing in `src/routes/`
- **TanStack Query** for server state management
- **OpenAPI integration**: Auto-generated TypeScript types and React Query hooks
- **Styling**: UnoCSS with Tailwind-compatible utilities and shadcn/ui components
- **State management**: Jotai for client state
- **Authentication**: Supabase Auth integration

### Key Directories

**Backend (`botcast-worker/crates/`):**
- `api/`: REST API controllers and main server
- `worker/`: Background worker and task processing
- `repos/`: Database entities and repository patterns
- `script_runtime/`: Custom runtime for podcast script execution
- `audio_generator/`: Audio synthesis and processing
- `openapi_gen/` & `openapi_client/`: Generated OpenAPI code

**Frontend (`web/src/`):**
- `routes/`: File-based routing structure
- `components/`: Reusable UI components organized by domain
- `lib/`: Utilities and API client configuration
- `hooks/`: Custom React hooks

## OpenAPI Integration

The project uses OpenAPI spec-first development:
- Spec defined in `doc/spec.yml`
- Rust server generated into `crates/openapi_gen/`
- Rust client generated into `crates/openapi_client/`
- TypeScript types generated into `web/src/lib/api.d.ts`

When making API changes, update `doc/spec.yml` first, then regenerate all clients.

## Environment Setup

The backend requires:
- PostgreSQL database (DATABASE_URL)
- Supabase authentication configuration
- R2 storage credentials
- Optional: VoiceVox engine for TTS

Frontend uses Vite for development with hot reload and TypeScript checking.

### Local Development Environment

**Database Setup with SeaORM:**
- SeaORM CLI is required for database migrations: `cargo install sea-orm-cli`
- Migration files are in `migration/` directory (part of workspace)
- Initialize migrations: `sea-orm-cli migrate init`
- Apply migrations: `sea-orm-cli migrate up -u "DATABASE_URL"`
- Database schema must be created before API server can start

**Environment Variables:**
- Use Justfile commands (`just api`) for automatic .env loading instead of raw `cargo run`
- Justfile automatically loads `.env` file with `set dotenv-filename := ".env"`
- Avoids environment variable not found errors when running Rust binaries

**Local Development Setup:**
1. Create `compose-dev.yaml` for local services (PostgreSQL + VoiceVox)
2. Use different port (e.g., 5435) to avoid conflicts with system PostgreSQL
3. Run database migrations before starting API server
4. Use `just api` to start API server with proper environment loading

## Testing

- Rust: Standard `cargo test` for unit tests
- Frontend: No specific test runner configured - check if tests exist before assuming testing approach