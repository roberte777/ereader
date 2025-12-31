import type { ApiError } from "./types";

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

export class ApiClientError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string
  ) {
    super(message);
    this.name = "ApiClientError";
  }
}

export class ApiClient {
  private baseUrl: string;
  private getToken: () => Promise<string | null>;

  constructor(getToken: () => Promise<string | null>) {
    this.baseUrl = `${API_BASE_URL}/api/v1`;
    this.getToken = getToken;
  }

  async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const token = await this.getToken();

    const headers: HeadersInit = {
      ...options.headers,
    };

    if (!(options.body instanceof FormData)) {
      (headers as Record<string, string>)["Content-Type"] = "application/json";
    }

    if (token) {
      (headers as Record<string, string>)["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers,
    });

    if (!response.ok) {
      let errorData: ApiError | null = null;
      try {
        errorData = await response.json();
      } catch {
        // Ignore JSON parse errors
      }

      throw new ApiClientError(
        response.status,
        errorData?.error?.code || "unknown_error",
        errorData?.error?.message || `Request failed with status ${response.status}`
      );
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  async get<T>(endpoint: string, params?: object): Promise<T> {
    let url = endpoint;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      const queryString = searchParams.toString();
      if (queryString) {
        url += `?${queryString}`;
      }
    }
    return this.request<T>(url, { method: "GET" });
  }

  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async put<T>(endpoint: string, data?: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  async delete<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: "DELETE" });
  }

  async uploadFile<T>(endpoint: string, file: File): Promise<T> {
    const formData = new FormData();
    formData.append("file", file);

    return this.request<T>(endpoint, {
      method: "POST",
      body: formData,
    });
  }

  getDownloadUrl(endpoint: string): string {
    return `${this.baseUrl}${endpoint}`;
  }

  getCoverUrl(bookId: string, size: "small" | "medium" | "large" = "medium"): string {
    // Use Next.js API route proxy to handle authentication
    return `/api/images?path=/books/${bookId}/cover/${size}`;
  }
}

let clientInstance: ApiClient | null = null;

export function createApiClient(getToken: () => Promise<string | null>): ApiClient {
  clientInstance = new ApiClient(getToken);
  return clientInstance;
}

export function getApiClient(): ApiClient {
  if (!clientInstance) {
    throw new Error("API client not initialized. Call createApiClient first.");
  }
  return clientInstance;
}
