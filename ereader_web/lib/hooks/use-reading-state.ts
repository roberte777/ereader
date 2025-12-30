"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useRef } from "react";
import { syncApi } from "@/lib/api/sync";
import type { ReadingLocation } from "@/lib/api/types";

export function useReadingState(bookId: string) {
  return useQuery({
    queryKey: ["reading-state", bookId],
    queryFn: () => syncApi.getReadingState(bookId),
    enabled: !!bookId,
    retry: false,
  });
}

export function useUpdateReadingState(bookId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (location: ReadingLocation) =>
      syncApi.updateReadingState(bookId, { location }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["reading-state", bookId] });
    },
  });
}

export function useDebouncedReadingStateUpdate(bookId: string) {
  const updateMutation = useUpdateReadingState(bookId);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastLocationRef = useRef<ReadingLocation | null>(null);

  const updateLocation = useCallback(
    (location: ReadingLocation) => {
      lastLocationRef.current = location;

      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      timeoutRef.current = setTimeout(() => {
        if (lastLocationRef.current) {
          updateMutation.mutate(lastLocationRef.current);
        }
      }, 2000);
    },
    [updateMutation]
  );

  const flushUpdate = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    if (lastLocationRef.current) {
      updateMutation.mutate(lastLocationRef.current);
    }
  }, [updateMutation]);

  return { updateLocation, flushUpdate };
}
