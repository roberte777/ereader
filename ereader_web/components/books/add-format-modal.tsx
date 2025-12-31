"use client";

import { useState } from "react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { FileDropzone } from "@/components/upload/file-dropzone";
import { useUploadBookFile } from "@/lib/hooks/use-books";
import type { Book } from "@/lib/api/types";

interface UploadFileModalProps {
  book: Book;
  isOpen: boolean;
  onClose: () => void;
}

export function UploadFileModal({ book, isOpen, onClose }: UploadFileModalProps) {
  const uploadFile = useUploadBookFile();
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [error, setError] = useState<string>("");

  const handleUpload = async () => {
    if (selectedFiles.length === 0) {
      setError("Please select a file to upload");
      return;
    }

    const file = selectedFiles[0];

    // Check if file is EPUB
    if (!file.name.toLowerCase().endsWith('.epub')) {
      setError("Only EPUB files are supported");
      return;
    }

    // Warn if book already has a file
    if (book.has_file) {
      if (!confirm("This book already has a file. Uploading will replace it. Continue?")) {
        return;
      }
    }

    try {
      await uploadFile.mutateAsync({ bookId: book.id, file });
      setSelectedFiles([]);
      setError("");
      onClose();
    } catch (err) {
      setError("Failed to upload file. Please try again.");
    }
  };

  const handleClose = () => {
    setSelectedFiles([]);
    setError("");
    onClose();
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Upload EPUB File">
      <div className="space-y-4">
        {book.has_file && (
          <div className="rounded-lg bg-amber-500/10 border border-amber-500/20 p-3">
            <p className="text-sm text-amber-600 dark:text-amber-400">
              This book already has an EPUB file. Uploading a new file will replace it.
            </p>
          </div>
        )}

        <div>
          <p className="text-sm text-foreground/60 mb-3">
            Upload an EPUB file for this book
          </p>
          <FileDropzone
            onFilesSelected={setSelectedFiles}
            selectedFiles={selectedFiles}
            onRemoveFile={(index) =>
              setSelectedFiles(selectedFiles.filter((_, i) => i !== index))
            }
            maxFiles={1}
            disabled={uploadFile.isPending}
          />
        </div>

        {error && (
          <div className="rounded-lg bg-red-500/10 border border-red-500/20 p-3">
            <p className="text-sm text-red-500">{error}</p>
          </div>
        )}

        <div className="flex gap-2 justify-end">
          <Button variant="ghost" onClick={handleClose} disabled={uploadFile.isPending}>
            Cancel
          </Button>
          <Button
            onClick={handleUpload}
            disabled={uploadFile.isPending || selectedFiles.length === 0}
          >
            {uploadFile.isPending ? "Uploading..." : "Upload"}
          </Button>
        </div>
      </div>
    </Modal>
  );
}

// Keep the old export name for backwards compatibility
export { UploadFileModal as AddFormatModal };
