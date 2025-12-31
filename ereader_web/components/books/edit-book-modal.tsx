"use client";

import { useState } from "react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { Input, Textarea } from "@/components/ui/input";
import { useUpdateBook } from "@/lib/hooks/use-books";
import type { BookWithAssets } from "@/lib/api/types";

interface EditBookModalProps {
  book: BookWithAssets;
  isOpen: boolean;
  onClose: () => void;
}

export function EditBookModal({ book, isOpen, onClose }: EditBookModalProps) {
  const updateBook = useUpdateBook();

  const [formData, setFormData] = useState({
    title: book.title,
    authors: book.authors.join(", "),
    description: book.description || "",
    language: book.language || "",
    publisher: book.publisher || "",
    published_date: book.published_date || "",
    isbn: book.isbn || "",
    series_name: book.series_name || "",
    series_index: book.series_index?.toString() || "",
    tags: book.tags?.join(", ") || "",
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    await updateBook.mutateAsync({
      id: book.id,
      data: {
        title: formData.title,
        authors: formData.authors.split(",").map((a) => a.trim()).filter(Boolean),
        description: formData.description || undefined,
        language: formData.language || undefined,
        publisher: formData.publisher || undefined,
        published_date: formData.published_date || undefined,
        isbn: formData.isbn || undefined,
        series_name: formData.series_name || undefined,
        series_index: formData.series_index ? parseFloat(formData.series_index) : undefined,
        tags: formData.tags ? formData.tags.split(",").map((t) => t.trim()).filter(Boolean) : undefined,
      },
    });

    onClose();
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    setFormData({ ...formData, [e.target.name]: e.target.value });
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Edit Book Metadata" className="max-w-2xl">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="title" className="block text-sm font-medium mb-1">
            Title <span className="text-red-500">*</span>
          </label>
          <Input
            id="title"
            name="title"
            value={formData.title}
            onChange={handleChange}
            required
          />
        </div>

        <div>
          <label htmlFor="authors" className="block text-sm font-medium mb-1">
            Authors <span className="text-foreground/60 text-xs">(comma-separated)</span>
          </label>
          <Input
            id="authors"
            name="authors"
            value={formData.authors}
            onChange={handleChange}
            placeholder="Author 1, Author 2"
          />
        </div>

        <div>
          <label htmlFor="description" className="block text-sm font-medium mb-1">
            Description
          </label>
          <Textarea
            id="description"
            name="description"
            value={formData.description}
            onChange={handleChange}
            className="min-h-[120px]"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label htmlFor="publisher" className="block text-sm font-medium mb-1">
              Publisher
            </label>
            <Input
              id="publisher"
              name="publisher"
              value={formData.publisher}
              onChange={handleChange}
            />
          </div>

          <div>
            <label htmlFor="published_date" className="block text-sm font-medium mb-1">
              Published Date
            </label>
            <Input
              id="published_date"
              name="published_date"
              value={formData.published_date}
              onChange={handleChange}
              placeholder="YYYY-MM-DD"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label htmlFor="isbn" className="block text-sm font-medium mb-1">
              ISBN
            </label>
            <Input
              id="isbn"
              name="isbn"
              value={formData.isbn}
              onChange={handleChange}
            />
          </div>

          <div>
            <label htmlFor="language" className="block text-sm font-medium mb-1">
              Language
            </label>
            <Input
              id="language"
              name="language"
              value={formData.language}
              onChange={handleChange}
              placeholder="en"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label htmlFor="series_name" className="block text-sm font-medium mb-1">
              Series Name
            </label>
            <Input
              id="series_name"
              name="series_name"
              value={formData.series_name}
              onChange={handleChange}
            />
          </div>

          <div>
            <label htmlFor="series_index" className="block text-sm font-medium mb-1">
              Series Index
            </label>
            <Input
              id="series_index"
              name="series_index"
              type="number"
              step="0.1"
              value={formData.series_index}
              onChange={handleChange}
              placeholder="1"
            />
          </div>
        </div>

        <div>
          <label htmlFor="tags" className="block text-sm font-medium mb-1">
            Tags <span className="text-foreground/60 text-xs">(comma-separated)</span>
          </label>
          <Input
            id="tags"
            name="tags"
            value={formData.tags}
            onChange={handleChange}
            placeholder="fiction, sci-fi, adventure"
          />
        </div>

        <div className="flex gap-2 justify-end pt-4">
          <Button type="button" variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit" disabled={updateBook.isPending}>
            {updateBook.isPending ? "Saving..." : "Save Changes"}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
