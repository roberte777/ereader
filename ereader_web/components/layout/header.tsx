"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { UserButton } from "@clerk/nextjs";
import { Search, Menu, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/lib/store/ui-store";
import { useLibraryStore } from "@/lib/store/library-store";
import { MobileNav } from "./mobile-nav";

export function Header() {
  const router = useRouter();
  const { mobileMenuOpen, setMobileMenuOpen } = useUIStore();
  const { searchQuery, setSearchQuery } = useLibraryStore();
  const [localSearch, setLocalSearch] = useState(searchQuery);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (localSearch !== searchQuery) {
        setSearchQuery(localSearch);
        if (localSearch) {
          router.push(`/library?q=${encodeURIComponent(localSearch)}`);
        }
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [localSearch, searchQuery, setSearchQuery, router]);

  return (
    <>
      <header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-foreground/10 bg-background/95 backdrop-blur px-4 md:px-6">
        <button
          onClick={() => setMobileMenuOpen(true)}
          className="md:hidden p-2 -ml-2 hover:bg-foreground/5 rounded-lg"
        >
          <Menu className="h-5 w-5" />
        </button>

        <div className="flex-1 max-w-md mx-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-foreground/50" />
            <input
              type="text"
              placeholder="Search books..."
              value={localSearch}
              onChange={(e) => setLocalSearch(e.target.value)}
              className="w-full h-10 rounded-lg border border-foreground/20 bg-transparent pl-10 pr-4 text-sm placeholder:text-foreground/50 focus:outline-none focus:ring-2 focus:ring-foreground/20 focus:border-transparent"
            />
            {localSearch && (
              <button
                onClick={() => {
                  setLocalSearch("");
                  setSearchQuery("");
                }}
                className="absolute right-3 top-1/2 -translate-y-1/2 p-1 hover:bg-foreground/10 rounded"
              >
                <X className="h-3 w-3" />
              </button>
            )}
          </div>
        </div>

        <UserButton
          afterSignOutUrl="/"
          appearance={{
            elements: {
              avatarBox: "h-9 w-9",
            },
          }}
        />
      </header>

      <MobileNav isOpen={mobileMenuOpen} onClose={() => setMobileMenuOpen(false)} />
    </>
  );
}
