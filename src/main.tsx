import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.scss";
import "ui-neumorphism/dist/index.css";
import { Store } from "./store.ts";
import { Store as TauriStore } from "@tauri-apps/plugin-store";

ReactDOM.createRoot(document.getElementById("app")!).render(
  <React.StrictMode>
    <Store.Provider value={new TauriStore("store.json")}>
      <App />
    </Store.Provider>
  </React.StrictMode>
);
