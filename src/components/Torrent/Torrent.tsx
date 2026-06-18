// Torrents / magnet links page. Adds a magnet (or uploaded .torrent) to
// the backend, polls status, lets the user play the partially-downloaded
// stream.

import { useEffect, useRef, useState } from "react";
import {
  ArrowRight,
  Download,
  Loader2,
  Magnet,
  Play,
  Trash2,
  Upload,
} from "lucide-react";
import {
  addTorrent,
  deleteTorrent,
  listTorrents,
} from "@/services/api";
import type { TorrentSessionInfo } from "@/types";
import StreamPlayer from "@/components/StreamPlayer/StreamPlayer";

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1_048_576).toFixed(1)} MB`;
  return `${(n / 1_073_741_824).toFixed(2)} GB`;
}

function formatSpeed(bps: number): string {
  if (bps < 1024) return `${bps} B/s`;
  if (bps < 1024 * 1024) return `${(bps / 1024).toFixed(1)} KB/s`;
  return `${(bps / 1_048_576).toFixed(2)} MB/s`;
}

export default function Torrent() {
  const [magnet, setMagnet] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sessions, setSessions] = useState<TorrentSessionInfo[]>([]);
  const [active, setActive] = useState<TorrentSessionInfo | null>(null);
  const fileInput = useRef<HTMLInputElement>(null);

  const refresh = async () => {
    const r = await listTorrents();
    if (r.success && r.data) setSessions(r.data);
  };

  useEffect(() => {
    refresh();
  }, []);

  useEffect(() => {
    if (sessions.length === 0) return;
    const id = setInterval(() => {
      refresh();
    }, 2000);
    return () => clearInterval(id);
  }, [sessions.length]);

  const handleAdd = async () => {
    setError(null);
    const uri = magnet.trim();
    if (!uri) {
      setError("Paste a magnet URI or upload a .torrent file");
      return;
    }
    if (!uri.startsWith("magnet:")) {
      setError("Input must be a magnet URI (starting with magnet:?)");
      return;
    }
    setSubmitting(true);
    try {
      const r = await addTorrent({ magnet: uri });
      if (!r.success) {
        setError(r.error || "Failed to add torrent");
        return;
      }
      setMagnet("");
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const handleFile = async (file: File) => {
    setError(null);
    setSubmitting(true);
    try {
      const buf = await file.arrayBuffer();
      const bytes = new Uint8Array(buf);
      let bin = "";
      for (let i = 0; i < bytes.byteLength; i++) {
        bin += String.fromCharCode(bytes[i]);
      }
      const b64 = btoa(bin);
      const r = await addTorrent({ torrent_b64: b64 });
      if (!r.success) {
        setError(r.error || "Failed to add torrent");
        return;
      }
      await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    await deleteTorrent(id);
    if (active?.id === id) setActive(null);
    await refresh();
  };

  if (active) {
    return (
      <StreamPlayer
        title={active.name}
        src={active.stream_url}
        isHls={false}
        onBack={() => setActive(null)}
      />
    );
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-2 flex items-center gap-2">
        <Magnet className="w-6 h-6 text-primary-400" />
        Torrents & Magnet Links
      </h1>
      <p className="text-sm text-dark-400 mb-6">
        Paste a magnet URI or upload a <code>.torrent</code> file. The
        backend fetches metadata from public caches (itorrents.org,
        btcache.me), then downloads pieces from HTTP web seeds (BEP 17/19)
        and streams the result as pieces arrive. Peer-only / DHT-only
        torrents and private torrents that aren't in any cache are not
        supported.
      </p>

      <div className="bg-dark-900 border border-dark-800 rounded-lg p-4 space-y-3">
        <label className="block text-xs uppercase tracking-wider text-dark-400">
          Magnet URI
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            value={magnet}
            onChange={(e) => setMagnet(e.target.value)}
            placeholder="magnet:?xt=urn:btih:..."
            className="flex-1 bg-dark-950 border border-dark-700 rounded px-3 py-2 text-sm font-mono focus:outline-none focus:border-primary-500"
            onKeyDown={(e) => {
              if (e.key === "Enter") handleAdd();
            }}
          />
          <button
            type="button"
            onClick={handleAdd}
            disabled={submitting}
            className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:bg-dark-700 disabled:text-dark-400 rounded text-sm font-medium"
          >
            {submitting ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <ArrowRight className="w-4 h-4" />
            )}
            Add
          </button>
          <button
            type="button"
            onClick={() => fileInput.current?.click()}
            disabled={submitting}
            className="flex items-center gap-2 px-3 py-2 bg-dark-800 hover:bg-dark-700 disabled:opacity-50 rounded text-sm"
            title="Upload .torrent file"
          >
            <Upload className="w-4 h-4" />
          </button>
          <input
            ref={fileInput}
            type="file"
            accept=".torrent,application/x-bittorrent"
            className="hidden"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) handleFile(f);
              e.target.value = "";
            }}
          />
        </div>
        {error && (
          <div className="text-red-400 text-sm bg-red-950/30 border border-red-900/40 rounded px-3 py-2">
            {error}
          </div>
        )}
      </div>

      <div className="mt-8 space-y-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-dark-400">
          Active sessions
        </h2>
        {sessions.length === 0 ? (
          <div className="text-sm text-dark-500">
            No torrents yet. Paste a magnet above to get started.
          </div>
        ) : (
          sessions.map((s) => (
            <div
              key={s.id}
              className="bg-dark-900 border border-dark-800 rounded-lg p-4"
            >
              <div className="flex items-start gap-3">
                <div className="flex-1 min-w-0">
                  <div className="font-medium truncate">{s.name}</div>
                  <div className="text-xs text-dark-400 font-mono truncate">
                    {s.info_hash}
                  </div>
                  <div className="mt-2 flex items-center gap-3 text-xs">
                    <span
                      className={
                        s.status === "ready"
                          ? "text-green-400"
                          : s.status === "failed"
                          ? "text-red-400"
                          : "text-primary-400"
                      }
                    >
                      {s.status}
                    </span>
                    <span className="text-dark-400">
                      {formatBytes(s.downloaded)} / {formatBytes(s.total)} (
                      {s.progress.toFixed(1)}%)
                    </span>
                    {s.download_speed_bps > 0 && (
                      <span className="text-dark-500">
                        {formatSpeed(s.download_speed_bps)}
                      </span>
                    )}
                  </div>
                  <div className="mt-2 h-1.5 bg-dark-800 rounded overflow-hidden">
                    <div
                      className="h-full bg-primary-500 transition-all"
                      style={{ width: `${Math.min(100, s.progress)}%` }}
                    />
                  </div>
                  {s.error && (
                    <div className="mt-2 text-xs text-red-400">{s.error}</div>
                  )}
                </div>
                <div className="flex flex-col gap-1">
                  <button
                    type="button"
                    onClick={() => setActive(s)}
                    disabled={s.total === 0}
                    className="flex items-center gap-1 px-3 py-1.5 bg-primary-600 hover:bg-primary-700 disabled:bg-dark-700 disabled:text-dark-400 rounded text-xs font-medium"
                  >
                    <Play className="w-3 h-3" />
                    Play
                  </button>
                  <button
                    type="button"
                    onClick={() => handleDelete(s.id)}
                    className="flex items-center gap-1 px-3 py-1.5 bg-dark-800 hover:bg-red-900/40 rounded text-xs"
                  >
                    <Trash2 className="w-3 h-3" />
                    Remove
                  </button>
                </div>
              </div>
              {s.files.length > 0 && (
                <details className="mt-2 text-xs text-dark-400">
                  <summary className="cursor-pointer hover:text-dark-300">
                    {s.files.length} file{s.files.length === 1 ? "" : "s"}
                  </summary>
                  <ul className="mt-2 space-y-0.5 font-mono">
                    {s.files.map((f, i) => (
                      <li key={i} className="truncate">
                        {f.path} ({formatBytes(f.length)})
                      </li>
                    ))}
                  </ul>
                </details>
              )}
            </div>
          ))
        )}
      </div>

      <div className="mt-6 text-xs text-dark-500 flex items-start gap-2">
        <Download className="w-4 h-4 flex-shrink-0 mt-0.5" />
        <div>
          Pieces download with 3 concurrent HTTP range requests against
          the first responding web seed. SHA1 is verified per piece.
          Stream playback starts as soon as the first few MB land on disk.
          Downloaded data is kept under{" "}
          <code>~/.mediavault/torrents/</code>.
        </div>
      </div>
    </div>
  );
}
