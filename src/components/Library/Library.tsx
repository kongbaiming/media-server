import { useEffect } from "react";
import { useMediaStore } from "@/stores/mediaStore";
import MediaGrid from "./MediaGrid";
import MediaDetail from "./MediaDetail";
import { Loader2 } from "lucide-react";

export default function Library() {
  const {
    library,
    isLoading,
    error,
    selectedMedia,
    selectMedia,
    fetchLibrary,
    fetchStatistics,
    statistics,
  } = useMediaStore();

  useEffect(() => {
    fetchLibrary();
    fetchStatistics();
  }, [fetchLibrary, fetchStatistics]);

  if (isLoading && library.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 text-primary-400 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <p className="text-red-400 mb-4">{error}</p>
        <button
          onClick={fetchLibrary}
          className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg"
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Stats Bar */}
      {statistics && (
        <div className="flex gap-4 mb-6">
          <div className="bg-dark-800 rounded-lg px-4 py-3 flex-1">
            <p className="text-sm text-dark-400">Total Media</p>
            <p className="text-2xl font-bold text-white">
              {statistics.total_files}
            </p>
          </div>
          <div className="bg-dark-800 rounded-lg px-4 py-3 flex-1">
            <p className="text-sm text-dark-400">Videos</p>
            <p className="text-2xl font-bold text-primary-400">
              {statistics.video_count}
            </p>
          </div>
          <div className="bg-dark-800 rounded-lg px-4 py-3 flex-1">
            <p className="text-sm text-dark-400">Music</p>
            <p className="text-2xl font-bold text-green-400">
              {statistics.audio_count}
            </p>
          </div>
          <div className="bg-dark-800 rounded-lg px-4 py-3 flex-1">
            <p className="text-sm text-dark-400">Total Size</p>
            <p className="text-2xl font-bold text-purple-400">
              {statistics.total_size
                ? `${(statistics.total_size / (1024 * 1024 * 1024)).toFixed(
                    1
                  )} GB`
                : "0 GB"}
            </p>
          </div>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-hidden flex gap-6">
        <div className="flex-1 overflow-auto">
          {library.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-dark-400">
              <p className="text-xl mb-2">No media found</p>
              <p className="text-sm">
                Add a folder to your library to get started
              </p>
            </div>
          ) : (
            <MediaGrid
              media={library}
              onSelect={selectMedia}
              selectedId={selectedMedia?.id}
            />
          )}
        </div>

        {/* Detail Panel */}
        {selectedMedia && (
          <div className="w-96 overflow-auto">
            <MediaDetail
              media={selectedMedia}
              onClose={() => selectMedia(null)}
            />
          </div>
        )}
      </div>
    </div>
  );
}
