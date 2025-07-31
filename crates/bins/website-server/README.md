# Paynet Website Server

A Rust web server for handling Starknet deposit routes with a modern frontend build pipeline.

## Architecture

- **Backend**: Rust server using Axum framework
- **Frontend**: Webpack-based build system with code splitting
- **Package Manager**: pnpm for fast, efficient dependency management

## Quick Start

### Prerequisites

- Node.js 18+ 
- pnpm 8+
- Rust (for backend)

### Installation

```bash
# Install frontend dependencies
pnpm install

# Build frontend for production
pnpm run build

# Or run in development mode (with file watching)
pnpm run dev
