-- Migration: Merge file_assets into books table
-- This changes the data model from one-to-many (book to files) to one-to-one (one file per book)
-- and restricts supported formats to only EPUB

-- Step 1: Add file columns to books table
ALTER TABLE books ADD COLUMN format book_format;
ALTER TABLE books ADD COLUMN content_hash TEXT;
ALTER TABLE books ADD COLUMN file_size BIGINT;
ALTER TABLE books ADD COLUMN storage_path TEXT;
ALTER TABLE books ADD COLUMN original_filename TEXT;

-- Step 2: Migrate existing data from file_assets (taking first/oldest asset per book)
UPDATE books b SET
    format = fa.format,
    content_hash = fa.content_hash,
    file_size = fa.file_size,
    storage_path = fa.storage_path,
    original_filename = fa.original_filename
FROM (
    SELECT DISTINCT ON (book_id) *
    FROM file_assets
    ORDER BY book_id, created_at ASC
) fa
WHERE b.id = fa.book_id;

-- Step 3: Drop file_assets table first (removes dependency on book_format enum)
DROP INDEX IF EXISTS idx_file_assets_book_id;
DROP INDEX IF EXISTS idx_file_assets_content_hash;
DROP TABLE file_assets;

-- Step 4: Recreate book_format enum with only 'epub'
-- PostgreSQL doesn't allow removing values from enums, so we create a new one
CREATE TYPE book_format_new AS ENUM ('epub');

-- Convert existing data - this will fail if there are non-epub records
-- which is intentional to prevent data loss
ALTER TABLE books
    ALTER COLUMN format TYPE book_format_new
    USING format::text::book_format_new;

-- Drop old type and rename new one
DROP TYPE book_format;
ALTER TYPE book_format_new RENAME TO book_format;

-- Step 5: Add index for content_hash deduplication queries
CREATE INDEX idx_books_content_hash ON books(content_hash) WHERE content_hash IS NOT NULL;
