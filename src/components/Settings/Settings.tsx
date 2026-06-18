import { useState, useEffect } from "react";
import {
  FolderOpen,
  Trash2,
  Plus,
  Save,
  RefreshCw,
  HardDrive,
  Key,
  Database,
  Loader2,
} from "lucide-react";
import { useMediaStore } from "@/stores/mediaStore";
import type { AppConfig, ScraperStatus } from "@/types";
import {
  getScraperStatus,
  refreshAllScrapes,
  setScraperKey,
  suggestSynologyPath,
} from "@/services/api";

type Tab = "library" | "metadata" | "synology";

export default function Settings() {
  const { config, updateConfig, scanLibrary } = useMediaStore();
  const [tab, setTab] = useState<Tab>("library");
  const [editedConfig, setEditedConfig] = useState<Partial<AppConfig>>({});
  const [newPath, setNewPath] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (config) {
      setEditedConfig({
        library_paths: [...config.library_paths],
        auto_scan: config.auto_scan,
        server_port: config.server_port,
      });
    }
  }, [config]);

  const handleAddPath = () => {
    if (newPath.trim() && editedConfig.library_paths) {
      setEditedConfig({
        ...editedConfig,
        library_paths: [...editedConfig.library_paths, newPath.trim()],
      });
      setNewPath("");
    }
  };

  const handleRemovePath = (index: number) => {
    if (editedConfig.library_paths) {
      setEditedConfig({
        ...editedConfig,
        library_paths: editedConfig.library_paths.filter((_, i) => i !== index),
      });
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await updateConfig(editedConfig);
    } finally {
      setIsSaving(false);
    }
  };

  const handleScanNow = () => {
    if (editedConfig.library_paths && editedConfig.library_paths.length > 0) {
      scanLibrary(editedConfig.library_paths);
    }
  };

  if (!config) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-dark-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h1 className="text-2xl font-bold text-white mb-4">Settings</h1>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 border-b border-dark-800">
        {(
          [
            { id: "library", label: "Library", icon: FolderOpen },
            { id: "metadata", label: "Metadata", icon: Database },
            { id: "synology", label: "Synology NAS", icon: HardDrive },
          ] as { id: Tab; label: string; icon: any }[]
        ).map((t) => (
          <button
            key={t.id}
            onClick={() => setTab(t.id)}
            className={
              "flex items-center gap-2 px-4 py-2 -mb-px border-b-2 text-sm font-medium " +
              (tab === t.id
                ? "border-primary-500 text-white"
                : "border-transparent text-dark-400 hover:text-white")
            }
          >
            <t.icon className="w-4 h-4" />
            {t.label}
          </button>
        ))}
      </div>

      {tab === "library" && (
        <LibraryTab
          editedConfig={editedConfig}
          setEditedConfig={setEditedConfig}
          newPath={newPath}
          setNewPath={setNewPath}
          handleAddPath={handleAddPath}
          handleRemovePath={handleRemovePath}
          handleSave={handleSave}
          handleScanNow={handleScanNow}
          isSaving={isSaving}
        />
      )}
      {tab === "metadata" && <MetadataTab />}
      {tab === "synology" && <SynologyTab />}
    </div>
  );
}

function LibraryTab(props: {
  editedConfig: Partial<AppConfig>;
  setEditedConfig: (c: Partial<AppConfig>) => void;
  newPath: string;
  setNewPath: (s: string) => void;
  handleAddPath: () => void;
  handleRemovePath: (i: number) => void;
  handleSave: () => Promise<void>;
  handleScanNow: () => void;
  isSaving: boolean;
}) {
  return (
    <div className="bg-dark-800 rounded-xl p-6 mb-6">
      <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <FolderOpen className="w-5 h-5" />
        Library Folders
      </h2>

      <div className="space-y-3 mb-4">
        {props.editedConfig.library_paths?.map((path, index) => (
          <div
            key={index}
            className="flex items-center gap-2 bg-dark-900 rounded px-3 py-2"
          >
            <HardDrive className="w-4 h-4 text-dark-400" />
            <code className="flex-1 text-sm text-dark-200 truncate">{path}</code>
            <button
              type="button"
              onClick={() => props.handleRemovePath(index)}
              className="text-dark-400 hover:text-red-400"
              aria-label="Remove path"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          </div>
        ))}
      </div>

      <div className="flex gap-2">
        <input
          type="text"
          value={props.newPath}
          onChange={(e) => props.setNewPath(e.target.value)}
          placeholder="\\server\share\Movies  or  D:\Videos"
          className="flex-1 bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
          onKeyDown={(e) => {
            if (e.key === "Enter") props.handleAddPath();
          }}
        />
        <button
          type="button"
          onClick={props.handleAddPath}
          className="flex items-center gap-1 px-3 py-2 bg-primary-600 hover:bg-primary-700 rounded text-sm"
        >
          <Plus className="w-4 h-4" />
          Add
        </button>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-3">
        <label className="flex items-center gap-2 text-sm text-dark-300">
          <input
            type="checkbox"
            checked={!!props.editedConfig.auto_scan}
            onChange={(e) =>
              props.setEditedConfig({
                ...props.editedConfig,
                auto_scan: e.target.checked,
              })
            }
            className="w-4 h-4 rounded"
          />
          Auto-scan on startup
        </label>
        <div className="flex items-center gap-2 text-sm text-dark-300 justify-end">
          <span>Port:</span>
          <input
            type="number"
            value={props.editedConfig.server_port ?? 8080}
            onChange={(e) =>
              props.setEditedConfig({
                ...props.editedConfig,
                server_port: parseInt(e.target.value, 10),
              })
            }
            className="w-20 bg-dark-900 border border-dark-700 rounded px-2 py-1"
          />
        </div>
      </div>

      <div className="mt-6 flex gap-2 justify-end">
        <button
          type="button"
          onClick={props.handleScanNow}
          className="flex items-center gap-1 px-3 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm"
        >
          <RefreshCw className="w-4 h-4" />
          Scan now
        </button>
        <button
          type="button"
          onClick={props.handleSave}
          disabled={props.isSaving}
          className="flex items-center gap-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm font-medium"
        >
          {props.isSaving ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <Save className="w-4 h-4" />
          )}
          Save
        </button>
      </div>
    </div>
  );
}

function MetadataTab() {
  const [status, setStatus] = useState<ScraperStatus | null>(null);
  const [apiKey, setApiKey] = useState<string>("");
  
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setError(null);
    try {
      const r = await getScraperStatus();
      if (r.success && r.data) {
        setStatus(r.data);
        
      }
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleSaveKey = async () => {
    setBusy(true);
    setError(null);
    try {
      const r = await setScraperKey(apiKey.trim() || null);
      if (!r.success) {
        setError(r.error || "Failed to set key");
      } else {
        setApiKey("");
        await refresh();
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRefreshAll = async () => {
    setBusy(true);
    setError(null);
    try {
      const r = await refreshAllScrapes();
      if (!r.success) setError(r.error || "Failed to enqueue");
      else await refresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="bg-dark-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold text-white mb-1 flex items-center gap-2">
          <Database className="w-5 h-5" />
          TMDB Scraper
        </h2>
        <p className="text-sm text-dark-400 mb-4">
          Auto-fetch posters, plot, cast, and franchise data from{" "}
          <a
            className="text-primary-400 hover:underline"
            href="https://www.themoviedb.org/settings/api"
            target="_blank"
            rel="noreferrer"
          >
            The Movie Database
          </a>
          . Free API key required.
        </p>

        <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
          API key (v3 auth)
        </label>
        <div className="flex gap-2">
          <input
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="tmdb api v3 key"
            className="flex-1 bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
          />
          <button
            type="button"
            onClick={handleSaveKey}
            disabled={busy}
            className="flex items-center gap-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm font-medium"
          >
            {busy ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            Save
          </button>
        </div>

        {error && (
          <div className="mt-3 text-sm text-red-400 bg-red-950/30 border border-red-900/40 rounded px-3 py-2">
            {error}
          </div>
        )}

        <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
          <div className="bg-dark-900 rounded p-3">
            <div className="text-xs text-dark-400">Status</div>
            <div className="text-white">
              {status?.enabled ? (
                <span className="text-green-400">Enabled</span>
              ) : (
                <span className="text-dark-500">No API key set</span>
              )}
            </div>
          </div>
          <div className="bg-dark-900 rounded p-3">
            <div className="text-xs text-dark-400">Queue</div>
            <div className="text-white">
              {status?.queue_len ?? 0} pending
            </div>
          </div>
          <div className="bg-dark-900 rounded p-3">
            <div className="text-xs text-dark-400">Scraped</div>
            <div className="text-white">{status?.scraped ?? 0}</div>
          </div>
          <div className="bg-dark-900 rounded p-3">
            <div className="text-xs text-dark-400">Failed</div>
            <div className={(status?.failed ?? 0) > 0 ? "text-red-400" : "text-white"}>
              {status?.failed ?? 0}
            </div>
          </div>
        </div>

        {status?.last_error && (
          <div className="mt-3 text-xs text-red-400 bg-dark-900 rounded px-3 py-2">
            Last error: {status.last_error}
          </div>
        )}

        <div className="mt-4 flex gap-2">
          <button
            type="button"
            onClick={handleRefreshAll}
            disabled={busy || !status?.enabled}
            className="flex items-center gap-1 px-3 py-2 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 rounded text-sm"
          >
            <RefreshCw className="w-4 h-4" />
            Refresh all
          </button>
          <button
            type="button"
            onClick={refresh}
            className="flex items-center gap-1 px-3 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm"
          >
            Refresh status
          </button>
        </div>
      </div>
    </div>
  );
}

function SynologyTab() {
  const [qc, setQc] = useState("");
  const [host, setHost] = useState("");
  const [share, setShare] = useState("");
  const [label, setLabel] = useState("");
  const [suggestedPath, setSuggestedPath] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const handleSuggest = async () => {
    if (!qc.trim() || !share.trim()) return;
    setBusy(true);
    try {
      const r = await suggestSynologyPath({
        quickconnect_id: qc.trim(),
        host: host.trim() || undefined,
        share: share.trim(),
        label: label.trim() || undefined,
      });
      if (r.success && r.data) {
        setSuggestedPath(r.data.path);
      }
    } finally {
      setBusy(false);
    }
  };

  const handleCopy = () => {
    if (suggestedPath) navigator.clipboard.writeText(suggestedPath);
  };

  return (
    <div className="space-y-4">
      <div className="bg-dark-800 rounded-xl p-6">
        <h2 className="text-lg font-semibold text-white mb-1 flex items-center gap-2">
          <HardDrive className="w-5 h-5" />
          Synology NAS
        </h2>
        <p className="text-sm text-dark-400 mb-4">
          Build a UNC path to a Synology SMB share. Use the QuickConnect ID
          (e.g. the part before{" "}
          <code className="text-dark-200">.quickconnect.to</code>) and the
          share name. If you can reach the NAS on your LAN, set the host too
          so MediaVault never has to go through the QuickConnect relay.
        </p>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
              QuickConnect ID
            </label>
            <input
              type="text"
              value={qc}
              onChange={(e) => setQc(e.target.value)}
              placeholder="myds"
              className="w-full bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
              LAN host (optional)
            </label>
            <input
              type="text"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              placeholder="192.168.1.100 or nas.local"
              className="w-full bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
              Share name
            </label>
            <input
              type="text"
              value={share}
              onChange={(e) => setShare(e.target.value)}
              placeholder="data"
              className="w-full bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm"
            />
          </div>
          <div>
            <label className="block text-xs uppercase tracking-wider text-dark-400 mb-1">
              Label (optional)
            </label>
            <input
              type="text"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder="Movies"
              className="w-full bg-dark-900 border border-dark-700 rounded px-3 py-2 text-sm"
            />
          </div>
        </div>

        <button
          type="button"
          onClick={handleSuggest}
          disabled={busy || !qc.trim() || !share.trim()}
          className="mt-4 flex items-center gap-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm font-medium"
        >
          {busy ? <Loader2 className="w-4 h-4 animate-spin" /> : <Key className="w-4 h-4" />}
          Build UNC path
        </button>

        {suggestedPath && (
          <div className="mt-4 bg-dark-900 rounded p-3">
            <div className="text-xs text-dark-400 mb-1">Add this path to your library:</div>
            <div className="flex items-center gap-2">
              <code className="flex-1 text-sm text-green-400 break-all">
                {suggestedPath}
              </code>
              <button
                type="button"
                onClick={handleCopy}
                className="px-2 py-1 bg-dark-700 hover:bg-dark-600 rounded text-xs"
              >
                Copy
              </button>
            </div>
            <div className="mt-2 text-xs text-dark-500">
              Copy the path above, then go to the Library tab and paste it into &quot;Add path&quot;.
            </div>
          </div>
        )}
      </div>
    </div>
  );
}



