# Code Style & Conventions

## Rust Backend Conventions
- **Standard Rust formatting**: Use `cargo fmt` for consistent formatting
- **Error handling**: Uses `anyhow::Error` for error propagation
- **Async patterns**: Heavily uses async/await with tokio runtime
- **Dependency injection**: Repository pattern with trait-based providers
- **Naming**: Snake_case for functions/variables, PascalCase for types
- **Workspace structure**: Multi-crate workspace with specialized responsibilities

## TypeScript/React Frontend Conventions

### Code Quality Tools
- **Biome**: Used for linting and formatting (configured in `biome.json`)
- **Formatting**: Tab indentation, double quotes for strings
- **Import organization**: Automatic import sorting enabled

### React Patterns
- **Components**: Functional components with hooks
- **State management**: TanStack Query for server state, Jotai for client state
- **Routing**: File-based routing with TanStack Router
- **Styling**: UnoCSS with Tailwind-compatible classes + shadcn/ui components
- **TypeScript**: Strict type checking enabled

### API Integration
- **Type safety**: Auto-generated TypeScript types from OpenAPI spec
- **Queries**: React Query hooks generated from OpenAPI spec
- **Client**: openapi-fetch for type-safe API calls

## OpenAPI Spec Conventions
- **Schema definition**: Explicit schemas in `components/schemas` (avoid inline schemas)
- **Naming**: Use descriptive, specific request/response type names
- **References**: Use `$ref` instead of inline type definitions
- **Error responses**: Consistent error response structure

## File Organization
- **Backend**: Domain-driven module organization (auth, episodes, scripts, etc.)
- **Frontend**: Feature-based component organization
- **Generated code**: Separate directories for generated OpenAPI code
- **Tests**: Co-located with source files when present