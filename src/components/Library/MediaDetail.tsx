import {
  X,
  Play,
  Heart,
  Trash2,
  RefreshCw,
  Film,
  Music,
  Clock,
  HardDrive,
  Monitor,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { MediaFile } from "@/types";
import {
  formatDuration,
  formatFileSize,
  formatDate,
  getResolutionLabel,
} from "@/lib/utils";
import { useMediaStore } from "@/stores/mediaStore";

interface MediaDetailProps {
  media: MediaFile;
  onClose: () => void;
}

export default function MediaDetail({ media, onClose }: MediaDetailProps) {
  const navigate = useNavigate();
  const { toggleFavorite, deleteMedia } = useMediaStore();

  const handlePlay = () => {
    navigate(`/player/${media.id}`);
  };

  const handleFavorite = () => {
    toggleFavorite(media.id);
  };

  const handleDelete = () => {
    if (window.confirm("Are you sure you want to delete this media?")) {
      deleteMedia(media.id);
      onClose();
    }
  };

  return (
    <div className="bg-dark-800 rounded-xl p-6 animate-fade-in">
      {/* Header */}
      <div className="flex items-start justify-between mb-4">
        <h2 className="text-lg font-semibold text-white pr-4">{media.name}</h2>
        <button
          onClick={onClose}
          className="p-1 hover:bg-dark-700 rounded-lg transition-colors"
        >
          <X className="w-5 h-5 text-dark-400" />
        </button>
      </div>

      {/* Type Badge */}
      <div className="flex items-center gap-2 mb-4">
        <span
          className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium ${
            media.media_type === "Video"
              ? "bg-blue-600/20 text-blue-400"
              : "bg-green-600/20 text-green-400"
          }`}
        >
          {media.media_type === "Video" ? (
            <Film className="w-3 h-3" />
          ) : (
            <Music className="w-3 h-3" />
          )}
          {media.media_type}
        </span>
        <span className="text-xs text-dark-400 uppercase">{media.format}</span>
      </div>

      {/* Actions */}
      <div className="flex gap-2 mb-6">
        <button
          onClick={handlePlay}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
        >
          <Play className="w-4 h-4 fill-white" />
          <span>Play</span>
        </button>
        <button
          onClick={handleFavorite}
          className={`px-4 py-2 rounded-lg transition-colors ${
            media.favorite
              ? "bg-red-600/20 text-red-400 hover:bg-red-600/30"
              : "bg-dark-700 text-dark-300 hover:bg-dark-600"
          }`}
        >
          <Heart
            className={`w-4 h-4 ${media.favorite ? "fill-current" : ""}`}
          />
        </button>
        <button className="px-4 py-2 bg-dark-700 text-dark-300 hover:bg-dark-600 rounded-lg transition-colors">
          <RefreshCw className="w-4 h-4" />
        </button>
        <button
          onClick={handleDelete}
          className="px-4 py-2 bg-dark-700 text-red-400 hover:bg-red-600/20 rounded-lg transition-colors"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      {/* Details */}
      <div className="space-y-4">
        {/* Duration */}
        {media.duration && (
          <div className="flex items-center gap-3">
            <Clock className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Duration</p>
              <p className="text-white">{formatDuration(media.duration)}</p>
            </div>
          </div>
        )}

        {/* File Size */}
        <div className="flex items-center gap-3">
          <HardDrive className="w-4 h-4 text-dark-400" />
          <div>
            <p className="text-sm text-dark-400">File Size</p>
            <p className="text-white">{formatFileSize(media.size)}</p>
          </div>
        </div>

        {/* Resolution */}
        {media.metadata.width && media.metadata.height && (
          <div className="flex items-center gap-3">
            <Monitor className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Resolution</p>
              <p className="text-white">
                {getResolutionLabel(media.metadata.width, media.metadata.height)}{" "}
                ({media.metadata.width}x{media.metadata.height})
              </p>
            </div>
          </div>
        )}

        {/* Video Codec */}
        {media.metadata.video_codec && (
          <div className="flex items-center gap-3">
            <Film className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Video Codec</p>
              <p className="text-white">{media.metadata.video_codec}</p>
            </div>
          </div>
        )}

        {/* Audio Codec */}
        {media.metadata.audio_codec && (
          <div className="flex items-center gap-3">
            <Music className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Audio Codec</p>
              <p className="text-white">{media.metadata.audio_codec}</p>
            </div>
          </div>
        )}

        {/* Bitrate */}
        {media.metadata.bitrate && (
          <div className="flex items-center gap-3">
            <HardDrive className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Bitrate</p>
              <p className="text-white">
                {(media.metadata.bitrate / 1000).toFixed(0)} kbps
              </p>
            </div>
          </div>
        )}

        {/* FPS */}
        {media.metadata.fps && (
          <div className="flex items-center gap-3">
            <Film className="w-4 h-4 text-dark-400" />
            <div>
              <p className="text-sm text-dark-400">Frame Rate</p>
              <p className="text-white">{media.metadata.fps.toFixed(2)} fps</p>
            </div>
          </div>
        )}

        {/* Dates */}
        <div className="border-t border-dark-700 pt-4 mt-4">
          <div className="text-sm text-dark-400">
            <p>Added: {formatDate(media.created_at)}</p>
            <p>Modified: {formatDate(media.modified_at)}</p>
            {media.last_played && (
              <p>Last Played: {formatDate(media.last_played)}</p>
            )}
          </div>
        </div>

        {/* Tags */}
        {media.tags.length > 0 && (
          <div className="border-t border-dark-700 pt-4">
            <p className="text-sm text-dark-400 mb-2">Tags</p>
            <div className="flex flex-wrap gap-2">
              {media.tags.map((tag, index) => (
                <span
                  key={index}
                  className="px-2 py-1 bg-dark-700 text-dark-300 rounded text-xs"
                >
                  {tag}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Path */}
        <div className="border-t border-dark-700 pt-4">
          <p className="text-sm text-dark-400 mb-1">File Path</p>
          <p className="text-xs text-dark-500 break-all">{media.path}</p>
        </div>
      </div>
    </div>
  );
}
