"use client";

import { useCallback } from "react";
import { useDropzone } from "react-dropzone";
import { Upload, File, X } from "lucide-react";
import { cn, formatFileSize } from "@/lib/utils";

const ACCEPTED_FORMATS = {
  "application/epub+zip": [".epub"],
  "application/pdf": [".pdf"],
  "application/vnd.comicbook+zip": [".cbz"],
  "application/x-mobipocket-ebook": [".mobi", ".azw3"],
};

interface FileDropzoneProps {
  onFilesSelected: (files: File[]) => void;
  selectedFiles: File[];
  onRemoveFile: (index: number) => void;
  maxFiles?: number;
  disabled?: boolean;
}

export function FileDropzone({
  onFilesSelected,
  selectedFiles,
  onRemoveFile,
  maxFiles = 10,
  disabled = false,
}: FileDropzoneProps) {
  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      const remaining = maxFiles - selectedFiles.length;
      const newFiles = acceptedFiles.slice(0, remaining);
      onFilesSelected([...selectedFiles, ...newFiles]);
    },
    [onFilesSelected, selectedFiles, maxFiles]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: ACCEPTED_FORMATS,
    maxFiles: maxFiles - selectedFiles.length,
    disabled: disabled || selectedFiles.length >= maxFiles,
  });

  return (
    <div className="space-y-4">
      <div
        {...getRootProps()}
        className={cn(
          "flex flex-col items-center justify-center rounded-xl border-2 border-dashed p-8 transition-colors cursor-pointer",
          isDragActive
            ? "border-foreground/50 bg-foreground/5"
            : "border-foreground/20 hover:border-foreground/30 hover:bg-foreground/5",
          disabled && "opacity-50 cursor-not-allowed"
        )}
      >
        <input {...getInputProps()} />
        <Upload className="h-10 w-10 text-foreground/40 mb-4" />
        <p className="text-lg font-medium">
          {isDragActive ? "Drop files here" : "Drag & drop ebook files"}
        </p>
        <p className="mt-2 text-sm text-foreground/60">
          or click to browse
        </p>
        <p className="mt-4 text-xs text-foreground/40">
          Supports EPUB, PDF, CBZ, MOBI (max {maxFiles} files)
        </p>
      </div>

      {selectedFiles.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-medium">Selected Files</h3>
          {selectedFiles.map((file, index) => (
            <div
              key={`${file.name}-${index}`}
              className="flex items-center justify-between rounded-lg border border-foreground/10 p-3"
            >
              <div className="flex items-center gap-3">
                <File className="h-5 w-5 text-foreground/60" />
                <div>
                  <p className="text-sm font-medium">{file.name}</p>
                  <p className="text-xs text-foreground/60">
                    {formatFileSize(file.size)}
                  </p>
                </div>
              </div>
              <button
                onClick={() => onRemoveFile(index)}
                className="rounded p-1 hover:bg-foreground/10"
                disabled={disabled}
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
