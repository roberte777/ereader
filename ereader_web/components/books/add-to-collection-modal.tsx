"use client";

import { useState } from "react";
import { Plus } from "lucide-react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  useCollections,
  useAddBookToCollection,
  useCreateCollection,
} from "@/lib/hooks/use-collections";

interface AddToCollectionModalProps {
  bookId: string;
  isOpen: boolean;
  onClose: () => void;
}

export function AddToCollectionModal({
  bookId,
  isOpen,
  onClose,
}: AddToCollectionModalProps) {
  const { data: collectionsData, isLoading } = useCollections();
  const addToCollection = useAddBookToCollection();
  const createCollection = useCreateCollection();

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newCollectionName, setNewCollectionName] = useState("");

  const collections = collectionsData?.items || [];

  const handleAddToCollection = async (collectionId: string) => {
    await addToCollection.mutateAsync({ collectionId, bookId });
    onClose();
  };

  const handleCreateAndAdd = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!newCollectionName.trim()) return;

    const newCollection = await createCollection.mutateAsync({
      name: newCollectionName.trim(),
      collection_type: "shelf",
    });

    await addToCollection.mutateAsync({
      collectionId: newCollection.id,
      bookId,
    });

    setNewCollectionName("");
    setShowCreateForm(false);
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Add to Collection">
      {isLoading ? (
        <div className="text-center py-8 text-foreground/60">Loading collections...</div>
      ) : (
        <div className="space-y-4">
          {collections.length === 0 && !showCreateForm ? (
            <div className="text-center py-8">
              <p className="text-foreground/60 mb-4">No collections yet</p>
              <Button onClick={() => setShowCreateForm(true)}>
                <Plus className="mr-2 h-4 w-4" />
                Create Collection
              </Button>
            </div>
          ) : (
            <>
              {!showCreateForm && (
                <>
                  <div className="space-y-2 max-h-64 overflow-y-auto">
                    {collections.map((collection) => (
                      <button
                        key={collection.id}
                        onClick={() => handleAddToCollection(collection.id)}
                        disabled={addToCollection.isPending}
                        className="w-full flex items-center justify-between p-3 rounded-lg border border-foreground/10 hover:bg-foreground/5 transition-colors text-left disabled:opacity-50"
                      >
                        <div>
                          <div className="font-medium">{collection.name}</div>
                          {collection.description && (
                            <div className="text-sm text-foreground/60">
                              {collection.description}
                            </div>
                          )}
                        </div>
                        <div className="text-sm text-foreground/60">
                          {collection.book_count} books
                        </div>
                      </button>
                    ))}
                  </div>

                  <Button
                    variant="ghost"
                    onClick={() => setShowCreateForm(true)}
                    className="w-full"
                  >
                    <Plus className="mr-2 h-4 w-4" />
                    Create New Collection
                  </Button>
                </>
              )}

              {showCreateForm && (
                <form onSubmit={handleCreateAndAdd} className="space-y-4">
                  <div>
                    <label htmlFor="collection-name" className="block text-sm font-medium mb-1">
                      Collection Name
                    </label>
                    <Input
                      id="collection-name"
                      value={newCollectionName}
                      onChange={(e) => setNewCollectionName(e.target.value)}
                      placeholder="My Collection"
                      required
                      autoFocus
                    />
                  </div>

                  <div className="flex gap-2 justify-end">
                    <Button
                      type="button"
                      variant="ghost"
                      onClick={() => {
                        setShowCreateForm(false);
                        setNewCollectionName("");
                      }}
                    >
                      Back
                    </Button>
                    <Button type="submit" disabled={createCollection.isPending}>
                      {createCollection.isPending ? "Creating..." : "Create & Add"}
                    </Button>
                  </div>
                </form>
              )}
            </>
          )}
        </div>
      )}
    </Modal>
  );
}
