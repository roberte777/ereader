# E-Reader Web Frontend Implementation Plan

## Overview
Build a modern, sleek web frontend for the e-reader ecosystem with library management, book upload, EPUB/PDF reading, and cross-device sync.

## Tech Stack
- Next.js 16.1 (App Router) + React 19.2
- Tailwind CSS 4 with dark mode
- TypeScript (strict)
- Clerk authentication
- epub.js + pdf.js for reading
- React Query for server state
- Zustand for client state

---

## Project Structure

```
ereader_web/
├── app/
│   ├── (auth)/                   # Public auth routes
│   │   ├── sign-in/[[...sign-in]]/page.tsx
│   │   └── sign-up/[[...sign-up]]/page.tsx
│   ├── (main)/                   # Protected routes
│   │   ├── library/page.tsx      # Main library view
│   │   ├── books/[id]/page.tsx   # Book details
│   │   ├── books/[id]/read/page.tsx  # Reader
│   │   ├── books/upload/page.tsx # Upload
│   │   ├── collections/page.tsx  # Collections list
│   │   └── collections/[id]/page.tsx
│   ├── layout.tsx                # Root with ClerkProvider
│   └── page.tsx                  # Landing/redirect
├── components/
│   ├── ui/                       # Button, Input, Modal, Card, Skeleton
│   ├── layout/                   # Sidebar, Header, MobileNav
│   ├── library/                  # BookGrid, BookCard, Filters, Search
│   ├── reader/                   # EpubReader, PdfReader, Controls
│   ├── collections/              # CollectionCard, CreateModal
│   └── upload/                   # FileDropzone, MetadataForm
├── lib/
│   ├── api/                      # API client layer
│   │   ├── client.ts             # Base client with Clerk token
│   │   ├── books.ts
│   │   ├── collections.ts
│   │   └── types.ts
│   ├── hooks/                    # useBooks, useCollections, useReadingState
│   └── store/                    # Zustand stores
├── middleware.ts                 # Clerk middleware
└── types/
```

---

## Implementation Phases

### Phase 1: Foundation
**Files to create/modify:**
- `middleware.ts` - Clerk route protection
- `app/layout.tsx` - Add ClerkProvider, QueryClientProvider
- `app/(auth)/sign-in/[[...sign-in]]/page.tsx`
- `app/(auth)/sign-up/[[...sign-up]]/page.tsx`
- `lib/api/client.ts` - Base API client
- `lib/api/types.ts` - TypeScript types matching backend
- `lib/utils/cn.ts` - className utility
- `components/ui/button.tsx`, `input.tsx`, `modal.tsx`, `card.tsx`, `skeleton.tsx`

**Dependencies:**
```bash
bun add @clerk/nextjs @tanstack/react-query zustand clsx tailwind-merge lucide-react
```

### Phase 2: Library View
**Files to create:**
- `app/(main)/layout.tsx` - Sidebar + header layout
- `app/(main)/library/page.tsx`
- `components/layout/sidebar.tsx`
- `components/layout/header.tsx`
- `components/library/book-grid.tsx`
- `components/library/book-card.tsx`
- `components/library/library-search.tsx`
- `components/library/library-filters.tsx`
- `lib/api/books.ts`
- `lib/hooks/use-books.ts`
- `lib/store/library-store.ts`

### Phase 3: Book Details & Upload
**Files to create:**
- `app/(main)/books/[id]/page.tsx`
- `app/(main)/books/upload/page.tsx`
- `components/upload/file-dropzone.tsx`
- `components/upload/metadata-form.tsx`
- `lib/api/assets.ts`

**Dependencies:**
```bash
bun add react-hook-form zod @hookform/resolvers react-dropzone
```

### Phase 4: Collections
**Files to create:**
- `app/(main)/collections/page.tsx`
- `app/(main)/collections/[id]/page.tsx`
- `components/collections/collection-card.tsx`
- `components/collections/create-collection-modal.tsx`
- `components/collections/add-to-collection-modal.tsx`
- `lib/api/collections.ts`
- `lib/hooks/use-collections.ts`

### Phase 5: EPUB Reader
**Files to create:**
- `app/(main)/books/[id]/read/page.tsx`
- `components/reader/epub-reader.tsx` (client component)
- `components/reader/reader-controls.tsx`
- `components/reader/reader-settings.tsx`
- `components/reader/table-of-contents.tsx`
- `lib/store/reader-store.ts`
- `lib/hooks/use-reading-state.ts`
- `lib/api/sync.ts`

**Dependencies:**
```bash
bun add epubjs
```

### Phase 6: PDF Reader
**Files to create:**
- `components/reader/pdf-reader.tsx` (client component)
- Copy `pdf.worker.min.mjs` to public/

**Dependencies:**
```bash
bun add react-pdf pdfjs-dist
```

### Phase 7: Polish
- Responsive design adjustments
- Loading states & skeletons
- Error handling & toasts
- Dark mode toggle
- Performance optimization

---

## Key Technical Details

### Clerk Middleware
```typescript
// middleware.ts
import { clerkMiddleware, createRouteMatcher } from '@clerk/nextjs/server';

const isPublicRoute = createRouteMatcher(['/', '/sign-in(.*)', '/sign-up(.*)']);

export default clerkMiddleware(async (auth, request) => {
  if (!isPublicRoute(request)) {
    await auth.protect();
  }
});
```

### API Client Pattern
```typescript
// lib/api/client.ts
export class ApiClient {
  constructor(private getToken: () => Promise<string | null>) {}

  async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const token = await this.getToken();
    const res = await fetch(`${API_URL}/api/v1${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...(token && { Authorization: `Bearer ${token}` }),
      },
    });
    if (!res.ok) throw new ApiError(res.status);
    return res.json();
  }
}
```

### EPUB Reader (Client Component)
```typescript
// components/reader/epub-reader.tsx
'use client';
import ePub from 'epubjs';
// Must be dynamic imported, renders only on client
```

---

## Environment Variables
```bash
# .env.local
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=pk_...
CLERK_SECRET_KEY=sk_...
NEXT_PUBLIC_API_URL=http://localhost:8080
```

---

## Backend API Reference
Endpoints at `/api/v1/`:
- `GET/POST /books` - List/create books
- `GET/PUT/DELETE /books/:id` - Book CRUD
- `GET /books/search?q=` - Search
- `POST /books/:id/upload` - Upload file
- `GET /books/:id/download` - Download file
- `GET/POST /collections` - Collections
- `PUT/DELETE /collections/:id`
- `POST /sync` - Batch sync
- `GET/PUT /sync/reading-state/:book_id`

---

## Estimated Component Count
- 6 pages
- ~25 components
- ~8 hooks
- 3 Zustand stores
- 5 API modules
