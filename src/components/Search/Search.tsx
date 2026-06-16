import { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { Search as SearchIcon, X } from "lucide-react";
import type { MediaFile } from "@/types";
import { searchMedia } from "@/services/api";
import MediaGrid from "@/components/Library/MediaGrid";
import { useMediaStore } from "@/stores/mediaStore";

export default function Search() {
  const [searchParams, setSearchParams] = useSearchParams();
  const [query, setQuery] = useState(searchParams.get("q") || "");
  const [results, setResults] = useState<MediaFile[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const { selectMedia } = useMediaStore();

  useEffect(() => {
    const q = searchParams.get("q");
    if (q) {
      setQuery(q);
      performSearch(q);
    }
  }, [searchParams]);

  const performSearch = async (searchQuery: string) => {
    if (!searchQuery.trim()) return;

    setIsSearching(true);
    setHasSearched(true);

    try {
      const response = await searchMedia(searchQuery);
      if (response.success && response.data) {
        setResults(response.data);
      }
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      setSearchParams({ q: query.trim() });
      performSearch(query.trim());
    }
  };

  const handleClear = () => {
    setQuery("");
    setResults([]);
    setHasSearched(false);
    setSearchParams({});
  };

  return (
    <div className="h-full flex flex-col">
      {/* Search Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white mb-4">Search Media</h1>

        <form onSubmit={handleSearch} className="relative max-w-2xl">
          <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-dark-400" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search by name, path, or tags..."
            className="w-full pl-12 pr-12 py-3 bg-dark-800 border border-dark-700 rounded-xl text-white placeholder-dark-400 focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 text-lg"
          />
          {query && (
            <button
              type="button"
              onClick={handleClear}
              className="absolute right-4 top-1/2 -translate-y-1/2 p-1 hover:bg-dark-700 rounded-lg transition-colors"
            >
              <X className="w-5 h-5 text-dark-400" />
            </button>
          )}
        </form>
      </div>

      {/* Results */}
      <div className="flex-1 overflow-auto">
        {isSearching ? (
          <div className="flex items-center justify-center h-64">
            <div className="text-dark-400">Searching...</div>
          </div>
        ) : hasSearched ? (
          results.length > 0 ? (
            <div>
              <p className="text-sm text-dark-400 mb-4">
                Found {results.length} results for "{query}"
              </p>
              <MediaGrid
                media={results}
                onSelect={selectMedia}
              />
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 text-dark-400">
              <SearchIcon className="w-12 h-12 mb-4 opacity-50" />
              <p className="text-lg mb-2">No results found</p>
              <p className="text-sm">Try different keywords</p>
            </div>
          )
        ) : (
          <div className="flex flex-col items-center justify-center h-64 text-dark-400">
            <SearchIcon className="w-12 h-12 mb-4 opacity-50" />
            <p className="text-lg">Start searching</p>
            <p className="text-sm mt-2">
              Search by filename, path, or tags
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
