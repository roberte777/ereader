"use client";

import { useState } from "react";
import { Plus } from "lucide-react";
import { useCollections, useDeleteCollection } from "@/lib/hooks/use-collections";
import { CollectionCard, CreateCollectionModal } from "@/components/collections";
import { Button } from "@/components/ui/button";
import { CollectionCardSkeleton } from "@/components/ui/skeleton";

export default function CollectionsPage() {
  const [showCreateModal, setShowCreateModal] = useState(false);
  const { data, isLoading } = useCollections();
  const deleteCollection = useDeleteCollection();

  const collections = data?.items || [];

  const handleDelete = (id: string) => {
    if (confirm("Are you sure you want to delete this collection?")) {
      deleteCollection.mutate(id);
    }
  };

  return (
    <div className="p-4 md:p-6">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-bold">Collections</h1>
        <Button size="sm" onClick={() => setShowCreateModal(true)}>
          <Plus className="mr-2 h-4 w-4" />
          New Collection
        </Button>
      </div>

      {isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 6 }).map((_, i) => (
            <CollectionCardSkeleton key={i} />
          ))}
        </div>
      ) : collections.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <p className="text-lg font-medium">No collections yet</p>
          <p className="mt-1 text-sm text-foreground/60">
            Create a collection to organize your books
          </p>
          <Button className="mt-4" onClick={() => setShowCreateModal(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Create Collection
          </Button>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {collections.map((collection) => (
            <CollectionCard
              key={collection.id}
              collection={collection}
              onDelete={() => handleDelete(collection.id)}
            />
          ))}
        </div>
      )}

      <CreateCollectionModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
      />
    </div>
  );
}
