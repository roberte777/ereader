"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft, Loader2 } from "lucide-react";
import { useCreateBook, useUploadBookFile } from "@/lib/hooks/use-books";
import { FileDropzone } from "@/components/upload/file-dropzone";
import { Button } from "@/components/ui/button";
import { Input, Textarea } from "@/components/ui/input";

export default function UploadPage() {
  const router = useRouter();
  const [files, setFiles] = useState<File[]>([]);
  const [step, setStep] = useState<"select" | "metadata" | "uploading">("select");
  const [currentUpload, setCurrentUpload] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const [metadata, setMetadata] = useState({
    title: "",
    authors: "",
    description: "",
    tags: "",
  });

  const createBook = useCreateBook();
  const uploadFile = useUploadBookFile();

  const handleFilesSelected = (newFiles: File[]) => {
    setFiles(newFiles);
    if (newFiles.length === 1) {
      const fileName = newFiles[0].name.replace(/\.[^/.]+$/, "");
      setMetadata((prev) => ({ ...prev, title: fileName }));
    }
  };

  const handleRemoveFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleUpload = async () => {
    if (files.length === 0) return;

    setStep("uploading");
    setError(null);

    try {
      for (let i = 0; i < files.length; i++) {
        setCurrentUpload(i);
        const file = files[i];

        const title = files.length === 1
          ? metadata.title || file.name.replace(/\.[^/.]+$/, "")
          : file.name.replace(/\.[^/.]+$/, "");

        const authors = metadata.authors
          .split(",")
          .map((a) => a.trim())
          .filter(Boolean);

        const tags = metadata.tags
          .split(",")
          .map((t) => t.trim())
          .filter(Boolean);

        const book = await createBook.mutateAsync({
          title,
          authors: files.length === 1 ? authors : [],
          description: files.length === 1 ? metadata.description : undefined,
          tags: files.length === 1 ? tags : [],
        });

        await uploadFile.mutateAsync({
          bookId: book.id,
          file,
        });
      }

      router.push("/library");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
      setStep("metadata");
    }
  };

  return (
    <div className="p-4 md:p-6 max-w-2xl mx-auto">
      <Link
        href="/library"
        className="inline-flex items-center gap-2 text-sm text-foreground/70 hover:text-foreground mb-6"
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Library
      </Link>

      <h1 className="text-2xl font-bold mb-6">Upload Books</h1>

      {step === "select" && (
        <div className="space-y-6">
          <FileDropzone
            onFilesSelected={handleFilesSelected}
            selectedFiles={files}
            onRemoveFile={handleRemoveFile}
          />

          {files.length > 0 && (
            <div className="flex justify-end gap-3">
              <Button variant="secondary" onClick={() => setFiles([])}>
                Clear
              </Button>
              <Button onClick={() => setStep("metadata")}>
                Continue
              </Button>
            </div>
          )}
        </div>
      )}

      {step === "metadata" && (
        <div className="space-y-6">
          {files.length === 1 ? (
            <>
              <p className="text-sm text-foreground/60">
                Add metadata for your book (optional)
              </p>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-2">Title</label>
                  <Input
                    value={metadata.title}
                    onChange={(e) =>
                      setMetadata((prev) => ({ ...prev, title: e.target.value }))
                    }
                    placeholder="Book title"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">
                    Authors
                  </label>
                  <Input
                    value={metadata.authors}
                    onChange={(e) =>
                      setMetadata((prev) => ({ ...prev, authors: e.target.value }))
                    }
                    placeholder="Author names (comma separated)"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">
                    Description
                  </label>
                  <Textarea
                    value={metadata.description}
                    onChange={(e) =>
                      setMetadata((prev) => ({
                        ...prev,
                        description: e.target.value,
                      }))
                    }
                    placeholder="Book description"
                    rows={4}
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">Tags</label>
                  <Input
                    value={metadata.tags}
                    onChange={(e) =>
                      setMetadata((prev) => ({ ...prev, tags: e.target.value }))
                    }
                    placeholder="Tags (comma separated)"
                  />
                </div>
              </div>
            </>
          ) : (
            <p className="text-sm text-foreground/60">
              Ready to upload {files.length} books. Metadata will be extracted
              automatically from each file.
            </p>
          )}

          {error && (
            <div className="rounded-lg bg-red-500/10 border border-red-500/20 p-4 text-red-500 text-sm">
              {error}
            </div>
          )}

          <div className="flex justify-between gap-3">
            <Button variant="secondary" onClick={() => setStep("select")}>
              Back
            </Button>
            <Button onClick={handleUpload}>
              Upload {files.length} {files.length === 1 ? "Book" : "Books"}
            </Button>
          </div>
        </div>
      )}

      {step === "uploading" && (
        <div className="flex flex-col items-center justify-center py-12">
          <Loader2 className="h-10 w-10 animate-spin text-foreground/50" />
          <p className="mt-4 text-lg font-medium">Uploading...</p>
          <p className="text-sm text-foreground/60">
            {currentUpload + 1} of {files.length} files
          </p>
        </div>
      )}
    </div>
  );
}
