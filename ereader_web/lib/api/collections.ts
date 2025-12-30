import { getApiClient } from "./client";
import type {
  Collection,
  CollectionWithBooks,
  CreateCollectionRequest,
  UpdateCollectionRequest,
  Paginated,
} from "./types";

export const collectionsApi = {
  list(): Promise<Paginated<Collection>> {
    return getApiClient().get<Paginated<Collection>>("/collections");
  },

  get(id: string): Promise<CollectionWithBooks> {
    return getApiClient().get<CollectionWithBooks>(`/collections/${id}`);
  },

  create(data: CreateCollectionRequest): Promise<Collection> {
    return getApiClient().post<Collection>("/collections", data);
  },

  update(id: string, data: UpdateCollectionRequest): Promise<Collection> {
    return getApiClient().put<Collection>(`/collections/${id}`, data);
  },

  delete(id: string): Promise<void> {
    return getApiClient().delete<void>(`/collections/${id}`);
  },

  addBook(collectionId: string, bookId: string): Promise<void> {
    return getApiClient().post<void>(`/collections/${collectionId}/books`, {
      book_id: bookId,
    });
  },

  removeBook(collectionId: string, bookId: string): Promise<void> {
    return getApiClient().delete<void>(
      `/collections/${collectionId}/books/${bookId}`
    );
  },
};
