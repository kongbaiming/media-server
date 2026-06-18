import React from "react";
import ReactDOM from "react-dom/client";
// HashRouter (not BrowserRouter) so deep links survive reloads in the Tauri
// production build, where the React app is served from tauri:// or asset://
// and the server has no real router.
import { HashRouter } from "react-router-dom";
import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <App />
    </HashRouter>
  </React.StrictMode>,
);
