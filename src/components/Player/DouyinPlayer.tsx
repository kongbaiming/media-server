import { useState, useRef, useEffect } from "react";
import {
  X,
  Play,
  Pause,
  Volume2,
  VolumeX,
  Maximize,
  ExternalLink,
  User,
  Heart,
  MessageCircle,
  Share2,
} from "lucide-react";
import type { DouyinVideo } from "@/types";
import { formatDuration } from "@/lib/utils";
import { addDouyinHistory, getDouyinProxyUrl, updateProgress } from "@/services/api";

interface DouyinPlayerProps {
  video: DouyinVideo;
  onClose: () => void;
  initialProgress?: number;
}

export default function DouyinPlayer({
  video,
  onClose,
  initialProgress = 0,
}: DouyinPlayerProps) {
  const historyId = `douyin:${video.id}`;
  const videoRef = useRef<HTMLVideoElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isMuted, setIsMuted] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    addDouyinHistory(video).catch(console.error);
  }, [video]);

  useEffect(() => {
    const videoElement = videoRef.current;
    if (!videoElement) return;

    const handleTimeUpdate = () => {
      setCurrentTime(videoElement.currentTime);
      if (videoElement.duration) {
        setProgress((videoElement.currentTime / videoElement.duration) * 100);
      }
    };

    const handleLoadedMetadata = () => {
      setDuration(videoElement.duration);
      if (initialProgress > 0 && initialProgress < videoElement.duration) {
        videoElement.currentTime = initialProgress;
        setCurrentTime(initialProgress);
        setProgress((initialProgress / videoElement.duration) * 100);
      }
    };

    const handlePlay = () => setIsPlaying(true);
    const handlePause = () => setIsPlaying(false);

    videoElement.addEventListener("timeupdate", handleTimeUpdate);
    videoElement.addEventListener("loadedmetadata", handleLoadedMetadata);
    videoElement.addEventListener("play", handlePlay);
    videoElement.addEventListener("pause", handlePause);

    return () => {
      videoElement.removeEventListener("timeupdate", handleTimeUpdate);
      videoElement.removeEventListener("loadedmetadata", handleLoadedMetadata);
      videoElement.removeEventListener("play", handlePlay);
      videoElement.removeEventListener("pause", handlePause);
    };
  }, [initialProgress]);

  const saveProgress = () => {
    const videoElement = videoRef.current;
    const totalDuration = videoElement?.duration || video.duration;
    if (!videoElement || !totalDuration) return;

    const progressPercent = (videoElement.currentTime / totalDuration) * 100;
    updateProgress(historyId, progressPercent, totalDuration).catch(console.error);
  };

  const handleClose = () => {
    saveProgress();
    onClose();
  };

  const togglePlay = () => {
    const videoElement = videoRef.current;
    if (!videoElement) return;

    if (isPlaying) {
      videoElement.pause();
    } else {
      videoElement.play();
    }
  };

  const toggleMute = () => {
    const videoElement = videoRef.current;
    if (!videoElement) return;

    videoElement.muted = !isMuted;
    setIsMuted(!isMuted);
  };

  const handleFullscreen = () => {
    const videoElement = videoRef.current;
    if (!videoElement) return;

    if (videoElement.requestFullscreen) {
      videoElement.requestFullscreen();
    }
  };

  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    const videoElement = videoRef.current;
    if (!videoElement || !duration) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    videoElement.currentTime = percentage * duration;
  };

  const rawPlayUrl = video.play_url_no_watermark || video.play_url;
  const proxyPlayUrl = rawPlayUrl ? getDouyinProxyUrl(rawPlayUrl) : "";

  return (
    <div className="fixed inset-0 bg-black/90 z-50 flex items-center justify-center">
      <div className="relative w-full max-w-4xl mx-4">
        {/* Close Button */}
        <button
          onClick={handleClose}
          className="absolute -top-10 right-0 p-2 text-white hover:text-gray-300 transition-colors"
        >
          <X className="w-6 h-6" />
        </button>

        <div className="bg-dark-900 rounded-xl overflow-hidden">
          {/* Video Player */}
          <div className="relative aspect-video bg-black">
            <video
              ref={videoRef}
              src={proxyPlayUrl}
              className="w-full h-full"
              playsInline
              onClick={togglePlay}
            />

            {/* Play/Pause Overlay */}
            {!isPlaying && (
              <div
                className="absolute inset-0 flex items-center justify-center cursor-pointer"
                onClick={togglePlay}
              >
                <div className="w-16 h-16 bg-white/20 backdrop-blur-sm rounded-full flex items-center justify-center">
                  <Play className="w-8 h-8 text-white fill-white ml-1" />
                </div>
              </div>
            )}

            {/* Progress Bar */}
            <div
              className="absolute bottom-0 left-0 right-0 h-1 bg-gray-600 cursor-pointer"
              onClick={handleSeek}
            >
              <div
                className="h-full bg-primary-500 transition-all"
                style={{ width: `${progress}%` }}
              />
            </div>

            {/* Controls */}
            <div className="absolute bottom-4 left-4 right-4 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <button
                  onClick={togglePlay}
                  className="p-2 text-white hover:text-primary-400 transition-colors"
                >
                  {isPlaying ? (
                    <Pause className="w-5 h-5" />
                  ) : (
                    <Play className="w-5 h-5 fill-white" />
                  )}
                </button>
                <button
                  onClick={toggleMute}
                  className="p-2 text-white hover:text-primary-400 transition-colors"
                >
                  {isMuted ? (
                    <VolumeX className="w-5 h-5" />
                  ) : (
                    <Volume2 className="w-5 h-5" />
                  )}
                </button>
                <span className="text-white text-sm">
                  {formatDuration(currentTime)} / {formatDuration(duration)}
                </span>
              </div>
              <button
                onClick={handleFullscreen}
                className="p-2 text-white hover:text-primary-400 transition-colors"
              >
                <Maximize className="w-5 h-5" />
              </button>
            </div>
          </div>

          {/* Video Info */}
          <div className="p-6">
            <h2 className="text-xl font-semibold text-white mb-2">
              {video.title}
            </h2>

            <div className="flex items-center gap-4 mb-4">
              <div className="flex items-center gap-2">
                {video.author_avatar ? (
                  <img
                    src={video.author_avatar}
                    alt={video.author}
                    className="w-8 h-8 rounded-full"
                  />
                ) : (
                  <div className="w-8 h-8 bg-dark-700 rounded-full flex items-center justify-center">
                    <User className="w-4 h-4 text-dark-400" />
                  </div>
                )}
                <span className="text-dark-300">{video.author}</span>
              </div>

              <a
                href={video.share_url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-primary-400 hover:text-primary-300 transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                <span className="text-sm">View on Douyin</span>
              </a>
            </div>

            {video.description && (
              <p className="text-dark-300 mb-4">{video.description}</p>
            )}

            {/* Stats */}
            <div className="flex items-center gap-6 text-dark-400">
              {video.likes !== null && (
                <div className="flex items-center gap-1">
                  <Heart className="w-4 h-4" />
                  <span>{formatCount(video.likes)}</span>
                </div>
              )}
              {video.comments !== null && (
                <div className="flex items-center gap-1">
                  <MessageCircle className="w-4 h-4" />
                  <span>{formatCount(video.comments)}</span>
                </div>
              )}
              {video.shares !== null && (
                <div className="flex items-center gap-1">
                  <Share2 className="w-4 h-4" />
                  <span>{formatCount(video.shares)}</span>
                </div>
              )}
              <div className="flex items-center gap-1">
                <span>Duration: {formatDuration(video.duration)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function formatCount(count: number): string {
  if (count >= 10000) {
    return `${(count / 10000).toFixed(1)}w`;
  }
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}k`;
  }
  return count.toString();
}
