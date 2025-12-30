"use client";

import { BookCard } from "./book-card";
import { BookGridSkeleton } from "@/components/ui/skeleton";
import type { Book } from "@/lib/api/types";

interface BookGridProps {
  books: Book[];
  isLoading?: boolean;
  onDeleteBook?: (id: string) => void;
}

export function BookGrid({ books, isLoading, onDeleteBook }: BookGridProps) {
  if (isLoading) {
    return <BookGridSkeleton count={12} />;
  }

  if (books.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <p className="text-lg font-medium">No books found</p>
        <p className="mt-1 text-sm text-foreground/60">
          Upload your first book to get started
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
      {books.map((book) => (
        <BookCard
          key={book.id}
          book={book}
          onDelete={onDeleteBook ? () => onDeleteBook(book.id) : undefined}
        />
      ))}
    </div>
  );
}
