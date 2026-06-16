import { create } from "zustand";
import type {
  MediaFile,
  AppConfig,
  ScanProgress,
  LibraryStatistics,
} from "@/types";
import * as api from "@/services/api";

interface MediaState {
  // Library
  library: MediaFile[];
  selectedMedia: MediaFile | null;
  isLoading: boolean;
  error: string | null;

  // Config
  config: AppConfig | null;

  // Scan
  scanProgress: ScanProgress | null;
  isScanning: boolean;

  // Stats
  statistics: LibraryStatistics | null;

  // Filters
  filterType: "all" | "video" | "audio";
  filterFavorite: boolean;
  sortBy: "name" | "date" | "size" | "duration";
  currentPage: number;

  // Actions
  fetchLibrary: () => Promise<void>;
  fetchConfig: () => Promise<void>;
  fetchStatistics: () => Promise<void>;
  scanLibrary: (paths: string[]) => Promise<void>;
  selectMedia: (media: MediaFile | null) => void;
  toggleFavorite: (id: string) => Promise<void>;
  deleteMedia: (id: string) => Promise<void>;
  setFilterType: (type: "all" | "video" | "audio") => void;
  setFilterFavorite: (favorite: boolean) => void;
  setSortBy: (sort: "name" | "date" | "size" | "duration") => void;
  setCurrentPage: (page: number) => void;
  updateConfig: (config: Partial<AppConfig>) => Promise<void>;
}

export const useMediaStore = create<MediaState>((set, get) => ({
  // Initial state
  library: [],
  selectedMedia: null,
  isLoading: false,
  error: null,
  config: null,
  scanProgress: null,
  isScanning: false,
  statistics: null,
  filterType: "all",
  filterFavorite: false,
  sortBy: "date",
  currentPage: 1,

  // Fetch library
  fetchLibrary: async () => {
    set({ isLoading: true, error: null });
    try {
      const { filterType, filterFavorite, sortBy, currentPage } = get();
      const response = await api.getLibrary({
        media_type: filterType === "all" ? undefined : filterType,
        favorite: filterFavorite || undefined,
        sort_by: sortBy,
        page: currentPage,
        per_page: 20,
      });

      if (response.success && response.data) {
        set({ library: response.data.items, isLoading: false });
      } else {
        set({ library: [], isLoading: false });
      }
    } catch (error) {
      console.error("Failed to fetch library:", error);
      // Set empty library when server is not available
      set({ library: [], isLoading: false });
    }
  },

  // Fetch config
  fetchConfig: async () => {
    try {
      const response = await api.getConfig();
      if (response.success && response.data) {
        set({ config: response.data });
      }
    } catch (error) {
      console.error("Failed to fetch config:", error);
      // Set default config when server is not available
      set({
        config: {
          library_paths: [],
          auto_scan: false,
          scan_interval: 300,
          transcode_quality: "Auto",
          hardware_acceleration: false,
          default_subtitle_language: "chi",
          server_port: 8080,
          thumbnail_width: 320,
          thumbnail_height: 180,
        },
      });
    }
  },

  // Fetch statistics
  fetchStatistics: async () => {
    try {
      const response = await api.getStatistics();
      if (response.success && response.data) {
        set({ statistics: response.data });
      }
    } catch (error) {
      console.error("Failed to fetch statistics:", error);
      // Set default statistics when server is not available
      set({
        statistics: {
          total_files: 0,
          video_count: 0,
          audio_count: 0,
          total_size: 0,
          total_duration: 0,
          favorite_count: 0,
          play_count: 0,
        },
      });
    }
  },

  // Scan library
  scanLibrary: async (paths: string[]) => {
    set({ isScanning: true });
    try {
      await api.scanLibrary(paths);

      // Poll for progress
      const pollInterval = setInterval(async () => {
        const progressResponse = await api.getScanProgress();
        if (progressResponse.success && progressResponse.data) {
          set({ scanProgress: progressResponse.data });

          if (progressResponse.data.status === "Completed") {
            clearInterval(pollInterval);
            set({ isScanning: false, scanProgress: null });
            get().fetchLibrary();
            get().fetchStatistics();
          }
        }
      }, 1000);
    } catch (error) {
      set({ isScanning: false });
      console.error("Scan failed:", error);
    }
  },

  // Select media
  selectMedia: (media) => {
    set({ selectedMedia: media });
  },

  // Toggle favorite
  toggleFavorite: async (id: string) => {
    try {
      const response = await api.toggleFavorite(id);
      if (response.success) {
        // Update local state
        set((state) => ({
          library: state.library.map((item) =>
            item.id === id ? { ...item, favorite: !item.favorite } : item
          ),
        }));
      }
    } catch (error) {
      console.error("Failed to toggle favorite:", error);
    }
  },

  // Delete media
  deleteMedia: async (id: string) => {
    try {
      const response = await api.deleteMedia(id);
      if (response.success) {
        set((state) => ({
          library: state.library.filter((item) => item.id !== id),
        }));
      }
    } catch (error) {
      console.error("Failed to delete media:", error);
    }
  },

  // Set filter type
  setFilterType: (type) => {
    set({ filterType: type, currentPage: 1 });
    get().fetchLibrary();
  },

  // Set filter favorite
  setFilterFavorite: (favorite) => {
    set({ filterFavorite: favorite, currentPage: 1 });
    get().fetchLibrary();
  },

  // Set sort by
  setSortBy: (sort) => {
    set({ sortBy: sort, currentPage: 1 });
    get().fetchLibrary();
  },

  // Set current page
  setCurrentPage: (page) => {
    set({ currentPage: page });
    get().fetchLibrary();
  },

  // Update config
  updateConfig: async (configUpdate) => {
    try {
      const response = await api.updateConfig(configUpdate);
      if (response.success && response.data) {
        set({ config: response.data });
      }
    } catch (error) {
      console.error("Failed to update config:", error);
    }
  },
}));
