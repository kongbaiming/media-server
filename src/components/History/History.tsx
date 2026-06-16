import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Clock, Loader2, Play, Video } from "lucide-react";
import type { DouyinVideo, MediaFile, PlayHistory } from "@/types";
import {
  getHistory,
  getLibrary,
  getThumbnailUrl,
  parseDouyinUrl,
} from "@/services/api";
import DouyinPlayer from "@/components/Player/DouyinPlayer";
import { formatDuration } from "@/lib/utils";

function isDouyinHistory(entry: PlayHistory): boolean {
  return entry.source === "douyin" || entry.media_id.startsWith("douyin:");
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return date.toLocaleString();
}

export default function History() {
  const navigate = useNavigate();
  const [history, setHistory] = useState<PlayHistory[]>([]);
  const [library, setLibrary] = useState<MediaFile[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [douyinVideo, setDouyinVideo] = useState<DouyinVideo | null>(null);
  const [initialProgress, setInitialProgress] = useState(0);
  const [loadingId, setLoadingId] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const [historyResponse, libraryResponse] = await Promise.all([
          getHistory(),
          getLibrary({ per_page: 1000 }),
        ]);

        if (historyResponse.success && historyResponse.data) {
          setHistory(historyResponse.data);
        } else {
          setHistory([]);
        }

        if (libraryResponse.success && libraryResponse.data) {
          setLibrary(libraryResponse.data.items);
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load history");
      } finally {
        setIsLoading(false);
      }
    };

    load();
  }, []);

  const handlePlay = async (entry: PlayHistory) => {
    if (isDouyinHistory(entry)) {
      const shareUrl = entry.share_url;
      if (!shareUrl) {
        setError("Missing Douyin share URL");
        return;
      }

      setLoadingId(entry.media_id);
      setError(null);
      try {
        const response = await parseDouyinUrl(shareUrl);
        if (response.success && response.data) {
          setInitialProgress(
            entry.duration > 0 ? (entry.progress / 100) * entry.duration : 0
          );
          setDouyinVideo(response.data);
        } else {
          setError(response.error || "Failed to load Douyin video");
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load video");
      } finally {
        setLoadingId(null);
      }
      return;
    }

    navigate(`/player/${entry.media_id}`);
  };

  const getEntryTitle = (entry: PlayHistory): string => {
    if (isDouyinHistory(entry)) {
      return entry.title || "Douyin Video";
    }
    return library.find((item) => item.id === entry.media_id)?.name || "Unknown Media";
  };

  const getEntrySubtitle = (entry: PlayHistory): string => {
    if (isDouyinHistory(entry)) {
      return entry.author ? `@${entry.author}` : "Douyin";
    }
    const media = library.find((item) => item.id === entry.media_id);
    return media?.format?.toUpperCase() || "Local Media";
  };

  const getEntryCover = (entry: PlayHistory): string | null => {
    if (isDouyinHistory(entry)) {
      return entry.cover || null;
    }
    const media = library.find((item) => item.id === entry.media_id);
    return media ? getThumbnailUrl(media.id) : null;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 text-primary-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white">Recent</h1>
        <p className="text-dark-400 mt-1">Your recently played media and Douyin videos</p>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-600/20 border border-red-600/50 rounded-xl text-red-300">
          {error}
        </div>
      )}

      {history.length === 0 ? (
        <div className="flex-1 flex flex-col items-center justify-center text-dark-400">
          <Clock className="w-12 h-12 mb-4" />
          <p>No playback history yet</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {history.map((entry) => {
            const cover = getEntryCover(entry);
            const douyin = isDouyinHistory(entry);

            return (
              <button
                key={entry.media_id}
                onClick={() => handlePlay(entry)}
                disabled={loadingId === entry.media_id}
                className="bg-dark-800 rounded-xl overflow-hidden text-left hover:bg-dark-700 transition-colors group disabled:opacity-60"
              >
                <div className="relative aspect-video bg-dark-900">
                  {cover ? (
                    <img
                      src={cover}
                      alt={getEntryTitle(entry)}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      {douyin ? (
                        <Video className="w-10 h-10 text-dark-500" />
                      ) : (
                        <Play className="w-10 h-10 text-dark-500" />
                      )}
                    </div>
                  )}

                  <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    {loadingId === entry.media_id ? (
                      <Loader2 className="w-10 h-10 text-white animate-spin" />
                    ) : (
                      <div className="w-14 h-14 bg-white/20 backdrop-blur-sm rounded-full flex items-center justify-center">
                        <Play className="w-7 h-7 text-white fill-white ml-1" />
                      </div>
                    )}
                  </div>

                  {entry.duration > 0 && (
                    <span className="absolute bottom-2 right-2 px-2 py-1 bg-black/70 rounded text-xs text-white">
                      {formatDuration(entry.duration)}
                    </span>
                  )}

                  {douyin && (
                    <span className="absolute top-2 left-2 px-2 py-1 bg-primary-600/90 rounded text-xs text-white">
                      Douyin
                    </span>
                  )}
                </div>

                <div className="p-4">
                  <h2 className="font-medium text-white line-clamp-2">
                    {getEntryTitle(entry)}
                  </h2>
                  <p className="text-sm text-dark-400 mt-1">{getEntrySubtitle(entry)}</p>
                  <p className="text-xs text-dark-500 mt-2">
                    {formatTimestamp(entry.timestamp)}
                    {entry.progress > 0 && entry.duration > 0
                      ? ` · ${Math.round(entry.progress)}% watched`
                      : ""}
                  </p>
                </div>
              </button>
            );
          })}
        </div>
      )}

      {douyinVideo && (
        <DouyinPlayer
          video={douyinVideo}
          initialProgress={initialProgress}
          onClose={() => {
            setDouyinVideo(null);
            setInitialProgress(0);
            getHistory().then((response) => {
              if (response.success && response.data) {
                setHistory(response.data);
              }
            });
          }}
        />
      )}
    </div>
  );
}
