import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

export async function GET(request: NextRequest) {
  try {
    const { getToken } = await auth();
    const token = await getToken({ template: "default" });

    const searchParams = request.nextUrl.searchParams;
    const path = searchParams.get("path");

    if (!path) {
      return new NextResponse("Missing path parameter", { status: 400 });
    }

    // Construct the backend URL
    const backendUrl = `${API_BASE_URL}/api/v1${path}`;

    // Fetch from backend with auth headers
    const headers: HeadersInit = {};
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(backendUrl, { headers });

    if (!response.ok) {
      return new NextResponse("Failed to fetch image", {
        status: response.status,
      });
    }

    // Get the image data
    const imageBuffer = await response.arrayBuffer();
    const contentType = response.headers.get("content-type") || "image/jpeg";

    // Return the image with appropriate headers
    return new NextResponse(imageBuffer, {
      status: 200,
      headers: {
        "Content-Type": contentType,
        "Cache-Control": "public, max-age=31536000, immutable",
      },
    });
  } catch (error) {
    console.error("Image proxy error:", error);
    return new NextResponse("Internal server error", { status: 500 });
  }
}
