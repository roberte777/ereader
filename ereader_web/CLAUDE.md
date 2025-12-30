# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the web frontend for a custom e-reader ecosystem. It will provide a web interface for managing ebook libraries, reading progress, and syncing across devices. The backend API is located at `../ereader_backend/`.

**Technology Stack:**
- Framework: Next.js 16.1 (App Router)
- React: 19.2
- Styling: Tailwind CSS 4
- TypeScript: Strict mode
- Package Manager: Bun

**Current State:** Bootstrapped with Next.js template. No business logic implemented yet.

## Common Development Commands

```bash
# Install dependencies
bun install

# Start development server (http://localhost:3000)
bun run dev

# Build for production
bun run build

# Start production server
bun run start

# Run linting
bun run lint
```

## Project Structure

- `app/` - Next.js App Router pages and layouts
- `app/globals.css` - Global styles and Tailwind imports
- `public/` - Static assets

## Related Documentation

- Backend API design: `../ereader_backend/Rust API Server Design.md`
- Backend architecture: `../ereader_backend/CLAUDE.md`
- Implementation roadmap: `../ereader_backend/IMPLEMENTATION_PLAN.md`
- Project scope: `../project_outline.md`
