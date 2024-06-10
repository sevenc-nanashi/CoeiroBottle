import { Store as TauriStore } from "@tauri-apps/plugin-store";
import { createContext } from "react";

export const Store = createContext<TauriStore>(new TauriStore("store.json"));
