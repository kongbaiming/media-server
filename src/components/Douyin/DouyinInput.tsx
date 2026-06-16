import { useState } from "react";
import { Link, Loader2, AlertCircle, Play } from "lucide-react";
import { parseDouyinUrl } from "@/services/api";
import type { DouyinVideo } from "@/types";
import DouyinPlayer from "@/components/Player/DouyinPlayer";

export default function DouyinInput() {
  const [url, setUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [video, setVideo] = useState<DouyinVideo | null>(null);
  const [showPlayer, setShowPlayer] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!url.trim()) {
      setError("Please enter a Douyin URL");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      console.log("Sending URL to parse:", url.trim());
      console.log("URL length:", url.trim().length);

      const response = await parseDouyinUrl(url.trim());

      console.log("API Response:", response);

      if (response.success && response.data) {
        setVideo(response.data);
      } else {
        setError(response.error || "Failed to parse Douyin URL");
      }
    } catch (err) {
      console.error("Parse error:", err);
      setError(err instanceof Error ? err.message : "Unknown error");
    } finally {
      setIsLoading(false);
    }
  };

  const handlePlay = () => {
    if (video) {
      setShowPlayer(true);
    }
  };

  const handleClosePlayer = () => {
    setShowPlayer(false);
  };

  return (
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold text-white mb-6">Douyin Player</h1>

      {/* Input Form */}
      <form onSubmit={handleSubmit} className="mb-6">
        <div className="flex gap-3">
          <div className="flex-1 relative">
            <Link className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-400" />
            <input
              type="text"
              value={url}
              onChange={(e) => {
                setUrl(e.target.value);
                setError(null);
              }}
              placeholder="Paste Douyin share link or URL..."
              className="w-full pl-12 pr-4 py-3 bg-dark-800 border border-dark-700 rounded-xl text-white placeholder-dark-400 focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20"
              disabled={isLoading}
            />
          </div>
          <button
            type="submit"
            disabled={isLoading || !url.trim()}
            className="px-6 py-3 bg-primary-600 hover:bg-primary-700 disabled:bg-primary-800 disabled:cursor-not-allowed text-white rounded-xl transition-colors flex items-center gap-2"
          >
            {isLoading ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Play className="w-5 h-5" />
            )}
            <span>{isLoading ? "Parsing..." : "Parse"}</span>
          </button>
        </div>
      </form>

      {/* Error Message */}
      {error && (
        <div className="mb-6 p-4 bg-red-600/20 border border-red-600/50 rounded-xl flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
          <div>
            <p className="text-red-400 font-medium">Error</p>
            <p className="text-red-300 text-sm">{error}</p>
          </div>
        </div>
      )}

      {/* Video Preview */}
      {video && (
        <div className="bg-dark-800 rounded-xl overflow-hidden">
          {/* Cover Image */}
          {video.cover && (
            <div className="relative aspect-video">
              <img
                src={video.cover}
                alt={video.title}
                className="w-full h-full object-cover"
              />
              <div className="absolute inset-0 bg-gradient-to-t from-black/80 to-transparent" />
              <button
                onClick={handlePlay}
                className="absolute inset-0 flex items-center justify-center"
              >
                <div className="w-16 h-16 bg-white/20 backdrop-blur-sm rounded-full flex items-center justify-center hover:bg-white/30 transition-colors">
                  <Play className="w-8 h-8 text-white fill-white ml-1" />
                </div>
              </button>
            </div>
          )}

          {/* Video Info */}
          <div className="p-6">
            <h2 className="text-lg font-semibold text-white mb-2">
              {video.title}
            </h2>

            <div className="flex items-center gap-2 mb-4 text-dark-300">
              <span>@{video.author}</span>
              {video.duration > 0 && (
                <>
                  <span>•</span>
                  <span>{Math.floor(video.duration)}s</span>
                </>
              )}
            </div>

            {video.description && (
              <p className="text-dark-400 text-sm mb-4 line-clamp-3">
                {video.description}
              </p>
            )}

            <div className="flex items-center gap-4">
              <button
                onClick={handlePlay}
                className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
              >
                <Play className="w-5 h-5 fill-white" />
                <span>Play Video</span>
              </button>

              <a
                href={video.share_url}
                target="_blank"
                rel="noopener noreferrer"
                className="px-4 py-3 bg-dark-700 hover:bg-dark-600 text-white rounded-lg transition-colors"
              >
                Open in Douyin
              </a>
            </div>
          </div>
        </div>
      )}

      {/* Usage Tips */}
      <div className="mt-8 bg-dark-800 rounded-xl p-6">
        <h3 className="text-lg font-semibold text-white mb-4">How to use</h3>
        <div className="space-y-3 text-dark-300">
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-primary-600 rounded-full flex items-center justify-center text-white text-sm flex-shrink-0">
              1
            </span>
            <p>Open Douyin app and find the video you want to watch</p>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-primary-600 rounded-full flex items-center justify-center text-white text-sm flex-shrink-0">
              2
            </span>
            <p>Tap "Share" and copy the link</p>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-primary-600 rounded-full flex items-center justify-center text-white text-sm flex-shrink-0">
              3
            </span>
            <p>Paste the link here and click "Parse"</p>
          </div>
          <div className="flex items-start gap-3">
            <span className="w-6 h-6 bg-primary-600 rounded-full flex items-center justify-center text-white text-sm flex-shrink-0">
              4
            </span>
            <p>Enjoy the video without watermark!</p>
          </div>
        </div>

        <div className="mt-4 p-3 bg-dark-700 rounded-lg">
          <p className="text-sm text-dark-400">
            <strong className="text-dark-300">Supported formats:</strong>
            <br />
            • Share links: <code className="text-primary-400">https://v.douyin.com/xxxxx</code>
            <br />
            • Video URLs: <code className="text-primary-400">https://www.douyin.com/video/xxxxx</code>
            <br />
            • Share text with embedded links
          </p>
        </div>
      </div>

      {/* Player Modal */}
      {showPlayer && video && (
        <DouyinPlayer video={video} onClose={handleClosePlayer} />
      )}
    </div>
  );
}
