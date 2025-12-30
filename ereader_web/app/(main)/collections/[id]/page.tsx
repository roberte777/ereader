"use client";

import { useParams, useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Trash2, Edit } from "lucide-react";
import {
  useCollection,
  useDeleteCollection,
  useRemoveBookFromCollection,
} from "@/lib/hooks/use-collections";
import { useLibraryStore } from "@/lib/store/library-store";
import { BookGrid, BookList } from "@/components/library";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

export default function CollectionDetailPage() {
  const params = useParams();
  const router = useRouter();
  const collectionId = params.id as string;

  const { viewMode } = useLibraryStore();
  const { data: collection, isLoading } = useCollection(collectionId);
  const deleteCollection = useDeleteCollection();
  const removeBook = useRemoveBookFromCollection();

  const handleDeleteCollection = async () => {
    if (confirm("Are you sure you want to delete this collection?")) {
      await deleteCollection.mutateAsync(collectionId);
      router.push("/collections");
    }
  };

  const handleRemoveBook = (bookId: string) => {
    if (confirm("Remove this book from the collection?")) {
      removeBook.mutate({ collectionId, bookId });
    }
  };

  if (isLoading) {
    return (
      <div className="p-4 md:p-6">
        <Skeleton className="h-6 w-32 mb-6" />
        <Skeleton className="h-10 w-64 mb-2" />
        <Skeleton className="h-5 w-48 mb-8" />
      </div>
    );
  }

  if (!collection) {
    return (
      <div className="flex flex-col items-center justify-center p-8">
        <p className="text-lg">Collection not found</p>
        <Link
          href="/collections"
          className="mt-4 text-foreground/70 hover:text-foreground"
        >
          Return to collections
        </Link>
      </div>
    );
  }

  return (
    <div className="p-4 md:p-6">
      <Link
        href="/collections"
        className="inline-flex items-center gap-2 text-sm text-foreground/70 hover:text-foreground mb-6"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Collections
      </Link>

      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold">{collection.name}</h1>
          {collection.description && (
            <p className="mt-1 text-foreground/60">{collection.description}</p>
          )}
          <p className="mt-2 text-sm text-foreground/50">
            {collection.book_count}{" "}
            {collection.book_count === 1 ? "book" : "books"}
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm">
            <Edit className="mr-2 h-4 w-4" />
            Edit
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleDeleteCollection}
            className="text-red-500 hover:text-red-600 hover:bg-red-500/10"
          >
            <Trash2 className="mr-2 h-4 w-4" />
            Delete
          </Button>
        </div>
      </div>

      {collection.books && collection.books.length > 0 ? (
        viewMode === "grid" ? (
          <BookGrid
            books={collection.books}
            onDeleteBook={handleRemoveBook}
          />
        ) : (
          <BookList
            books={collection.books}
            onDeleteBook={handleRemoveBook}
          />
        )
      ) : (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-lg font-medium">No books in this collection</p>
          <p className="mt-1 text-sm text-foreground/60">
            Add books from your library to this collection
          </p>
        </div>
      )}
    </div>
  );
}
