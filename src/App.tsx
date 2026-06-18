import { Routes, Route } from "react-router-dom";
import { useEffect } from "react";
import Layout from "./components/Layout/Layout";
import Library from "./components/Library/Library";
import Player from "./components/Player/Player";
import Search from "./components/Search/Search";
import Settings from "./components/Settings/Settings";
import DouyinInput from "./components/Douyin/DouyinInput";
import History from "./components/History/History";
import Online from "./components/Online/Online";
import Collections from "./components/Collections/Collections";
import Torrent from "./components/Torrent/Torrent";
import { waitForServer } from "@/services/api";
import { useMediaStore } from "./stores/mediaStore";

function App() {
  const { fetchLibrary, fetchConfig } = useMediaStore();

  useEffect(() => {
    const init = async () => {
      await waitForServer();
      fetchLibrary();
      fetchConfig();
    };
    init();
  }, [fetchLibrary, fetchConfig]);

  return (
    <div className="h-full flex flex-col bg-dark-950">
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Library />} />
          <Route path="search" element={<Search />} />
          <Route path="douyin" element={<DouyinInput />} />
          <Route path="online" element={<Online />} />
          <Route path="collections" element={<Collections />} />
          <Route path="torrent" element={<Torrent />} />
          <Route path="history" element={<History />} />
          <Route path="settings" element={<Settings />} />
        </Route>
        <Route path="/player/:id" element={<Player />} />
      </Routes>
    </div>
  );
}

export default App;

