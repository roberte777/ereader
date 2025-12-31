"use client";

import { useState } from "react";
import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import {
  BookOpen,
  ArrowLeft,
  Edit,
  Trash2,
  Download,
  FolderPlus,
  Upload,
} from "lucide-react";
import { useBook, useDeleteBook } from "@/lib/hooks/use-books";
import { booksApi } from "@/lib/api/books";
import { formatAuthors, formatDate, formatFileSize } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { EditBookModal } from "@/components/books/edit-book-modal";
import { AddToCollectionModal } from "@/components/books/add-to-collection-modal";
import { AddFormatModal } from "@/components/books/add-format-modal";

export default function BookDetailPage() {
  const params = useParams();
  const router = useRouter();
  const bookId = params.id as string;

  const { data: book, isLoading } = useBook(bookId);
  const deleteBook = useDeleteBook();

  const [imageError, setImageError] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showCollectionModal, setShowCollectionModal] = useState(false);
  const [showUploadModal, setShowUploadModal] = useState(false);

  const handleDelete = async () => {
    if (confirm("Are you sure you want to delete this book?")) {
      await deleteBook.mutateAsync(bookId);
      router.push("/library");
    }
  };

  if (isLoading) {
    return (
      <div className="p-4 md:p-6">
        <div className="flex gap-8">
          <Skeleton className="h-80 w-56 rounded-lg" />
          <div className="flex-1 space-y-4">
            <Skeleton className="h-8 w-1/2" />
            <Skeleton className="h-6 w-1/3" />
            <Skeleton className="h-24 w-full" />
          </div>
        </div>
      </div>
    );
  }

  if (!book) {
    return (
      <div className="flex flex-col items-center justify-center p-8">
        <p className="text-lg">Book not found</p>
        <Link href="/library" className="mt-4 text-foreground/70 hover:text-foreground">
          Return to library
        </Link>
      </div>
    );
  }

  const coverUrl = booksApi.getCoverUrl(book.id, "large");

  return (
    <div className="p-4 md:p-6">
      <Link
        href="/library"
        className="inline-flex items-center gap-2 text-sm text-foreground/70 hover:text-foreground mb-6"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Library
      </Link>

      <div className="flex flex-col gap-8 lg:flex-row">
        <div className="flex-shrink-0">
          <div className="relative aspect-[2/3] w-56 overflow-hidden rounded-lg bg-foreground/5 shadow-lg">
            {!imageError ? (
              <img
                src={coverUrl}
                alt={book.title}
                className="h-full w-full object-cover"
                onError={() => setImageError(true)}
              />
            ) : (
              <div className="flex h-full w-full items-center justify-center">
                <BookOpen className="h-16 w-16 text-foreground/20" />
              </div>
            )}
          </div>

          <div className="mt-4 space-y-2">
            {book.has_file ? (
              <a
                href={booksApi.getDownloadUrl(book.id)}
                download
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button variant="secondary" className="w-full">
                  <Download className="mr-2 h-4 w-4" />
                  Download EPUB
                </Button>
              </a>
            ) : (
              <div className="rounded-lg bg-foreground/5 p-4 text-center text-sm text-foreground/60">
                No file uploaded
              </div>
            )}

            <Button variant="secondary" className="w-full" onClick={() => setShowUploadModal(true)}>
              <Upload className="mr-2 h-4 w-4" />
              {book.has_file ? "Replace File" : "Upload File"}
            </Button>
          </div>
        </div>

        <div className="flex-1 space-y-6">
          <div>
            <h1 className="text-3xl font-bold">{book.title}</h1>
            <p className="mt-2 text-lg text-foreground/70">
              by {formatAuthors(book.authors)}
            </p>
          </div>

          {book.series_name && (
            <div className="inline-flex items-center rounded-full bg-foreground/5 px-3 py-1 text-sm">
              {book.series_name}
              {book.series_index && ` #${book.series_index}`}
            </div>
          )}

          {book.description && (
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <p className="text-foreground/80 whitespace-pre-wrap">
                {book.description}
              </p>
            </div>
          )}

          {book.tags && book.tags.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {book.tags.map((tag) => (
                <span
                  key={tag}
                  className="rounded-full bg-foreground/10 px-3 py-1 text-sm"
                >
                  {tag}
                </span>
              ))}
            </div>
          )}

          <div className="border-t border-foreground/10 pt-6">
            <h2 className="text-sm font-medium text-foreground/60 mb-4">Details</h2>
            <dl className="grid grid-cols-2 gap-4 text-sm">
              {book.publisher && (
                <>
                  <dt className="text-foreground/60">Publisher</dt>
                  <dd>{book.publisher}</dd>
                </>
              )}
              {book.published_date && (
                <>
                  <dt className="text-foreground/60">Published</dt>
                  <dd>{book.published_date}</dd>
                </>
              )}
              {book.isbn && (
                <>
                  <dt className="text-foreground/60">ISBN</dt>
                  <dd>{book.isbn}</dd>
                </>
              )}
              {book.language && (
                <>
                  <dt className="text-foreground/60">Language</dt>
                  <dd>{book.language}</dd>
                </>
              )}
              <dt className="text-foreground/60">Added</dt>
              <dd>{formatDate(book.created_at)}</dd>
              {book.has_file && book.format && (
                <>
                  <dt className="text-foreground/60">Format</dt>
                  <dd className="uppercase">{book.format}</dd>
                </>
              )}
              {book.has_file && book.file_size && (
                <>
                  <dt className="text-foreground/60">File Size</dt>
                  <dd>{formatFileSize(book.file_size)}</dd>
                </>
              )}
            </dl>
          </div>

          <div className="flex gap-2 border-t border-foreground/10 pt-6">
            <Button variant="ghost" size="sm" onClick={() => setShowCollectionModal(true)}>
              <FolderPlus className="mr-2 h-4 w-4" />
              Add to Collection
            </Button>
            <Button variant="ghost" size="sm" onClick={() => setShowEditModal(true)}>
              <Edit className="mr-2 h-4 w-4" />
              Edit
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDelete}
              className="text-red-500 hover:text-red-600 hover:bg-red-500/10"
            >
              <Trash2 className="mr-2 h-4 w-4" />
              Delete
            </Button>
          </div>
        </div>
      </div>

      {/* Modals */}
      {showEditModal && (
        <EditBookModal
          book={book}
          isOpen={showEditModal}
          onClose={() => setShowEditModal(false)}
        />
      )}
      {showCollectionModal && (
        <AddToCollectionModal
          bookId={book.id}
          isOpen={showCollectionModal}
          onClose={() => setShowCollectionModal(false)}
        />
      )}
      {showUploadModal && (
        <AddFormatModal
          book={book}
          isOpen={showUploadModal}
          onClose={() => setShowUploadModal(false)}
        />
      )}
    </div>
  );
}
