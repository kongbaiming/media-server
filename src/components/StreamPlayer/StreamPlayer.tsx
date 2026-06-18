// Generic player used by the Online and Torrent pages. Picks HLS.js for
// m3u8, native playback for everything else, and lets the parent control
// progress reporting.

import { useEffect, useRef, useState } from "react";
import Hls from "hls.js";
import Plyr from "plyr";
import "plyr/dist/plyr.css";
import { ArrowLeft } from "lucide-react";

export interface StreamPlayerProps {
  title: string;
  src: string;
  /**
   * If true, the source is HLS (m3u8). The player loads it via hls.js on
   * browsers that need it (everything except Safari).
   */
  isHls?: boolean;
  /**
   * Called roughly every 10 seconds with (currentTime, duration) so the
   * host can persist progress / refresh history. Not used for online +
   * torrent in the current build, but wired for future.
   */
  onProgress?: (currentTime: number, duration: number) => void;
  /**
   * Subtitle / captions tracks to expose. Kept for symmetry with the
   * local Player; not exercised yet by online / torrent.
   */
  tracks?: { src: string; label: string; srclang: string }[];
  onBack?: () => void;
}

export default function StreamPlayer({
  title,
  src,
  isHls,
  onProgress,
  tracks,
  onBack,
}: StreamPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const playerRef = useRef<Plyr | null>(null);
  const hlsRef = useRef<Hls | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!videoRef.current || !src) return;
    const video = videoRef.current;

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
      settings: ["captions", "speed"],
      speed: { selected: 1, options: [0.5, 0.75, 1, 1.25, 1.5, 2] },
    });

    let hls: Hls | null = null;
    if (isHls && Hls.isSupported()) {
      hls = new Hls({
        // Live streams have no duration; hls.js handles this internally.
        liveDurationInfinity: true,
      });
      hls.loadSource(src);
      hls.attachMedia(video);
      hls.on(Hls.Events.MANIFEST_PARSED, () => {
        video.play().catch((e) => {
          // autoplay may be blocked; the user can press play manually
          console.warn("autoplay blocked:", e);
        });
      });
      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) {
          console.error("HLS fatal error:", data);
          setError("Stream error. Falling back to direct playback...");
          // try direct fallback
          video.src = src;
          video.play().catch(() => {});
        }
      });
      hlsRef.current = hls;
    } else if (isHls && video.canPlayType("application/vnd.apple.mpegurl")) {
      video.src = src;
      video.play().catch(() => {});
    } else {
      video.src = src;
      video.play().catch(() => {});
    }

    if (tracks) {
      for (const t of tracks) {
        const track = document.createElement("track");
        track.kind = "captions";
        track.label = t.label;
        track.srclang = t.srclang;
        track.src = t.src;
        video.appendChild(track);
      }
    }

    const interval = onProgress
      ? window.setInterval(() => {
          if (video.currentTime > 0 && video.duration > 0) {
            onProgress(video.currentTime, video.duration);
          }
        }, 10000)
      : null;

    return () => {
      if (interval !== null) window.clearInterval(interval);
      if (hlsRef.current) {
        hlsRef.current.destroy();
        hlsRef.current = null;
      }
      if (playerRef.current) {
        playerRef.current.destroy();
        playerRef.current = null;
      }
    };
  }, [src, isHls, onProgress, tracks]);

  return (
    <div className="h-full flex flex-col bg-dark-950 text-white">
      <div className="flex items-center gap-3 px-4 py-3 border-b border-dark-800">
        {onBack && (
          <button
            type="button"
            onClick={onBack}
            className="text-dark-300 hover:text-white"
            aria-label="Back"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
        )}
        <h2 className="text-lg font-semibold truncate">{title}</h2>
      </div>
      <div className="flex-1 flex items-center justify-center bg-black">
        {error && (
          <div className="text-red-400 text-sm absolute top-16 right-4 bg-dark-900/80 px-3 py-2 rounded">
            {error}
          </div>
        )}
        <video
          ref={videoRef}
          className="w-full h-full max-h-[calc(100vh-80px)]"
          playsInline
          controls
        />
      </div>
    </div>
  );
}
