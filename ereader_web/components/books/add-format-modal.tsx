"use client";

import { useState } from "react";
import { Modal } from "@/components/ui/modal";
import { Button } from "@/components/ui/button";
import { FileDropzone } from "@/components/upload/file-dropzone";
import { useUploadBookFile } from "@/lib/hooks/use-books";
import type { BookWithAssets, BookFormat } from "@/lib/api/types";

interface AddFormatModalProps {
  book: BookWithAssets;
  isOpen: boolean;
  onClose: () => void;
}

export function AddFormatModal({ book, isOpen, onClose }: AddFormatModalProps) {
  const uploadFile = useUploadBookFile();
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [error, setError] = useState<string>("");

  const handleUpload = async () => {
    if (selectedFiles.length === 0) {
      setError("Please select a file to upload");
      return;
    }

    const file = selectedFiles[0];

    // Check if this format already exists
    const fileExt = file.name.split('.').pop()?.toLowerCase() as BookFormat;
    if (book.formats.includes(fileExt)) {
      setError(`This book already has a ${fileExt.toUpperCase()} format`);
      return;
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
    <Modal isOpen={isOpen} onClose={handleClose} title="Add Format">
      <div className="space-y-4">
        {book.formats.length > 0 && (
          <div className="rounded-lg bg-foreground/5 p-3">
            <p className="text-sm font-medium mb-1">Current formats:</p>
            <p className="text-sm text-foreground/60">
              {book.formats.map((f) => f.toUpperCase()).join(", ")}
            </p>
          </div>
        )}

        <div>
          <p className="text-sm text-foreground/60 mb-3">
            Upload an additional format for this book (EPUB, PDF, CBZ, or MOBI)
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
