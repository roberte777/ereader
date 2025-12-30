"use client";

import Link from "next/link";
import { FolderOpen, MoreVertical } from "lucide-react";
import type { Collection } from "@/lib/api/types";
import { Dropdown, DropdownItem } from "@/components/ui/dropdown";

interface CollectionCardProps {
  collection: Collection;
  onEdit?: () => void;
  onDelete?: () => void;
}

export function CollectionCard({
  collection,
  onEdit,
  onDelete,
}: CollectionCardProps) {
  return (
    <div className="group relative rounded-xl border border-foreground/10 p-4 hover:bg-foreground/5 transition-colors">
      <div className="flex items-start justify-between">
        <Link href={`/collections/${collection.id}`} className="flex-1">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-foreground/5 p-2">
              <FolderOpen className="h-5 w-5" />
            </div>
            <div>
              <h3 className="font-medium">{collection.name}</h3>
              <p className="text-sm text-foreground/60">
                {collection.book_count}{" "}
                {collection.book_count === 1 ? "book" : "books"}
              </p>
            </div>
          </div>
        </Link>

        <Dropdown
          align="right"
          trigger={
            <button className="rounded p-1 opacity-0 transition-opacity hover:bg-foreground/10 group-hover:opacity-100">
              <MoreVertical className="h-4 w-4" />
            </button>
          }
        >
          {onEdit && <DropdownItem onClick={onEdit}>Edit</DropdownItem>}
          {onDelete && (
            <DropdownItem onClick={onDelete} className="text-red-500">
              Delete
            </DropdownItem>
          )}
        </Dropdown>
      </div>

      {collection.description && (
        <Link href={`/collections/${collection.id}`}>
          <p className="mt-3 text-sm text-foreground/60 line-clamp-2">
            {collection.description}
          </p>
        </Link>
      )}
    </div>
  );
}
