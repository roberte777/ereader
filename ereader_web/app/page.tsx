import Link from "next/link";
import { auth } from "@clerk/nextjs/server";
import { redirect } from "next/navigation";
import { BookOpen, ArrowRight } from "lucide-react";

export default async function Home() {
  const { userId } = await auth();

  if (userId) {
    redirect("/library");
  }

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-4">
      <div className="max-w-md text-center space-y-8">
        <div className="flex justify-center">
          <div className="rounded-2xl bg-foreground/5 p-4">
            <BookOpen className="h-12 w-12" />
          </div>
        </div>

        <div className="space-y-4">
          <h1 className="text-4xl font-bold tracking-tight">E-Reader</h1>
          <p className="text-lg text-foreground/70">
            Your personal e-book library. Read anywhere, sync everywhere.
          </p>
        </div>

        <div className="flex flex-col gap-3 sm:flex-row sm:justify-center">
          <Link
            href="/sign-in"
            className="inline-flex h-12 items-center justify-center gap-2 rounded-lg bg-foreground px-6 text-background font-medium transition-colors hover:bg-foreground/90"
          >
            Sign In
            <ArrowRight className="h-4 w-4" />
          </Link>
          <Link
            href="/sign-up"
            className="inline-flex h-12 items-center justify-center rounded-lg border border-foreground/20 px-6 font-medium transition-colors hover:bg-foreground/5"
          >
            Create Account
          </Link>
        </div>
      </div>
    </div>
  );
}
