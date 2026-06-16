import { useEffect, useRef, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft, Settings, Subtitles, Volume2 } from "lucide-react";
import Hls from "hls.js";
import Plyr from "plyr";
import "plyr/dist/plyr.css";
import type { MediaFile } from "@/types";
import { getMediaDetail } from "@/services/api";
import { getStreamUrl, getHlsUrl, updateProgress } from "@/services/api";
import { formatDuration } from "@/lib/utils";

export default function Player() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const videoRef = useRef<HTMLVideoElement>(null);
  const playerRef = useRef<Plyr | null>(null);
  const hlsRef = useRef<Hls | null>(null);

  const [media, setMedia] = useState<MediaFile | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;

    const loadMedia = async () => {
      try {
        const response = await getMediaDetail(id);
        if (response.success && response.data) {
          setMedia(response.data);
        } else {
          setError("Media not found");
        }
      } catch (err) {
        setError("Failed to load media");
      } finally {
        setIsLoading(false);
      }
    };

    loadMedia();
  }, [id]);

  useEffect(() => {
    if (!media || !videoRef.current) return;

    const video = videoRef.current;
    const source = getStreamUrl(media.id);

    // Initialize Plyr
    playerRef.current = new Plyr(video, {
      controls: [
        "play-large",
        "play",
        "progress",
        "current-time",
        "duration",
        "mute",
        "volume",
        "captions",
        "settings",
        "pip",
        "airplay",
        "fullscreen",
      ],
      settings: ["captions", "quality", "speed"],
      speed: { selected: 1, options: [0.5, 0.75, 1, 1.25, 1.5, 2] },
    });

    // Check if HLS is needed
    const hlsUrl = getHlsUrl(media.id);

    if (Hls.isSupported()) {
      const hls = new Hls({
        startPosition: media.play_progress
          ? (media.play_progress / 100) * (media.duration || 0)
          : 0,
      });

      hls.loadSource(hlsUrl);
      hls.attachMedia(video);

      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        video.play().catch(console.error);
      });

      hls.on(Hls.Events.ERROR, (event, data) => {
        if (data.fatal) {
          console.error("HLS fatal error:", data);
          // Fallback to direct stream
          video.src = source;
          video.play().catch(console.error);
        }
      });

      hlsRef.current = hls;
    } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
      // Native HLS support (Safari)
      video.src = hlsUrl;
      video.play().catch(console.error);
    } else {
      // Fallback to direct stream
      video.src = source;
      video.play().catch(console.error);
    }

    // Save progress periodically
    const progressInterval = setInterval(() => {
      if (video.currentTime > 0 && media.duration) {
        const progress = (video.currentTime / media.duration) * 100;
        updateProgress(media.id, progress, media.duration).catch(console.error);
      }
    }, 10000);

    // Cleanup
    return () => {
      clearInterval(progressInterval);
      if (hlsRef.current) {
        hlsRef.current.destroy();
      }
      if (playerRef.current) {
        playerRef.current.destroy();
      }
    };
  }, [media]);

  const handleBack = () => {
    // Save final progress
    if (videoRef.current && media?.duration) {
      const progress =
        (videoRef.current.currentTime / media.duration) * 100;
      updateProgress(media.id, progress, media.duration).catch(console.error);
    }
    navigate(-1);
  };

  if (isLoading) {
    return (
      <div className="h-screen bg-black flex items-center justify-center">
        <div className="text-white">Loading...</div>
      </div>
    );
  }

  if (error || !media) {
    return (
      <div className="h-screen bg-black flex flex-col items-center justify-center">
        <p className="text-red-400 mb-4">{error || "Media not found"}</p>
        <button
          onClick={() => navigate("/")}
          className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
        >
          Back to Library
        </button>
      </div>
    );
  }

  return (
    <div className="h-screen bg-black flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 bg-gradient-to-b from-black/80 to-transparent absolute top-0 left-0 right-0 z-10">
        <button
          onClick={handleBack}
          className="flex items-center gap-2 text-white hover:text-primary-400 transition-colors"
        >
          <ArrowLeft className="w-5 h-5" />
          <span>Back</span>
        </button>
        <h1 className="text-white font-medium truncate max-w-md">
          {media.name}
        </h1>
        <div className="flex items-center gap-2">
          <button className="p-2 text-white hover:text-primary-400 transition-colors">
            <Subtitles className="w-5 h-5" />
          </button>
          <button className="p-2 text-white hover:text-primary-400 transition-colors">
            <Settings className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Video Player */}
      <div className="flex-1 flex items-center justify-center">
        <video
          ref={videoRef}
          className="w-full h-full"
          crossOrigin="anonymous"
        />
      </div>

      {/* Info Bar */}
      <div className="p-4 bg-gradient-to-t from-black/80 to-transparent absolute bottom-0 left-0 right-0 z-10">
        <div className="flex items-center justify-between text-white text-sm">
          <div>
            <p className="font-medium">{media.name}</p>
            <p className="text-dark-400">
              {media.metadata.video_codec && `${media.metadata.video_codec} • `}
              {media.metadata.width &&
                media.metadata.height &&
                `${media.metadata.width}x${media.metadata.height} • `}
              {formatDuration(media.duration)}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Volume2 className="w-4 h-4" />
          </div>
        </div>
      </div>
    </div>
  );
}
