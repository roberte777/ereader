"use client";

import { useState } from "react";
import Link from "next/link";
import Image from "next/image";
import { BookOpen, MoreVertical } from "lucide-react";
import { cn, formatAuthors } from "@/lib/utils";
import type { Book } from "@/lib/api/types";
import { booksApi } from "@/lib/api/books";
import { Dropdown, DropdownItem } from "@/components/ui/dropdown";

interface BookCardProps {
  book: Book;
  progress?: number;
  onDelete?: () => void;
}

export function BookCard({ book, progress, onDelete }: BookCardProps) {
  const [imageError, setImageError] = useState(false);
  const coverUrl = booksApi.getCoverUrl(book.id, "medium");

  return (
    <div className="group relative flex flex-col">
      <Link
        href={`/books/${book.id}`}
        className="relative aspect-[2/3] overflow-hidden rounded-lg bg-foreground/5"
      >
        {!imageError ? (
          <Image
            src={coverUrl}
            alt={book.title}
            fill
            className="object-cover transition-transform group-hover:scale-105"
            onError={() => setImageError(true)}
            sizes="(max-width: 640px) 50vw, (max-width: 1024px) 33vw, 20vw"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <BookOpen className="h-12 w-12 text-foreground/20" />
          </div>
        )}

        {progress !== undefined && progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-foreground/10">
            <div
              className="h-full bg-foreground/70 transition-all"
              style={{ width: `${progress * 100}%` }}
            />
          </div>
        )}
      </Link>

      <div className="mt-2 flex items-start justify-between gap-2">
        <Link href={`/books/${book.id}`} className="min-w-0 flex-1">
          <h3 className="truncate text-sm font-medium">{book.title}</h3>
          <p className="truncate text-xs text-foreground/60">
            {formatAuthors(book.authors)}
          </p>
        </Link>

        <Dropdown
          align="right"
          trigger={
            <button className="rounded p-1 opacity-0 transition-opacity hover:bg-foreground/5 group-hover:opacity-100">
              <MoreVertical className="h-4 w-4" />
            </button>
          }
        >
          <DropdownItem>
            <Link href={`/books/${book.id}/read`} className="block w-full">
              Read
            </Link>
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
    </div>
  );
}
