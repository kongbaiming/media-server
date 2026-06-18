// Collections view: groups movies by TMDB "belongs_to_collection" (e.g.
// "The Dark Knight Trilogy"). Each card is a poster collage of the
// movies in the collection, with the collection name and a count.

import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Layers, Star, Loader2, RefreshCw } from "lucide-react";
import { listCollections, tmdbImageUrl, refreshAllScrapes } from "@/services/api";
import type { MovieCollection } from "@/types";

export default function Collections() {
  const [collections, setCollections] = useState<MovieCollection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const navigate = useNavigate();

  const load = async () => {
    try {
      const r = await listCollections();
      if (r.success && r.data) setCollections(r.data);
      else setError(r.error || "Failed to load collections");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      await refreshAllScrapes();
      setTimeout(() => load(), 4000);
      setTimeout(() => load(), 10000);
    } finally {
      setRefreshing(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 text-primary-400 animate-spin" />
      </div>
    );
  }

  if (collections.length === 0) {
    return (
      <div className="p-6 max-w-4xl mx-auto">
        <h1 className="text-2xl font-bold text-white mb-2 flex items-center gap-2">
          <Layers className="w-6 h-6 text-primary-400" />
          Collections
        </h1>
        <p className="text-sm text-dark-400 mb-6">
          Movie franchises are detected automatically from the TMDB
          <code className="mx-1 text-dark-200">belongs_to_collection</code>
          field. Once a movie is scraped, it shows up here alongside its
          peers.
        </p>
        <div className="bg-dark-800 border border-dark-700 rounded-xl p-8 text-center">
          <p className="text-dark-300">
            {error ??
              "No collections yet. Add a TMDB API key in Settings -> Metadata and scrape your library."}
          </p>
          <button
            onClick={handleRefresh}
            disabled={refreshing}
            className="mt-4 inline-flex items-center gap-1 px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm"
          >
            {refreshing ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <RefreshCw className="w-4 h-4" />
            )}
            Scrape library
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white flex items-center gap-2">
            <Layers className="w-6 h-6 text-primary-400" />
            Collections
          </h1>
          <p className="text-sm text-dark-400 mt-1">
            {collections.length} franchise{collections.length === 1 ? "" : "s"} in your library.
          </p>
        </div>
        <button
          onClick={handleRefresh}
          disabled={refreshing}
          className="flex items-center gap-1 px-3 py-2 bg-dark-800 hover:bg-dark-700 disabled:opacity-50 rounded text-sm"
        >
          {refreshing ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <RefreshCw className="w-4 h-4" />
          )}
          Scrape library
        </button>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {collections.map((c) => (
          <CollectionCard
            key={c.id}
            collection={c}
            onPlay={(id) => navigate(`/player/${id}`)}
          />
        ))}
      </div>
    </div>
  );
}

function CollectionCard({
  collection,
  onPlay,
}: {
  collection: MovieCollection;
  onPlay: (id: string) => void;
}) {
  const backdrops = collection.movies
    .map((m) => m.scraped?.backdrop_path ?? m.scraped?.poster_path)
    .filter((p): p is string => !!p)
    .slice(0, 4);

  return (
    <div className="bg-dark-800 rounded-xl overflow-hidden group">
      <div className="relative aspect-video bg-dark-700">
        {backdrops.length > 0 ? (
          <div
            className="w-full h-full grid"
            style={{
              gridTemplateColumns:
                backdrops.length === 1
                  ? "1fr"
                  : backdrops.length === 2
                  ? "1fr 1fr"
                  : "2fr 1fr 1fr",
              gridTemplateRows: backdrops.length >= 3 ? "1fr 1fr" : "1fr",
            }}
          >
            {backdrops.map((p, i) => (
              <img
                key={i}
                src={tmdbImageUrl(p, "w780")!}
                alt=""
                className="w-full h-full object-cover"
                loading="lazy"
              />
            ))}
          </div>
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Layers className="w-12 h-12 text-dark-500" />
          </div>
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-dark-900 to-transparent" />
        <div className="absolute bottom-2 left-3 right-3">
          <div className="font-semibold text-white text-lg leading-tight drop-shadow">
            {collection.name}
          </div>
          <div className="text-xs text-dark-200 mt-0.5">
            {collection.movies.length} movie
            {collection.movies.length === 1 ? "" : "s"}
          </div>
        </div>
      </div>

      <div className="p-3 space-y-1.5">
        {collection.movies
          .slice()
          .sort((a, b) => (a.scraped?.year ?? 9999) - (b.scraped?.year ?? 9999))
          .map((m) => (
            <button
              key={m.id}
              onClick={() => onPlay(m.id)}
              className="w-full flex items-center gap-2 text-left px-2 py-1.5 rounded hover:bg-dark-700"
            >
              <div className="w-8 h-12 bg-dark-600 rounded flex-shrink-0 overflow-hidden">
                {m.scraped?.poster_path ? (
                  <img
                    src={tmdbImageUrl(m.scraped.poster_path, "w92")!}
                    alt=""
                    className="w-full h-full object-cover"
                    loading="lazy"
                  />
                ) : null}
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-sm text-white truncate">
                  {m.scraped?.title ?? m.name}
                </div>
                <div className="flex items-center gap-2 text-xs text-dark-400">
                  {m.scraped?.year && <span>{m.scraped.year}</span>}
                  {m.scraped?.rating && (
                    <span className="flex items-center gap-0.5 text-yellow-400">
                      <Star className="w-3 h-3 fill-current" />
                      {m.scraped.rating.toFixed(1)}
                    </span>
                  )}
                </div>
              </div>
            </button>
          ))}
      </div>
    </div>
  );
}
