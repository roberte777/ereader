"use client";

import { X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { NavItem } from "./epub-reader";

interface TableOfContentsProps {
  items: NavItem[];
  currentLocation?: string;
  onNavigate: (href: string) => void;
  onClose: () => void;
}

export function TableOfContents({
  items,
  currentLocation,
  onNavigate,
  onClose,
}: TableOfContentsProps) {
  return (
    <div className="absolute left-0 top-0 z-50 h-full w-72 border-r border-foreground/10 bg-background shadow-lg">
      <div className="flex items-center justify-between border-b border-foreground/10 px-4 py-3">
        <h3 className="font-medium">Table of Contents</h3>
        <button onClick={onClose} className="rounded p-1 hover:bg-foreground/5">
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="overflow-auto h-[calc(100%-52px)] p-2">
        <TocItems
          items={items}
          currentLocation={currentLocation}
          onNavigate={onNavigate}
          depth={0}
        />
      </div>
    </div>
  );
}

interface TocItemsProps {
  items: NavItem[];
  currentLocation?: string;
  onNavigate: (href: string) => void;
  depth: number;
}

function TocItems({ items, currentLocation, onNavigate, depth }: TocItemsProps) {
  return (
    <ul className="space-y-0.5">
      {items.map((item, index) => (
        <li key={`${item.id}-${index}`}>
          <button
            onClick={() => onNavigate(item.href)}
            className={cn(
              "w-full rounded-lg px-3 py-2 text-left text-sm hover:bg-foreground/5 transition-colors",
              currentLocation === item.href && "bg-foreground/10 font-medium"
            )}
            style={{ paddingLeft: `${12 + depth * 16}px` }}
          >
            {item.label}
          </button>
          {item.subitems && item.subitems.length > 0 && (
            <TocItems
              items={item.subitems}
              currentLocation={currentLocation}
              onNavigate={onNavigate}
              depth={depth + 1}
            />
          )}
        </li>
      ))}
    </ul>
  );
}
