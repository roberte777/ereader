"use client";

import { useState } from "react";
import Link from "next/link";
import { BookOpen, MoreVertical } from "lucide-react";
import { cn, formatAuthors, formatDate } from "@/lib/utils";
import type { Book } from "@/lib/api/types";
import { booksApi } from "@/lib/api/books";
import { Dropdown, DropdownItem } from "@/components/ui/dropdown";
import { Skeleton } from "@/components/ui/skeleton";

interface BookListProps {
  books: Book[];
  isLoading?: boolean;
  onDeleteBook?: (id: string) => void;
}

export function BookList({ books, isLoading, onDeleteBook }: BookListProps) {
  if (isLoading) {
    return (
      <div className="space-y-2">
        {Array.from({ length: 10 }).map((_, i) => (
          <div key={i} className="flex items-center gap-4 rounded-lg border border-foreground/10 p-3">
            <Skeleton className="h-16 w-12 rounded" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-1/3" />
              <Skeleton className="h-3 w-1/4" />
            </div>
          </div>
        ))}
      </div>
    );
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
    <div className="space-y-2">
      {books.map((book) => (
        <BookListItem
          key={book.id}
          book={book}
          onDelete={onDeleteBook ? () => onDeleteBook(book.id) : undefined}
        />
      ))}
    </div>
  );
}

interface BookListItemProps {
  book: Book;
  progress?: number;
  onDelete?: () => void;
}

function BookListItem({ book, progress, onDelete }: BookListItemProps) {
  const [imageError, setImageError] = useState(false);
  const coverUrl = booksApi.getCoverUrl(book.id, "small");

  return (
    <div className="group flex items-center gap-4 rounded-lg border border-foreground/10 p-3 hover:bg-foreground/5 transition-colors">
      <Link
        href={`/books/${book.id}`}
        className="relative h-16 w-12 flex-shrink-0 overflow-hidden rounded bg-foreground/5"
      >
        {!imageError ? (
          <img
            src={coverUrl}
            alt={book.title}
            className="h-full w-full object-cover"
            onError={() => setImageError(true)}
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <BookOpen className="h-6 w-6 text-foreground/20" />
          </div>
        )}
      </Link>

      <Link href={`/books/${book.id}`} className="flex-1 min-w-0">
        <h3 className="truncate font-medium">{book.title}</h3>
        <p className="truncate text-sm text-foreground/60">
          {formatAuthors(book.authors)}
        </p>
      </Link>

      <div className="hidden sm:block text-sm text-foreground/60">
        {formatDate(book.created_at)}
      </div>

      {book.series_name && (
        <div className="hidden md:block text-sm text-foreground/60">
          {book.series_name}
          {book.series_index && ` #${book.series_index}`}
        </div>
      )}

      {progress !== undefined && progress > 0 && (
        <div className="hidden sm:block w-24">
          <div className="h-1.5 rounded-full bg-foreground/10">
            <div
              className="h-full rounded-full bg-foreground/70"
              style={{ width: `${progress * 100}%` }}
            />
          </div>
        </div>
      )}

      <Dropdown
        align="right"
        trigger={
          <button className="rounded p-1 opacity-0 transition-opacity hover:bg-foreground/10 group-hover:opacity-100">
            <MoreVertical className="h-4 w-4" />
          </button>
        }
      >
        <DropdownItem>
          <a
            href={booksApi.getDownloadUrl(book.id)}
            target="_blank"
            rel="noopener noreferrer"
            className="block w-full"
          >
            Download
          </a>
        </DropdownItem>
        <DropdownItem>
          <Link href={`/books/${book.id}`} className="block w-full">
            View Details
          </Link>
        </DropdownItem>
        {onDelete && (
          <DropdownItem onClick={onDelete} className="text-red-500">
            Delete
          </DropdownItem>
        )}
      </Dropdown>
    </div>
  );
}
