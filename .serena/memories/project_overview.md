# Botcast Project Overview

## Project Purpose
Botcast is a podcast generation system with AI-powered script generation and audio synthesis capabilities.

## Architecture
- **Backend**: Rust-based workspace (`botcast-worker/`) with multiple specialized crates
- **Frontend**: React + TypeScript + TanStack Router (`web/`)  
- **Documentation**: Deno-based documentation site (`doc/`)

## Tech Stack

### Backend (Rust)
- **Framework**: Axum web framework with OpenAPI-first design
- **Database**: PostgreSQL with SeaORM ORM and migrations
- **Authentication**: Supabase Auth integration
- **Storage**: R2 (Cloudflare) object storage
- **Audio**: FFmpeg integration + VoiceVox TTS engine
- **Script Runtime**: Custom JavaScript-like runtime for podcast scripts
- **API**: OpenAPI spec-driven with auto-generated server/client code

### Frontend (TypeScript/React)
- **React 18** with TypeScript
- **Routing**: TanStack Router (file-based routing)
- **State Management**: TanStack Query + Jotai
- **Styling**: UnoCSS with Tailwind-compatible utilities + shadcn/ui
- **API Integration**: Auto-generated TypeScript types and React Query hooks from OpenAPI spec
- **Build Tool**: Vite with hot reload

## Key Features
- Podcast script generation with AI/LLM integration
- Audio synthesis and processing pipeline
- User authentication and content management
- Background task processing system
- Multi-format audio export capabilities