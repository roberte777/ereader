"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { booksApi } from "@/lib/api/books";
import type {
  ListBooksParams,
  SearchBooksParams,
  CreateBookRequest,
  UpdateBookRequest,
} from "@/lib/api/types";

export function useBooks(params?: ListBooksParams) {
  return useQuery({
    queryKey: ["books", params],
    queryFn: () => booksApi.list(params),
  });
}

export function useBook(id: string) {
  return useQuery({
    queryKey: ["books", id],
    queryFn: () => booksApi.get(id),
    enabled: !!id,
  });
}

export function useSearchBooks(params: SearchBooksParams) {
  return useQuery({
    queryKey: ["books", "search", params],
    queryFn: () => booksApi.search(params),
    enabled: !!params.q,
  });
}

export function useCreateBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: CreateBookRequest) => booksApi.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["books"] });
    },
  });
}

export function useUpdateBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateBookRequest }) =>
      booksApi.update(id, data),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["books"] });
      queryClient.invalidateQueries({ queryKey: ["books", id] });
    },
  });
}

export function useDeleteBook() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => booksApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["books"] });
    },
  });
}

export function useUploadBookFile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ bookId, file }: { bookId: string; file: File }) =>
      booksApi.upload(bookId, file),
    onSuccess: (_, { bookId }) => {
      queryClient.invalidateQueries({ queryKey: ["books", bookId] });
    },
  });
}
