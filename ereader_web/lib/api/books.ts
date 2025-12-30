import { getApiClient } from "./client";
import type {
  Book,
  BookWithAssets,
  CreateBookRequest,
  UpdateBookRequest,
  ListBooksParams,
  SearchBooksParams,
  Paginated,
  UploadResponse,
} from "./types";

export const booksApi = {
  list(params?: ListBooksParams): Promise<Paginated<Book>> {
    return getApiClient().get<Paginated<Book>>("/books", params);
  },

  get(id: string): Promise<BookWithAssets> {
    return getApiClient().get<BookWithAssets>(`/books/${id}`);
  },

  create(data: CreateBookRequest): Promise<Book> {
    return getApiClient().post<Book>("/books", data);
  },

  update(id: string, data: UpdateBookRequest): Promise<Book> {
    return getApiClient().put<Book>(`/books/${id}`, data);
  },

  delete(id: string): Promise<void> {
    return getApiClient().delete<void>(`/books/${id}`);
  },

  search(params: SearchBooksParams): Promise<Paginated<Book>> {
    return getApiClient().get<Paginated<Book>>("/books/search", params);
  },

  upload(bookId: string, file: File): Promise<UploadResponse> {
    return getApiClient().uploadFile<UploadResponse>(
      `/books/${bookId}/upload`,
      file
    );
  },

  getDownloadUrl(bookId: string, format?: string): string {
    const endpoint = format
      ? `/books/${bookId}/download/${format}`
      : `/books/${bookId}/download`;
    return getApiClient().getDownloadUrl(endpoint);
  },

  getCoverUrl(bookId: string, size?: "small" | "medium" | "large"): string {
    return getApiClient().getCoverUrl(bookId, size);
  },
};
