// Media types
export type MediaType = "Video" | "Audio";

export type ScanStatus = "Idle" | "Scanning" | "Completed" | { Error: string };

export type TranscodeStatus =
  | "Pending"
  | "InProgress"
  | "Completed"
  | { Failed: string };

export type TranscodeQuality = "High" | "Medium" | "Low" | "Auto";

export interface MediaMetadata {
  bitrate: number | null;
  width: number | null;
  height: number | null;
  video_codec: string | null;
  audio_codec: string | null;
  sample_rate: number | null;
  channels: number | null;
  fps: number | null;
}

export interface MediaFile {
  id: string;
  name: string;
  path: string;
  media_type: MediaType;
  size: number;
  duration: number | null;
  format: string;
  created_at: string;
  modified_at: string;
  thumbnail: string | null;
  metadata: MediaMetadata;
  tags: string[];
  favorite: boolean;
  last_played: string | null;
  play_progress: number | null;
  scraped?: ScrapedMetadata;
}

export interface PlayHistory {
  media_id: string;
  timestamp: string;
  progress: number;
  duration: number;
  source?: "local" | "douyin";
  title?: string | null;
  author?: string | null;
  cover?: string | null;
  share_url?: string | null;
}

export interface AppConfig {
  library_paths: string[];
  auto_scan: boolean;
  scan_interval: number;
  transcode_quality: TranscodeQuality;
  hardware_acceleration: boolean;
  default_subtitle_language: string;
  server_port: number;
  thumbnail_width: number;
  thumbnail_height: number;
  tmdb_api_key?: string;
  synology_shares?: SynologyShare[];
}

export interface ScanProgress {
  total_files: number;
  processed_files: number;
  current_file: string;
  status: ScanStatus;
}

export interface TranscodeTask {
  id: string;
  media_id: string;
  status: TranscodeStatus;
  progress: number;
  output_path: string;
  quality: TranscodeQuality;
}

export interface LibraryStatistics {
  total_files: number;
  video_count: number;
  audio_count: number;
  total_size: number;
  total_duration: number;
  favorite_count: number;
  play_count: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error: string | null;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

// Douyin types
export interface DouyinVideo {
  id: string;
  title: string;
  author: string;
  author_avatar: string | null;
  cover: string | null;
  duration: number;
  play_url: string;
  play_url_no_watermark: string;
  share_url: string;
  description: string | null;
  likes: number | null;
  comments: number | null;
  shares: number | null;
}


// Online / live stream
export type StreamKind = "hls" | "direct" | "audio" | "other";

export interface ProbeResult {
  url: string;
  content_type: string | null;
  content_length: number | null;
  accepts_ranges: boolean;
  kind: StreamKind;
}

export interface OnlineRecentItem {
  url: string;
  title: string | null;
  kind: string | null;
  last_played: string;
}

// Torrent
export type SessionStatus = "resolving" | "downloading" | "ready" | "failed";

export interface TorrentFileInfo {
  path: string;
  length: number;
  downloaded: number;
}

export interface TorrentSessionInfo {
  id: string;
  name: string;
  status: SessionStatus;
  progress: number;
  downloaded: number;
  total: number;
  download_speed_bps: number;
  info_hash: string;
  files: TorrentFileInfo[];
  error: string | null;
  stream_url: string;
}


// Metadata scraper (TMDB)
export interface ScrapedMetadata {
  source?: string;
  tmdb_id?: number;
  title?: string;
  original_title?: string;
  year?: number;
  plot?: string;
  rating?: number;
  genres?: string[];
  director?: string;
  cast?: string[];
  runtime_minutes?: number;
  poster_path?: string;
  backdrop_path?: string;
  collection_id?: number;
  collection_name?: string;
  scraped_at?: string;
  scrape_error?: string;
}

export interface MovieCollection {
  id: number;
  name: string;
  overview?: string;
  poster_path?: string;
  backdrop_path?: string;
  movies: MediaFile[];
}

export interface ScraperStatus {
  enabled: boolean;
  queue_len: number;
  last_run_at?: string;
  last_error?: string;
  scraped: number;
  failed: number;
}

export interface SynologyShare {
  quickconnect_id: string;
  host?: string;
  share: string;
  label?: string;
}

export interface ScraperImageUrl {
  url: string;
  size: string;
}
