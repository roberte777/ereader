"use client";

import { useEffect } from "react";
import { useAuth } from "@clerk/nextjs";
import { Sidebar, Header } from "@/components/layout";
import { createApiClient } from "@/lib/api";

export default function MainLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { getToken } = useAuth();

  useEffect(() => {
    // Create a wrapper that uses the "default" JWT template
    const getTokenWithTemplate = () => getToken({ template: "default" });
    createApiClient(getTokenWithTemplate);
  }, [getToken]);

  return (
    <div className="flex min-h-screen bg-background">
      <Sidebar />
      <div className="flex flex-1 flex-col">
        <Header />
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
