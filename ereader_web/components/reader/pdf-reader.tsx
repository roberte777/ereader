"use client";

import { useState, useCallback, useEffect } from "react";
import { Document, Page, pdfjs } from "react-pdf";
import { ChevronLeft, ChevronRight, ZoomIn, ZoomOut } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ReadingLocation } from "@/lib/api/types";

import "react-pdf/dist/Page/AnnotationLayer.css";
import "react-pdf/dist/Page/TextLayer.css";

pdfjs.GlobalWorkerOptions.workerSrc = `//unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

interface PdfReaderProps {
  url: string;
  initialPage?: number;
  onLocationChange?: (location: ReadingLocation) => void;
  onReady?: () => void;
}

export function PdfReader({
  url,
  initialPage = 1,
  onLocationChange,
  onReady,
}: PdfReaderProps) {
  const [numPages, setNumPages] = useState<number>(0);
  const [pageNumber, setPageNumber] = useState(initialPage);
  const [scale, setScale] = useState(1.0);
  const [isLoading, setIsLoading] = useState(true);

  const onDocumentLoadSuccess = useCallback(
    ({ numPages }: { numPages: number }) => {
      setNumPages(numPages);
      setIsLoading(false);
      onReady?.();
    },
    [onReady]
  );

  const goToPage = useCallback(
    (page: number) => {
      const newPage = Math.max(1, Math.min(page, numPages));
      setPageNumber(newPage);
      onLocationChange?.({
        locator: `page:${newPage}`,
        progress: numPages > 0 ? newPage / numPages : 0,
        chapter: null,
      });
    },
    [numPages, onLocationChange]
  );

  const goNext = useCallback(() => {
    if (pageNumber < numPages) {
      goToPage(pageNumber + 1);
    }
  }, [pageNumber, numPages, goToPage]);

  const goPrev = useCallback(() => {
    if (pageNumber > 1) {
      goToPage(pageNumber - 1);
    }
  }, [pageNumber, goToPage]);

  const zoomIn = useCallback(() => {
    setScale((s) => Math.min(s + 0.2, 3.0));
  }, []);

  const zoomOut = useCallback(() => {
    setScale((s) => Math.max(s - 0.2, 0.5));
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === " ") {
        goNext();
      } else if (e.key === "ArrowLeft") {
        goPrev();
      } else if (e.key === "+" || e.key === "=") {
        zoomIn();
      } else if (e.key === "-") {
        zoomOut();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [goNext, goPrev, zoomIn, zoomOut]);

  return (
    <div className="relative flex h-full flex-col bg-gray-100 dark:bg-gray-900">
      <div className="flex items-center justify-between border-b border-foreground/10 bg-background px-4 py-2">
        <div className="flex items-center gap-2">
          <button
            onClick={goPrev}
            disabled={pageNumber <= 1}
            className="rounded-lg p-2 hover:bg-foreground/5 disabled:opacity-50"
          >
            <ChevronLeft className="h-5 w-5" />
          </button>
          <span className="text-sm">
            <input
              type="number"
              value={pageNumber}
              onChange={(e) => goToPage(parseInt(e.target.value) || 1)}
              className="w-12 rounded border border-foreground/20 bg-transparent px-2 py-1 text-center text-sm"
              min={1}
              max={numPages}
            />
            <span className="mx-2 text-foreground/60">of {numPages}</span>
          </span>
          <button
            onClick={goNext}
            disabled={pageNumber >= numPages}
            className="rounded-lg p-2 hover:bg-foreground/5 disabled:opacity-50"
          >
            <ChevronRight className="h-5 w-5" />
          </button>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={zoomOut}
            className="rounded-lg p-2 hover:bg-foreground/5"
          >
            <ZoomOut className="h-5 w-5" />
          </button>
          <span className="text-sm text-foreground/60">
            {Math.round(scale * 100)}%
          </span>
          <button
            onClick={zoomIn}
            className="rounded-lg p-2 hover:bg-foreground/5"
          >
            <ZoomIn className="h-5 w-5" />
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        <div className="flex min-h-full items-start justify-center p-4">
          <Document
            file={url}
            onLoadSuccess={onDocumentLoadSuccess}
            loading={
              <div className="flex h-64 items-center justify-center">
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-foreground" />
              </div>
            }
            error={
              <div className="text-center text-red-500">
                Failed to load PDF
              </div>
            }
          >
            <Page
              pageNumber={pageNumber}
              scale={scale}
              className="shadow-lg"
              loading={
                <div className="flex h-64 w-full items-center justify-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-foreground" />
                </div>
              }
            />
          </Document>
        </div>
      </div>

      <button
        onClick={goPrev}
        className="absolute left-0 top-14 bottom-0 w-1/4 cursor-pointer opacity-0"
        aria-label="Previous page"
      />
      <button
        onClick={goNext}
        className="absolute right-0 top-14 bottom-0 w-1/4 cursor-pointer opacity-0"
        aria-label="Next page"
      />
    </div>
  );
}
