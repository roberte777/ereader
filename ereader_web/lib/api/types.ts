// Core types matching backend API

export interface Book {
  id: string;
  title: string;
  authors: string[];
  description: string | null;
  language: string | null;
  publisher: string | null;
  published_date: string | null;
  isbn: string | null;
  series_name: string | null;
  series_index: number | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface BookWithAssets extends Book {
  formats: BookFormat[];
  has_cover: boolean;
}

export type BookFormat = "epub" | "pdf" | "cbz" | "mobi";

export interface CreateBookRequest {
  title: string;
  authors?: string[];
  description?: string;
  language?: string;
  publisher?: string;
  published_date?: string;
  isbn?: string;
  series_name?: string;
  series_index?: number;
  tags?: string[];
}

export interface UpdateBookRequest {
  title?: string;
  authors?: string[];
  description?: string;
  language?: string;
  publisher?: string;
  published_date?: string;
  isbn?: string;
  series_name?: string;
  series_index?: number;
  tags?: string[];
}

export interface Collection {
  id: string;
  name: string;
  description: string | null;
  collection_type: "shelf" | "tag" | "series";
  book_count: number;
  created_at: string;
  updated_at: string;
}

export interface CollectionWithBooks extends Collection {
  books: Book[];
}

export interface CreateCollectionRequest {
  name: string;
  description?: string;
  collection_type?: "shelf" | "tag" | "series";
}

export interface UpdateCollectionRequest {
  name?: string;
  description?: string;
}

export interface ReadingLocation {
  locator: string;
  progress: number;
  chapter: string | null;
}

export interface ReadingState {
  id: string;
  user_id: string;
  book_id: string;
  device_id: string;
  location: ReadingLocation;
  updated_at: string;
}

export interface UpdateReadingStateRequest {
  location: ReadingLocation;
}

export interface FileAsset {
  id: string;
  book_id: string;
  format: BookFormat;
  file_size: number;
  content_hash: string;
  created_at: string;
}

export interface UploadResponse {
  asset_id: string;
  format: BookFormat;
  file_size: number;
  content_hash: string;
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
  has_more: boolean;
}

export interface ListBooksParams {
  page?: number;
  per_page?: number;
  sort_by?: "title" | "created_at" | "updated_at" | "series_index";
  sort_order?: "asc" | "desc";
  tag?: string;
  series?: string;
  author?: string;
}

export interface SearchBooksParams {
  q: string;
  page?: number;
  per_page?: number;
}

export interface SyncRequest {
  device_id: string;
  last_sync_at?: string;
  reading_states: ReadingStateSync[];
  annotations: AnnotationSync[];
}

export interface ReadingStateSync {
  book_id: string;
  current_location: string;
  progress_percent: number;
  updated_at: string;
  version: number;
}

export interface AnnotationSync {
  id: string;
  book_id: string;
  annotation_type: "highlight" | "note" | "bookmark";
  location: string;
  content: string | null;
  text_content: string | null;
  color: string | null;
  deleted: boolean;
  created_at: string;
  updated_at: string;
  version: number;
}

export interface SyncResponse {
  server_time: string;
  reading_states: ReadingStateSync[];
  annotations: AnnotationSync[];
  conflicts: SyncConflict[];
}

export interface SyncConflict {
  entity_type: "reading_state" | "annotation";
  entity_id: string;
  local_updated_at: string;
  server_updated_at: string;
  resolution: "server_wins" | "client_wins";
}

export interface ApiError {
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}
