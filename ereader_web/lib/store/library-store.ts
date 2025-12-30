import { create } from "zustand";
import { persist } from "zustand/middleware";

type ViewMode = "grid" | "list";
type SortBy = "title" | "created_at" | "updated_at" | "series_index";
type SortOrder = "asc" | "desc";

interface LibraryFilters {
  tags: string[];
  series: string | null;
  author: string | null;
}

interface LibraryState {
  viewMode: ViewMode;
  sortBy: SortBy;
  sortOrder: SortOrder;
  filters: LibraryFilters;
  searchQuery: string;

  setViewMode: (mode: ViewMode) => void;
  setSortBy: (sortBy: SortBy) => void;
  toggleSortOrder: () => void;
  setFilters: (filters: Partial<LibraryFilters>) => void;
  clearFilters: () => void;
  setSearchQuery: (query: string) => void;
}

const defaultFilters: LibraryFilters = {
  tags: [],
  series: null,
  author: null,
};

export const useLibraryStore = create<LibraryState>()(
  persist(
    (set) => ({
      viewMode: "grid",
      sortBy: "created_at",
      sortOrder: "desc",
      filters: defaultFilters,
      searchQuery: "",

      setViewMode: (viewMode) => set({ viewMode }),
      setSortBy: (sortBy) => set({ sortBy }),
      toggleSortOrder: () =>
        set((state) => ({
          sortOrder: state.sortOrder === "asc" ? "desc" : "asc",
        })),
      setFilters: (filters) =>
        set((state) => ({
          filters: { ...state.filters, ...filters },
        })),
      clearFilters: () => set({ filters: defaultFilters }),
      setSearchQuery: (searchQuery) => set({ searchQuery }),
    }),
    {
      name: "ereader-library-preferences",
      partialize: (state) => ({
        viewMode: state.viewMode,
        sortBy: state.sortBy,
        sortOrder: state.sortOrder,
      }),
    }
  )
);
