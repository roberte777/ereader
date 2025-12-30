"use client";

import { useEffect, useRef, useCallback, useState } from "react";
import type { Book, Rendition, NavItem } from "epubjs";
import { useReaderStore, themeStyles } from "@/lib/store/reader-store";
import type { ReadingLocation } from "@/lib/api/types";

interface EpubReaderProps {
  url: string;
  initialLocation?: string;
  onLocationChange?: (location: ReadingLocation) => void;
  onTocLoaded?: (toc: NavItem[]) => void;
  onReady?: () => void;
}

export function EpubReader({
  url,
  initialLocation,
  onLocationChange,
  onTocLoaded,
  onReady,
}: EpubReaderProps) {
  const viewerRef = useRef<HTMLDivElement>(null);
  const bookRef = useRef<Book | null>(null);
  const renditionRef = useRef<Rendition | null>(null);
  const [isReady, setIsReady] = useState(false);

  const { fontSize, fontFamily, theme, lineHeight, margins } = useReaderStore();

  const applyStyles = useCallback(() => {
    if (!renditionRef.current) return;

    const themeColors = themeStyles[theme];

    renditionRef.current.themes.default({
      body: {
        "font-size": `${fontSize}px !important`,
        "font-family": `${fontFamily} !important`,
        "line-height": `${lineHeight} !important`,
        "background-color": `${themeColors.background} !important`,
        color: `${themeColors.text} !important`,
        padding: `0 ${margins}px !important`,
      },
      p: {
        "font-size": `${fontSize}px !important`,
        "line-height": `${lineHeight} !important`,
      },
      a: {
        color: "inherit !important",
      },
    });
  }, [fontSize, fontFamily, theme, lineHeight, margins]);

  useEffect(() => {
    if (!viewerRef.current || bookRef.current) return;

    const initBook = async () => {
      const ePub = (await import("epubjs")).default;
      const book = ePub(url);
      bookRef.current = book;

      const rendition = book.renderTo(viewerRef.current!, {
        width: "100%",
        height: "100%",
        spread: "none",
        flow: "paginated",
      });

      renditionRef.current = rendition;

      book.ready.then(() => {
        book.navigation.toc && onTocLoaded?.(book.navigation.toc);
      });

      rendition.on("relocated", (location: { start: { cfi: string; percentage: number; href: string } }) => {
        const chapter = book.navigation.get(location.start.href);
        onLocationChange?.({
          locator: location.start.cfi,
          progress: location.start.percentage,
          chapter: chapter?.label || null,
        });
      });

      rendition.on("rendered", () => {
        if (!isReady) {
          setIsReady(true);
          onReady?.();
        }
      });

      applyStyles();

      if (initialLocation) {
        rendition.display(initialLocation);
      } else {
        rendition.display();
      }
    };

    initBook();

    return () => {
      if (bookRef.current) {
        bookRef.current.destroy();
        bookRef.current = null;
        renditionRef.current = null;
      }
    };
  }, [url]);

  useEffect(() => {
    if (isReady) {
      applyStyles();
    }
  }, [isReady, applyStyles]);

  const goNext = useCallback(() => {
    renditionRef.current?.next();
  }, []);

  const goPrev = useCallback(() => {
    renditionRef.current?.prev();
  }, []);

  const goTo = useCallback((target: string) => {
    renditionRef.current?.display(target);
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === " ") {
        goNext();
      } else if (e.key === "ArrowLeft") {
        goPrev();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [goNext, goPrev]);

  return (
    <div className="relative h-full w-full">
      <div
        ref={viewerRef}
        className="h-full w-full"
        style={{ backgroundColor: themeStyles[theme].background }}
      />

      <button
        onClick={goPrev}
        className="absolute left-0 top-0 h-full w-1/4 cursor-pointer opacity-0 hover:opacity-10 hover:bg-black/10"
        aria-label="Previous page"
      />
      <button
        onClick={goNext}
        className="absolute right-0 top-0 h-full w-1/4 cursor-pointer opacity-0 hover:opacity-10 hover:bg-black/10"
        aria-label="Next page"
      />
    </div>
  );
}

export type { NavItem };
