-- Initial database schema for e-reader API server

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Custom types
CREATE TYPE book_format AS ENUM ('epub', 'pdf', 'cbz', 'mobi');
CREATE TYPE annotation_type AS ENUM ('highlight', 'note', 'bookmark');

-- Users table (synced from Clerk via webhook)
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- Clerk user ID
    email TEXT,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Devices
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    device_type TEXT NOT NULL DEFAULT 'unknown',
    public_key TEXT,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_devices_user_id ON devices(user_id);

-- Books
CREATE TABLE books (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    authors TEXT[] NOT NULL DEFAULT '{}',
    description TEXT,
    language TEXT,
    publisher TEXT,
    published_date TEXT,
    isbn TEXT,
    series_name TEXT,
    series_index REAL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_user_id ON books(user_id);
CREATE INDEX idx_books_title ON books(user_id, title);
CREATE INDEX idx_books_series ON books(user_id, series_name) WHERE series_name IS NOT NULL;
CREATE INDEX idx_books_created_at ON books(user_id, created_at DESC);

-- File assets
CREATE TABLE file_assets (
    id UUID PRIMARY KEY,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    format book_format NOT NULL,
    file_size BIGINT NOT NULL,
    content_hash TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(book_id, format)
);

CREATE INDEX idx_file_assets_book_id ON file_assets(book_id);
CREATE INDEX idx_file_assets_content_hash ON file_assets(content_hash);

-- Covers
CREATE TABLE covers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    size_variant TEXT NOT NULL,  -- 'small', 'medium', 'large'
    width INT NOT NULL,
    height INT NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(book_id, size_variant)
);

CREATE INDEX idx_covers_book_id ON covers(book_id);

-- Collections/Shelves
CREATE TABLE collections (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    collection_type TEXT NOT NULL DEFAULT 'shelf',  -- 'shelf', 'tag', 'series'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name, collection_type)
);

CREATE INDEX idx_collections_user_id ON collections(user_id);

-- Collection membership
CREATE TABLE collection_books (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sort_order INT,
    PRIMARY KEY (collection_id, book_id)
);

-- Reading state
CREATE TABLE reading_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    location JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, book_id)
);

CREATE INDEX idx_reading_states_user_book ON reading_states(user_id, book_id);
CREATE INDEX idx_reading_states_updated ON reading_states(user_id, updated_at);

-- Annotations
CREATE TABLE annotations (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    annotation_type annotation_type NOT NULL,
    location_start TEXT NOT NULL,
    location_end TEXT,
    content TEXT,
    style TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ  -- Soft delete for sync
);

CREATE INDEX idx_annotations_user_book ON annotations(user_id, book_id);
CREATE INDEX idx_annotations_updated ON annotations(user_id, updated_at);

-- Background tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'running', 'completed', 'failed'
    priority INT NOT NULL DEFAULT 0,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_status_scheduled ON tasks(status, scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_tasks_type ON tasks(task_type);

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to relevant tables
CREATE TRIGGER update_books_updated_at
    BEFORE UPDATE ON books
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_collections_updated_at
    BEFORE UPDATE ON collections
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_annotations_updated_at
    BEFORE UPDATE ON annotations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
