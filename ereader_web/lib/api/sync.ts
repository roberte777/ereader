import { getApiClient } from "./client";
import type {
  ReadingState,
  UpdateReadingStateRequest,
  SyncRequest,
  SyncResponse,
} from "./types";

export const syncApi = {
  batch(data: SyncRequest): Promise<SyncResponse> {
    return getApiClient().post<SyncResponse>("/sync", data);
  },

  getReadingState(bookId: string): Promise<ReadingState> {
    return getApiClient().get<ReadingState>(`/sync/reading-state/${bookId}`);
  },

  updateReadingState(
    bookId: string,
    data: UpdateReadingStateRequest
  ): Promise<ReadingState> {
    return getApiClient().put<ReadingState>(
      `/sync/reading-state/${bookId}`,
      data
    );
  },
};
