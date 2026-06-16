import { useState, useEffect } from "react";
import {
  FolderOpen,
  Trash2,
  Plus,
  Save,
  RefreshCw,
  HardDrive,
  Server,
} from "lucide-react";
import { useMediaStore } from "@/stores/mediaStore";
import type { AppConfig } from "@/types";

export default function Settings() {
  const { config, updateConfig, fetchConfig, scanLibrary } = useMediaStore();
  const [editedConfig, setEditedConfig] = useState<Partial<AppConfig>>({});
  const [newPath, setNewPath] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

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
    <div className="max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold text-white mb-6">Settings</h1>

      {/* Library Paths */}
      <div className="bg-dark-800 rounded-xl p-6 mb-6">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <FolderOpen className="w-5 h-5" />
          Library Folders
        </h2>

        <div className="space-y-3 mb-4">
          {editedConfig.library_paths?.map((path, index) => (
            <div
              key={index}
              className="flex items-center gap-3 p-3 bg-dark-700 rounded-lg"
            >
              <FolderOpen className="w-4 h-4 text-dark-400 flex-shrink-0" />
              <span className="flex-1 text-sm text-white truncate">
                {path}
              </span>
              <button
                onClick={() => handleRemovePath(index)}
                className="p-1 hover:bg-dark-600 rounded transition-colors"
              >
                <Trash2 className="w-4 h-4 text-red-400" />
              </button>
            </div>
          ))}
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={newPath}
            onChange={(e) => setNewPath(e.target.value)}
            placeholder="Enter folder path..."
            className="flex-1 px-4 py-2 bg-dark-700 border border-dark-600 rounded-lg text-white placeholder-dark-400 focus:outline-none focus:border-primary-500"
            onKeyDown={(e) => e.key === "Enter" && handleAddPath()}
          />
          <button
            onClick={handleAddPath}
            className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors flex items-center gap-2"
          >
            <Plus className="w-4 h-4" />
            Add
          </button>
        </div>
      </div>

      {/* Server Settings */}
      <div className="bg-dark-800 rounded-xl p-6 mb-6">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Server className="w-5 h-5" />
          Server Settings
        </h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-dark-400 mb-2">
              Server Port
            </label>
            <input
              type="number"
              value={editedConfig.server_port || config.server_port}
              onChange={(e) =>
                setEditedConfig({
                  ...editedConfig,
                  server_port: parseInt(e.target.value),
                })
              }
              className="w-full px-4 py-2 bg-dark-700 border border-dark-600 rounded-lg text-white focus:outline-none focus:border-primary-500"
            />
          </div>

          <div className="flex items-center justify-between">
            <div>
              <p className="text-white">Auto Scan</p>
              <p className="text-sm text-dark-400">
                Automatically scan library on startup
              </p>
            </div>
            <button
              onClick={() =>
                setEditedConfig({
                  ...editedConfig,
                  auto_scan: !editedConfig.auto_scan,
                })
              }
              className={`w-12 h-6 rounded-full transition-colors ${
                editedConfig.auto_scan ? "bg-primary-600" : "bg-dark-600"
              }`}
            >
              <div
                className={`w-5 h-5 bg-white rounded-full transition-transform ${
                  editedConfig.auto_scan
                    ? "translate-x-6"
                    : "translate-x-0.5"
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Storage */}
      <div className="bg-dark-800 rounded-xl p-6 mb-6">
        <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <HardDrive className="w-5 h-5" />
          Storage
        </h2>

        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-dark-700 rounded-lg">
            <div>
              <p className="text-white">Thumbnails Cache</p>
              <p className="text-sm text-dark-400">
                Cached video thumbnails
              </p>
            </div>
            <button className="px-3 py-1 text-sm bg-dark-600 hover:bg-dark-500 text-white rounded transition-colors">
              Clear
            </button>
          </div>

          <div className="flex items-center justify-between p-3 bg-dark-700 rounded-lg">
            <div>
              <p className="text-white">Transcode Cache</p>
              <p className="text-sm text-dark-400">
                HLS segments and transcoded files
              </p>
            </div>
            <button className="px-3 py-1 text-sm bg-dark-600 hover:bg-dark-500 text-white rounded transition-colors">
              Clear
            </button>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="flex gap-3">
        <button
          onClick={handleSave}
          disabled={isSaving}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 hover:bg-primary-700 disabled:bg-primary-800 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
        >
          <Save className="w-4 h-4" />
          <span>{isSaving ? "Saving..." : "Save Settings"}</span>
        </button>

        <button
          onClick={handleScanNow}
          className="flex items-center justify-center gap-2 px-4 py-3 bg-dark-700 hover:bg-dark-600 text-white rounded-lg transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
          <span>Scan Now</span>
        </button>
      </div>
    </div>
  );
}
