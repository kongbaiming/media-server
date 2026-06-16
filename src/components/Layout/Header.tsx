import { Search, RefreshCw, FolderOpen } from "lucide-react";
import { useMediaStore } from "@/stores/mediaStore";
import { useState } from "react";
import { useNavigate } from "react-router-dom";

export default function Header() {
  const [searchQuery, setSearchQuery] = useState("");
  const navigate = useNavigate();
  const { scanLibrary, isScanning, config } = useMediaStore();

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (searchQuery.trim()) {
      navigate(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  };

  const handleScan = () => {
    if (config?.library_paths && config.library_paths.length > 0) {
      scanLibrary(config.library_paths);
    }
  };

  return (
    <header className="h-16 bg-dark-900 border-b border-dark-800 flex items-center justify-between px-6">
      {/* Search */}
      <form onSubmit={handleSearch} className="flex-1 max-w-md">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-dark-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search media..."
            className="w-full pl-10 pr-4 py-2 bg-dark-800 border border-dark-700 rounded-lg text-white placeholder-dark-400 focus:outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
          />
        </div>
      </form>

      {/* Actions */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleScan}
          disabled={isScanning}
          className="flex items-center gap-2 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:bg-primary-800 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
        >
          <RefreshCw
            className={`w-4 h-4 ${isScanning ? "animate-spin" : ""}`}
          />
          <span>{isScanning ? "Scanning..." : "Scan Library"}</span>
        </button>

        <button className="flex items-center gap-2 px-4 py-2 bg-dark-800 hover:bg-dark-700 text-white rounded-lg transition-colors">
          <FolderOpen className="w-4 h-4" />
          <span>Add Folder</span>
        </button>
      </div>
    </header>
  );
}
