# Task Completion Checklist

## After Making Code Changes

### Backend Changes (Rust)
1. **Type checking**: Run `cargo check` or `just check`
2. **Tests**: Run `cargo test` if tests exist
3. **Formatting**: Code is auto-formatted with rustfmt

### Frontend Changes (TypeScript/React)
1. **Quality check**: Run `npm run check` (includes TypeScript, linting, formatting)
2. **Type generation**: Run `npm run generate` if OpenAPI spec changed
3. **Build verification**: Ensure `npm run build` passes if significant changes

### OpenAPI Spec Changes
1. **Update spec**: Modify `doc/spec.yml` with proper schemas
2. **Regenerate backend**: Run `just generate` in `botcast-worker/`
3. **Regenerate frontend**: Run `npm run generate` in `web/`
4. **Update implementations**: Update Rust controllers and frontend code to use new types

### Database Changes
1. **Create migration**: Use SeaORM CLI to create migration files
2. **Apply migration**: Run `sea-orm-cli migrate up` before starting API server
3. **Update entities**: Regenerate SeaORM entities if needed

## Local Development Environment
1. **Database**: Ensure PostgreSQL is running (use `compose-dev.yaml`)
2. **Environment**: Use `just api` for proper .env loading
3. **Dependencies**: Install required tools (sea-orm-cli, openapi-generator-cli)

## Code Quality Standards
- No lint errors or warnings
- Type checking passes without errors
- Consistent code formatting applied
- API endpoints have proper error handling
- Database migrations are reversible
- Generated code is not manually modified