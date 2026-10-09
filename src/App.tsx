import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CSSProperties } from "react";
import { useEffect, useRef, useState } from "react";
import { MainView } from "./components/MainView";
import { SettingsView } from "./components/settings/SettingsView";
import { TrayHover } from "./components/TrayHover";
import { useAutoHeight, useNow, usePacer } from "./lib/hooks";

type View = "main" | "settings";

// A janela "hover" é uma janela nativa separada, pequena e fixa (sem auto-height nem o
// estado de abas) que só mostra as barras de Sessão e Semanal perto do ícone da bandeja.
const isHoverWindow = new URLSearchParams(window.location.search).get("view") === "hover";

export default function App() {
  return isHoverWindow ? <TrayHover /> : <MainApp />;
}

function MainApp() {
  const state = usePacer();
  const now = useNow();
  const [view, setView] = useState<View>("main");
  const ref = useRef<HTMLDivElement>(null);
  useAutoHeight(ref);

  useEffect(() => {
    const off = listen("open-settings", () => setView("settings"));
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setView((v) => {
        if (v === "settings") return "main";
        invoke("hide_window");
        return v;
      });
    };
    window.addEventListener("keydown", onKey);
    return () => {
      off.then((f) => f());
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  const style = state ? ({ "--accent": state.config.accentColor } as CSSProperties) : undefined;
  return (
    <div className="wg" ref={ref} style={style}>
      {!state ? (
        <p className="muted note">Carregando…</p>
      ) : view === "main" ? (
        <MainView state={state} now={now} onSettings={() => setView("settings")} />
      ) : (
        <SettingsView state={state} onBack={() => setView("main")} />
      )}
    </div>
  );
}
