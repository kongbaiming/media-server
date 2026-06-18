// Online / live stream page. Accepts an m3u8 / mp4 / webm / ts URL, probes
// it via the backend, and plays it through StreamPlayer.

import { useEffect, useState } from "react";
import { ArrowRight, Loader2, Radio, Trash2 } from "lucide-react";
import {
  getOnlineRecent,
  onlineStreamUrl,
  probeOnline,
} from "@/services/api";
import type { OnlineRecentItem, ProbeResult } from "@/types";
import StreamPlayer from "@/components/StreamPlayer/StreamPlayer";

export default function Online() {
  const [url, setUrl] = useState("");
  const [referer, setReferer] = useState("");
  const [probe, setProbe] = useState<ProbeResult | null>(null);
  const [probing, setProbing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [recent, setRecent] = useState<OnlineRecentItem[]>([]);
  const [activeUrl, setActiveUrl] = useState<string | null>(null);
  const [activeHls, setActiveHls] = useState(false);
  const [activeTitle, setActiveTitle] = useState("");

  useEffect(() => {
    getOnlineRecent()
      .then((r) => setRecent(r.data ?? []))
      .catch(() => setRecent([]));
  }, []);

  const handlePlay = async (overrideUrl?: string) => {
    const target = (overrideUrl ?? url).trim();
    if (!target) {
      setError("Please enter a URL");
      return;
    }
    setError(null);
    setProbing(true);
    setProbe(null);
    try {
      const resp = await probeOnline(target, referer.trim() || undefined);
      if (!resp.success || !resp.data) {
        setError(resp.error || "Probe failed");
        return;
      }
      const info = resp.data;
      setProbe(info);
      setActiveUrl(target);
      setActiveHls(info.kind === "hls");
      setActiveTitle(info.content_type || target);
    } catch (e) {
      setError(String(e));
    } finally {
      setProbing(false);
    }
  };

  const isPlaying = activeUrl !== null;

  if (isPlaying) {
    return (
      <StreamPlayer
        title={activeTitle}
        src={onlineStreamUrl(activeUrl!, referer.trim() || undefined)}
        isHls={activeHls}
        onBack={() => {
          setActiveUrl(null);
          setProbe(null);
        }}
      />
    );
  }

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <h1 className="text-2xl font-bold mb-2 flex items-center gap-2">
        <Radio className="w-6 h-6 text-primary-400" />
        Online & Live Streams
      </h1>
      <p className="text-sm text-dark-400 mb-6">
        Paste any m3u8 (HLS), mp4, webm, or ts URL. The backend proxies the
        stream through localhost so CORS / Referer restrictions on the
        origin server don't bite the in-app player.
      </p>

      <div className="bg-dark-900 border border-dark-800 rounded-lg p-4 space-y-3">
        <label className="block text-xs uppercase tracking-wider text-dark-400">
          Stream URL
        </label>
        <div className="flex gap-2">
          <input
            type="url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://example.com/live.m3u8"
            className="flex-1 bg-dark-950 border border-dark-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
            onKeyDown={(e) => {
              if (e.key === "Enter") handlePlay();
            }}
          />
          <button
            type="button"
            onClick={() => handlePlay()}
            disabled={probing || !url.trim()}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:bg-dark-700 disabled:text-dark-400 rounded text-sm font-medium"
          >
            {probing ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <ArrowRight className="w-4 h-4" />
            )}
            Play
          </button>
        </div>
        <div>
          <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
            Referer <span className="text-dark-600">(optional, some CDNs require it)</span>
          </label>
          <input
            type="text"
            value={referer}
            onChange={(e) => setReferer(e.target.value)}
            placeholder="https://example.com/"
            className="w-full bg-dark-950 border border-dark-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
          />
        </div>
        {error && (
          <div className="text-red-400 text-sm bg-red-950/30 border border-red-900/40 rounded px-3 py-2">
            {error}
          </div>
        )}
        {probe && (
          <div className="text-xs text-dark-300 bg-dark-950 border border-dark-800 rounded p-3 space-y-1">
            <div>
              <span className="text-dark-500">kind:</span>{" "}
              <span className="text-primary-400">{probe.kind}</span>
            </div>
            <div>
              <span className="text-dark-500">content-type:</span>{" "}
              {probe.content_type ?? "?"}
            </div>
            <div>
              <span className="text-dark-500">size:</span>{" "}
              {probe.content_length
                ? `${(probe.content_length / 1_048_576).toFixed(2)} MB`
                : "?"}{" "}
              <span className="text-dark-500 ml-3">range:</span>{" "}
              {probe.accepts_ranges ? "yes" : "no"}
            </div>
          </div>
        )}
      </div>

      <div className="mt-8">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-dark-400 mb-2">
          Recently played
        </h2>
        {recent.length === 0 ? (
          <div className="text-sm text-dark-500">No online streams yet.</div>
        ) : (
          <ul className="space-y-1">
            {recent.map((item) => (
              <li
                key={item.url}
                className="flex items-center gap-2 bg-dark-900 border border-dark-800 rounded px-3 py-2"
              >
                <button
                  type="button"
                  onClick={() => handlePlay(item.url)}
                  className="flex-1 text-left truncate text-sm hover:text-primary-400"
                >
                  {item.title || item.url}
                </button>
                <button
                  type="button"
                  onClick={() =>
                    setRecent((r) => r.filter((x) => x.url !== item.url))
                  }
                  className="text-dark-500 hover:text-red-400"
                  aria-label="Remove from list"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
