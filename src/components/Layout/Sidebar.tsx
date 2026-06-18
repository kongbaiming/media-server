import { NavLink } from "react-router-dom";
import {
  Library,
  Layers,
  Search,
  Settings,
  Film,
  Music,
  Heart,
  Clock,
  Video,
  Radio,
  Magnet,
} from "lucide-react";
import { useMediaStore } from "@/stores/mediaStore";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", icon: Library, label: "Library" },
  { to: "/search", icon: Search, label: "Search" },
  { to: "/douyin", icon: Video, label: "Douyin" },
  { to: "/online", icon: Radio, label: "Online" },
  { to: "/torrent", icon: Magnet, label: "Torrents" },
  { to: "/collections", icon: Layers, label: "Collections" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

const filterItems = [
  { type: "all" as const, icon: Library, label: "All Media" },
  { type: "video" as const, icon: Film, label: "Videos" },
  { type: "audio" as const, icon: Music, label: "Music" },
];

export default function Sidebar() {
  const { filterType, setFilterType, filterFavorite, setFilterFavorite } =
    useMediaStore();

  return (
    <aside className="w-64 bg-dark-900 border-r border-dark-800 flex flex-col">
      <div className="p-6 border-b border-dark-800">
        <h1 className="text-2xl font-bold text-primary-400">MediaVault</h1>
        <p className="text-sm text-dark-400 mt-1">Local Media Server</p>
      </div>

      <nav className="flex-1 p-4 space-y-1">
        <div className="mb-6">
          <h2 className="text-xs font-semibold text-dark-400 uppercase tracking-wider mb-3 px-3">
            Navigation
          </h2>
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 px-3 py-2 rounded-lg transition-colors",
                  isActive
                    ? "bg-primary-600/20 text-primary-400"
                    : "text-dark-300 hover:bg-dark-800 hover:text-white"
                )
              }
            >
              <item.icon className="w-5 h-5" />
              <span>{item.label}</span>
            </NavLink>
          ))}
        </div>

        <div className="mb-6">
          <h2 className="text-xs font-semibold text-dark-400 uppercase tracking-wider mb-3 px-3">
            Filters
          </h2>
          {filterItems.map((item) => (
            <button
              key={item.type}
              onClick={() => setFilterType(item.type)}
              className={cn(
                "flex items-center gap-3 px-3 py-2 rounded-lg transition-colors w-full text-left",
                filterType === item.type
                  ? "bg-primary-600/20 text-primary-400"
                  : "text-dark-300 hover:bg-dark-800 hover:text-white"
              )}
            >
              <item.icon className="w-5 h-5" />
              <span>{item.label}</span>
            </button>
          ))}

          <button
            onClick={() => setFilterFavorite(!filterFavorite)}
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-lg transition-colors w-full text-left",
              filterFavorite
                ? "bg-red-600/20 text-red-400"
                : "text-dark-300 hover:bg-dark-800 hover:text-white"
            )}
          >
            <Heart
              className={cn("w-5 h-5", filterFavorite && "fill-current")}
            />
            <span>Favorites</span>
          </button>
        </div>

        <div>
          <h2 className="text-xs font-semibold text-dark-400 uppercase tracking-wider mb-3 px-3">
            Quick Access
          </h2>
          <NavLink
            to="/history"
            className={({ isActive }) =>
              cn(
                "flex items-center gap-3 px-3 py-2 rounded-lg transition-colors",
                isActive
                  ? "bg-primary-600/20 text-primary-400"
                  : "text-dark-300 hover:bg-dark-800 hover:text-white"
              )
            }
          >
            <Clock className="w-5 h-5" />
            <span>Recent</span>
          </NavLink>
        </div>
      </nav>
    </aside>
  );
}






