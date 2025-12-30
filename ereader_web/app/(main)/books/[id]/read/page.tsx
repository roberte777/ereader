"use client";

import { useState, useEffect, useCallback, useMemo } from "react";
import { useParams, useRouter } from "next/navigation";
import dynamic from "next/dynamic";
import { Loader2 } from "lucide-react";
import { useBook } from "@/lib/hooks/use-books";
import {
  useReadingState,
  useDebouncedReadingStateUpdate,
} from "@/lib/hooks/use-reading-state";
import { useReaderStore } from "@/lib/store/reader-store";
import { booksApi } from "@/lib/api/books";
import type { ReadingLocation, BookFormat } from "@/lib/api/types";
import type { NavItem } from "@/components/reader/epub-reader";
import {
  ReaderToolbar,
  ReaderBottomBar,
  ReaderSettings,
  TableOfContents,
} from "@/components/reader";

const EpubReader = dynamic(
  () => import("@/components/reader/epub-reader").then((mod) => mod.EpubReader),
  {
    ssr: false,
    loading: () => (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-foreground/50" />
      </div>
    ),
  }
);

const PdfReader = dynamic(
  () => import("@/components/reader/pdf-reader").then((mod) => mod.PdfReader),
  {
    ssr: false,
    loading: () => (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-foreground/50" />
      </div>
    ),
  }
);

export default function ReadPage() {
  const params = useParams();
  const router = useRouter();
  const bookId = params.id as string;

  const { data: book, isLoading: bookLoading } = useBook(bookId);
  const { data: readingState } = useReadingState(bookId);
  const { updateLocation, flushUpdate } = useDebouncedReadingStateUpdate(bookId);

  const {
    showToc,
    setShowToc,
    showSettings,
    setShowSettings,
  } = useReaderStore();

  const [toc, setToc] = useState<NavItem[]>([]);
  const [currentLocation, setCurrentLocation] = useState<ReadingLocation | null>(
    null
  );
  const [isReaderReady, setIsReaderReady] = useState(false);

  const readerFormat = useMemo((): BookFormat | null => {
    if (!book?.formats) return null;
    if (book.formats.includes("epub")) return "epub";
    if (book.formats.includes("pdf")) return "pdf";
    return null;
  }, [book?.formats]);

  useEffect(() => {
    return () => {
      flushUpdate();
    };
  }, [flushUpdate]);

  const handleLocationChange = useCallback(
    (location: ReadingLocation) => {
      setCurrentLocation(location);
      updateLocation(location);
    },
    [updateLocation]
  );

  const handleTocLoaded = useCallback((items: NavItem[]) => {
    setToc(items);
  }, []);

  const handleNavigate = useCallback((href: string) => {
    setShowToc(false);
  }, [setShowToc]);

  if (bookLoading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-foreground/50" />
      </div>
    );
  }

  if (!book) {
    return (
      <div className="flex h-screen flex-col items-center justify-center">
        <p className="text-lg">Book not found</p>
        <button
          onClick={() => router.push("/library")}
          className="mt-4 text-foreground/70 hover:text-foreground"
        >
          Return to library
        </button>
      </div>
    );
  }

  if (!readerFormat) {
    return (
      <div className="flex h-screen flex-col items-center justify-center">
        <p className="text-lg">No readable format available</p>
        <p className="mt-2 text-sm text-foreground/60">
          This book doesn&apos;t have an EPUB or PDF file to read
        </p>
        <button
          onClick={() => router.push(`/books/${bookId}`)}
          className="mt-4 text-foreground/70 hover:text-foreground"
        >
          Return to book details
        </button>
      </div>
    );
  }

  const downloadUrl = booksApi.getDownloadUrl(bookId, readerFormat);

  const parseInitialPage = (): number => {
    if (!readingState?.location?.locator) return 1;
    const match = readingState.location.locator.match(/page:(\d+)/);
    return match ? parseInt(match[1]) : 1;
  };

  return (
    <div className="h-screen flex flex-col bg-background overflow-hidden">
      <ReaderToolbar
        bookId={bookId}
        title={book.title}
        chapter={currentLocation?.chapter}
        progress={currentLocation?.progress || 0}
        onToggleToc={() => setShowToc(!showToc)}
        onToggleSettings={() => setShowSettings(!showSettings)}
        showToc={showToc}
        showSettings={showSettings}
      />

      <div className="flex-1 relative mt-14">
        {readerFormat === "epub" ? (
          <EpubReader
            url={downloadUrl}
            initialLocation={readingState?.location?.locator}
            onLocationChange={handleLocationChange}
            onTocLoaded={handleTocLoaded}
            onReady={() => setIsReaderReady(true)}
          />
        ) : (
          <PdfReader
            url={downloadUrl}
            initialPage={parseInitialPage()}
            onLocationChange={handleLocationChange}
            onReady={() => setIsReaderReady(true)}
          />
        )}

        {showToc && readerFormat === "epub" && (
          <TableOfContents
            items={toc}
            currentLocation={currentLocation?.locator}
            onNavigate={handleNavigate}
            onClose={() => setShowToc(false)}
          />
        )}

        {showSettings && readerFormat === "epub" && (
          <ReaderSettings onClose={() => setShowSettings(false)} />
        )}
      </div>

      <ReaderBottomBar progress={currentLocation?.progress || 0} />
    </div>
  );
}
