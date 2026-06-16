import { Heart, Play, Clock } from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { MediaFile } from "@/types";
import { cn } from "@/lib/utils";
import { formatDuration, formatFileSize, getResolutionLabel } from "@/lib/utils";
import { getThumbnailUrl } from "@/services/api";
import { useMediaStore } from "@/stores/mediaStore";

interface MediaGridProps {
  media: MediaFile[];
  onSelect: (media: MediaFile) => void;
  selectedId?: string;
}

export default function MediaGrid({ media, onSelect, selectedId }: MediaGridProps) {
  const navigate = useNavigate();
  const { toggleFavorite } = useMediaStore();

  const handlePlay = (e: React.MouseEvent, mediaItem: MediaFile) => {
    e.stopPropagation();
    navigate(`/player/${mediaItem.id}`);
  };

  const handleFavorite = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    toggleFavorite(id);
  };

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      {media.map((item) => (
        <div
          key={item.id}
          onClick={() => onSelect(item)}
          className={cn(
            "group bg-dark-800 rounded-xl overflow-hidden cursor-pointer card-hover",
            selectedId === item.id && "ring-2 ring-primary-500"
          )}
        >
          {/* Thumbnail */}
          <div className="relative aspect-video bg-dark-700">
            {item.thumbnail ? (
              <img
                src={getThumbnailUrl(item.id)}
                alt={item.name}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center">
                <span className="text-4xl">
                  {item.media_type === "Video" ? "🎬" : "🎵"}
                </span>
              </div>
            )}

            {/* Overlay */}
            <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
              <button
                onClick={(e) => handlePlay(e, item)}
                className="w-12 h-12 bg-primary-600 rounded-full flex items-center justify-center hover:bg-primary-700 transition-colors"
              >
                <Play className="w-6 h-6 text-white fill-white" />
              </button>
            </div>

            {/* Duration Badge */}
            {item.duration && (
              <div className="absolute bottom-2 right-2 bg-black/80 px-2 py-1 rounded text-xs text-white">
                {formatDuration(item.duration)}
              </div>
            )}

            {/* Type Badge */}
            <div className="absolute top-2 left-2 bg-black/80 px-2 py-1 rounded text-xs text-white">
              {item.media_type}
            </div>

            {/* Favorite Button */}
            <button
              onClick={(e) => handleFavorite(e, item.id)}
              className="absolute top-2 right-2 p-1.5 bg-black/50 rounded-full opacity-0 group-hover:opacity-100 transition-opacity hover:bg-black/80"
            >
              <Heart
                className={cn(
                  "w-4 h-4",
                  item.favorite
                    ? "text-red-500 fill-red-500"
                    : "text-white"
                )}
              />
            </button>
          </div>

          {/* Info */}
          <div className="p-3">
            <h3 className="text-sm font-medium text-white truncate">
              {item.name}
            </h3>
            <div className="flex items-center gap-2 mt-1 text-xs text-dark-400">
              {item.metadata.width && item.metadata.height && (
                <span>{getResolutionLabel(item.metadata.width, item.metadata.height)}</span>
              )}
              <span>{formatFileSize(item.size)}</span>
            </div>

            {/* Progress Bar */}
            {item.play_progress !== null && item.play_progress > 0 && (
              <div className="mt-2">
                <div className="h-1 bg-dark-600 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-primary-500 rounded-full"
                    style={{ width: `${item.play_progress}%` }}
                  />
                </div>
                <div className="flex items-center gap-1 mt-1 text-xs text-dark-400">
                  <Clock className="w-3 h-3" />
                  <span>{Math.round(item.play_progress)}% watched</span>
                </div>
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
