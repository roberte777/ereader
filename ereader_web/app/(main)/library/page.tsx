"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";
import { Plus } from "lucide-react";
import Link from "next/link";
import { useBooks, useSearchBooks, useDeleteBook } from "@/lib/hooks/use-books";
import { useLibraryStore } from "@/lib/store/library-store";
import { BookGrid, BookList, LibraryToolbar } from "@/components/library";
import { Button } from "@/components/ui/button";
import { BookGridSkeleton } from "@/components/ui/skeleton";

function LibraryContent() {
  const searchParams = useSearchParams();
  const searchQuery = searchParams.get("q") || "";

  const { viewMode, sortBy, sortOrder, filters } = useLibraryStore();

  const booksQuery = useBooks({
    sort_by: sortBy,
    sort_order: sortOrder,
    tag: filters.tags[0],
    series: filters.series || undefined,
    author: filters.author || undefined,
  });

  const searchBooksQuery = useSearchBooks({
    q: searchQuery,
  });

  const deleteBook = useDeleteBook();

  const isSearching = !!searchQuery;
  const activeQuery = isSearching ? searchBooksQuery : booksQuery;
  const books = activeQuery.data?.items || [];
  const totalBooks = activeQuery.data?.total || 0;

  const handleDeleteBook = (id: string) => {
    if (confirm("Are you sure you want to delete this book?")) {
      deleteBook.mutate(id);
    }
  };

  return (
    <div className="p-4 md:p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">
            {isSearching ? `Search: "${searchQuery}"` : "Library"}
          </h1>
          {isSearching && (
            <p className="text-sm text-foreground/60">
              {totalBooks} {totalBooks === 1 ? "result" : "results"} found
            </p>
          )}
        </div>
        <Link href="/books/upload">
          <Button size="sm">
            <Plus className="mr-2 h-4 w-4" />
            Add Book
          </Button>
        </Link>
      </div>

      {!isSearching && <LibraryToolbar totalBooks={totalBooks} />}

      {viewMode === "grid" ? (
        <BookGrid
          books={books}
          isLoading={activeQuery.isLoading}
          onDeleteBook={handleDeleteBook}
        />
      ) : (
        <BookList
          books={books}
          isLoading={activeQuery.isLoading}
          onDeleteBook={handleDeleteBook}
        />
      )}
    </div>
  );
}

export default function LibraryPage() {
  return (
    <Suspense fallback={<BookGridSkeleton count={12} />}>
      <LibraryContent />
    </Suspense>
  );
}
