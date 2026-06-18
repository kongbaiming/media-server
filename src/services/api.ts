import type { ApiResponse, AppConfig, DouyinVideo, MediaFile, OnlineRecentItem, PaginatedResponse, PlayHistory, ProbeResult, ScanProgress, LibraryStatistics, TorrentSessionInfo, TranscodeTask } from "@/types";

const BASE_URL = "http://127.0.0.1:8080";

export function isTauriApp(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

function defaultTimeout(): number {
  return isTauriApp() ? 15000 : 3000;
}

async function fetchApi<T>(
  endpoint: string,
  options?: RequestInit,
  timeoutMs?: number
): Promise<T> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs ?? defaultTimeout());

  try {
    const response = await fetch(`${BASE_URL}${endpoint}`, {
      ...options,
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    return data;
  } catch (error) {
    if (error instanceof Error && error.name === "AbortError") {
      throw new Error("Connection timeout - is the server running?");
    }
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

// Library API
export async function getLibrary(params?: {
  media_type?: string;
  favorite?: boolean;
  sort_by?: string;
  page?: number;
  per_page?: number;
}): Promise<ApiResponse<PaginatedResponse<MediaFile>>> {
  const searchParams = new URLSearchParams();
  if (params?.media_type) searchParams.set("media_type", params.media_type);
  if (params?.favorite !== undefined)
    searchParams.set("favorite", String(params.favorite));
  if (params?.sort_by) searchParams.set("sort_by", params.sort_by);
  if (params?.page) searchParams.set("page", String(params.page));
  if (params?.per_page) searchParams.set("per_page", String(params.per_page));

  const query = searchParams.toString();
  return fetchApi(`/api/library${query ? `?${query}` : ""}`);
}

export async function getMediaDetail(
  id: string
): Promise<ApiResponse<MediaFile>> {
  return fetchApi(`/api/library/${id}`);
}

export async function deleteMedia(
  id: string
): Promise<ApiResponse<string>> {
  return fetchApi(`/api/library/${id}`, { method: "DELETE" });
}

export async function scanLibrary(
  paths: string[]
): Promise<ApiResponse<string>> {
  return fetchApi("/api/library/scan", {
    method: "POST",
    body: JSON.stringify({ paths }),
  });
}

export async function getScanProgress(): Promise<ApiResponse<ScanProgress>> {
  return fetchApi("/api/library/scan/progress");
}

// Search API
export async function searchMedia(
  query: string
): Promise<ApiResponse<MediaFile[]>> {
  return fetchApi(`/api/search?q=${encodeURIComponent(query)}`);
}

// Favorites API
export async function getFavorites(): Promise<ApiResponse<MediaFile[]>> {
  return fetchApi("/api/favorites");
}

export async function toggleFavorite(
  id: string
): Promise<ApiResponse<boolean>> {
  return fetchApi(`/api/favorites/${id}`, { method: "POST" });
}

// History API
export async function getHistory(): Promise<ApiResponse<PlayHistory[]>> {
  return fetchApi("/api/history");
}

export async function addDouyinHistory(
  video: DouyinVideo
): Promise<ApiResponse<string>> {
  return fetchApi("/api/history/douyin", {
    method: "POST",
    body: JSON.stringify(video),
  });
}

export async function updateProgress(
  id: string,
  progress: number,
  duration: number
): Promise<ApiResponse<string>> {
  return fetchApi(
    `/api/history/${id}/progress?progress=${progress}&duration=${duration}`,
    { method: "POST" }
  );
}

export async function getProgress(
  id: string
): Promise<ApiResponse<{ progress: number | null; last_played: string | null }>> {
  return fetchApi(`/api/history/${id}/progress`);
}

// Transcode API
export async function startTranscode(
  mediaId: string,
  quality?: string
): Promise<ApiResponse<string>> {
  return fetchApi("/api/transcode", {
    method: "POST",
    body: JSON.stringify({ media_id: mediaId, quality }),
  });
}

export async function getTranscodeStatus(
  id: string
): Promise<ApiResponse<TranscodeTask>> {
  return fetchApi(`/api/transcode/${id}`);
}

export async function deleteTranscode(
  id: string
): Promise<ApiResponse<string>> {
  return fetchApi(`/api/transcode/${id}`, { method: "DELETE" });
}

// Config API
export async function getConfig(): Promise<ApiResponse<AppConfig>> {
  return fetchApi("/api/config");
}

export async function updateConfig(
  config: Partial<AppConfig>
): Promise<ApiResponse<AppConfig>> {
  return fetchApi("/api/config", {
    method: "PUT",
    body: JSON.stringify(config),
  });
}

// Stats API
export async function getStatistics(): Promise<
  ApiResponse<LibraryStatistics>
> {
  return fetchApi("/api/stats");
}

export async function waitForServer(maxAttempts = 30): Promise<boolean> {
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const response = await fetch(`${BASE_URL}/api/system/info`);
      if (response.ok) return true;
    } catch {
      // server still starting
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  return false;
}

// System API
export async function getSystemInfo(): Promise<
  ApiResponse<{
    ffmpeg_installed: boolean;
    version: string;
    platform: string;
  }>
> {
  return fetchApi("/api/system/info");
}

// Stream URLs
export function getStreamUrl(id: string): string {
  return `${BASE_URL}/api/stream/${id}/direct`;
}

export function getHlsUrl(id: string): string {
  return `${BASE_URL}/api/stream/${id}/master.m3u8`;
}

export function getThumbnailUrl(id: string): string {
  return `${BASE_URL}/api/stream/${id}/thumbnail`;
}

// Douyin API
export async function parseDouyinUrl(
  url: string
): Promise<ApiResponse<DouyinVideo>> {
  return fetchApi(
    "/api/douyin/parse",
    {
      method: "POST",
      body: JSON.stringify({ url }),
    },
    30000
  );
}

export function getDouyinProxyUrl(playUrl: string): string {
  return `${BASE_URL}/api/douyin/proxy?url=${encodeURIComponent(playUrl)}`;
}

export async function getDouyinPlayUrl(
  url: string
): Promise<ApiResponse<{ play_url: string }>> {
  return fetchApi("/api/douyin/play", {
    method: "POST",
    body: JSON.stringify({ url }),
  });
}

// -- Online / live streams -------------------------------------------------

export function onlineStreamUrl(url: string, referer?: string): string {
  const params = new URLSearchParams({ url });
  if (referer) params.set("referer", referer);
  return `${BASE_URL}/api/stream/online?${params.toString()}`;
}

export async function probeOnline(
  url: string,
  referer?: string
): Promise<ApiResponse<ProbeResult>> {
  const params = new URLSearchParams({ url });
  if (referer) params.set("referer", referer);
  return fetchApi(`/api/online/probe?${params.toString()}`);
}

export async function getOnlineRecent(): Promise<ApiResponse<OnlineRecentItem[]>> {
  return fetchApi("/api/online/recent");
}

// -- Torrents --------------------------------------------------------------

export async function addTorrent(
  body: { magnet?: string; torrent_b64?: string }
): Promise<ApiResponse<TorrentSessionInfo>> {
  return fetchApi(
    "/api/torrent/add",
    { method: "POST", body: JSON.stringify(body) },
    30000
  );
}

export async function listTorrents(): Promise<ApiResponse<TorrentSessionInfo[]>> {
  return fetchApi("/api/torrent/list");
}

export async function getTorrent(
  id: string
): Promise<ApiResponse<TorrentSessionInfo>> {
  return fetchApi(`/api/torrent/${encodeURIComponent(id)}`);
}

export async function deleteTorrent(id: string): Promise<ApiResponse<null>> {
  return fetchApi(`/api/torrent/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

