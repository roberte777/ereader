"use client";

import Link from "next/link";
import { ArrowLeft, List, Settings, ChevronLeft, ChevronRight } from "lucide-react";
import { cn, formatProgress } from "@/lib/utils";

interface ReaderToolbarProps {
  bookId: string;
  title: string;
  chapter?: string | null;
  progress: number;
  onToggleToc: () => void;
  onToggleSettings: () => void;
  onPrev?: () => void;
  onNext?: () => void;
  showToc: boolean;
  showSettings: boolean;
}

export function ReaderToolbar({
  bookId,
  title,
  chapter,
  progress,
  onToggleToc,
  onToggleSettings,
  onPrev,
  onNext,
  showToc,
  showSettings,
}: ReaderToolbarProps) {
  return (
    <div className="absolute top-0 left-0 right-0 z-40 flex items-center justify-between border-b border-foreground/10 bg-background/95 backdrop-blur px-4 py-3">
      <div className="flex items-center gap-3">
        <Link
          href={`/books/${bookId}`}
          className="rounded-lg p-2 hover:bg-foreground/5"
        >
          <ArrowLeft className="h-5 w-5" />
        </Link>

        <button
          onClick={onToggleToc}
          className={cn(
            "rounded-lg p-2 hover:bg-foreground/5",
            showToc && "bg-foreground/10"
          )}
          title="Table of Contents"
        >
          <List className="h-5 w-5" />
        </button>
      </div>

      <div className="flex-1 min-w-0 px-4 text-center">
        <h1 className="truncate text-sm font-medium">{title}</h1>
        {chapter && (
          <p className="truncate text-xs text-foreground/60">{chapter}</p>
        )}
      </div>

      <div className="flex items-center gap-3">
        <span className="text-sm text-foreground/60">{formatProgress(progress)}</span>

        <button
          onClick={onToggleSettings}
          className={cn(
            "rounded-lg p-2 hover:bg-foreground/5",
            showSettings && "bg-foreground/10"
          )}
          title="Settings"
        >
          <Settings className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}

export function ReaderBottomBar({ progress }: { progress: number }) {
  return (
    <div className="absolute bottom-0 left-0 right-0 h-1 bg-foreground/10">
      <div
        className="h-full bg-foreground/50 transition-all duration-300"
        style={{ width: `${progress * 100}%` }}
      />
    </div>
  );
}
