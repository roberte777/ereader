"use client";

import { Grid, List, ArrowUpDown, SortAsc, SortDesc } from "lucide-react";
import { cn } from "@/lib/utils";
import { useLibraryStore } from "@/lib/store/library-store";
import { Select } from "@/components/ui/dropdown";

const sortOptions = [
  { value: "created_at", label: "Date Added" },
  { value: "updated_at", label: "Recently Updated" },
  { value: "title", label: "Title" },
  { value: "series_index", label: "Series Order" },
];

interface LibraryToolbarProps {
  totalBooks?: number;
}

export function LibraryToolbar({ totalBooks }: LibraryToolbarProps) {
  const { viewMode, setViewMode, sortBy, setSortBy, sortOrder, toggleSortOrder } =
    useLibraryStore();

  return (
    <div className="flex items-center justify-between gap-4 py-4">
      <div className="text-sm text-foreground/60">
        {totalBooks !== undefined && (
          <span>
            {totalBooks} {totalBooks === 1 ? "book" : "books"}
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        <Select
          value={sortBy}
          onChange={(value) => setSortBy(value as typeof sortBy)}
          options={sortOptions}
          className="w-40"
        />

        <button
          onClick={toggleSortOrder}
          className="flex h-10 items-center justify-center rounded-lg border border-foreground/20 px-3 hover:bg-foreground/5"
          title={sortOrder === "asc" ? "Ascending" : "Descending"}
        >
          {sortOrder === "asc" ? (
            <SortAsc className="h-4 w-4" />
          ) : (
            <SortDesc className="h-4 w-4" />
          )}
        </button>

        <div className="flex rounded-lg border border-foreground/20 overflow-hidden">
          <button
            onClick={() => setViewMode("grid")}
            className={cn(
              "flex h-10 w-10 items-center justify-center transition-colors",
              viewMode === "grid"
                ? "bg-foreground/10"
                : "hover:bg-foreground/5"
            )}
            title="Grid view"
          >
            <Grid className="h-4 w-4" />
          </button>
          <button
            onClick={() => setViewMode("list")}
            className={cn(
              "flex h-10 w-10 items-center justify-center transition-colors",
              viewMode === "list"
                ? "bg-foreground/10"
                : "hover:bg-foreground/5"
            )}
            title="List view"
          >
            <List className="h-4 w-4" />
          </button>
        </div>
      </div>
    </div>
  );
}
