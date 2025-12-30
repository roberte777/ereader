import { BookGridSkeleton } from "@/components/ui/skeleton";

export default function LibraryLoading() {
  return (
    <div className="p-4 md:p-6">
      <div className="mb-6 flex items-center justify-between">
        <div className="h-8 w-24 animate-pulse rounded bg-foreground/10" />
        <div className="h-9 w-24 animate-pulse rounded bg-foreground/10" />
      </div>
      <div className="mb-4 flex items-center justify-between py-4">
        <div className="h-5 w-16 animate-pulse rounded bg-foreground/10" />
        <div className="flex gap-2">
          <div className="h-10 w-40 animate-pulse rounded bg-foreground/10" />
          <div className="h-10 w-10 animate-pulse rounded bg-foreground/10" />
          <div className="h-10 w-20 animate-pulse rounded bg-foreground/10" />
        </div>
      </div>
      <BookGridSkeleton count={12} />
    </div>
  );
}
