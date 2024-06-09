import React, { useState } from "react";
import { version } from "../package.json";
import CoeiroinkManager from "./views/CoeiroinkManager";
import clsx from "clsx";

const App: React.FC = () => {
  const [view, setView] = useState("coeiroinkManager");

  return (
    <div className="overflow-hidden w-screen h-screen grid grid-rows-[1fr_auto]">
      <main className="p-2 flex flex-col">
        <header className="grid grid-cols-2 gap-2">
          <button
            className={clsx("button", view === "coeiroinkManager" && "active")}
            onClick={() => setView("coeiroinkManager")}
          >
            Coeiroinkを管理する
          </button>

          <button
            className={clsx("button", view === "myCoe" && "active")}
            onClick={() => setView("myCoe")}
          >
            MyCoeを管理する
          </button>
        </header>
        <div className="h-2" />
        {view === "coeiroinkManager" ? (
          <CoeiroinkManager />
        ) : (
          <div>MyCoe管理</div>
        )}
      </main>
      <footer className="grid place-items-center opacity-50 py-2 text-xs">
        CoeiroBottle - Coeiroink Helper v{version}
      </footer>
    </div>
  );
};

export default App;
