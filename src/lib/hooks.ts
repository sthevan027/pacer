import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type RefObject, useEffect, useState } from "react";
import type { AppState, Config, Snapshot } from "./types";

export function usePacer(): AppState | null {
  const [state, setState] = useState<AppState | null>(null);
  useEffect(() => {
    invoke<AppState>("get_state").then(setState);
    const offSnap = listen<Snapshot[]>("snapshot", (e) => setState((s) => (s ? { ...s, snapshots: e.payload } : s)));
    const offCfg = listen<Config>("config", (e) => setState((s) => (s ? { ...s, config: e.payload } : s)));
    return () => {
      offSnap.then((f) => f());
      offCfg.then((f) => f());
    };
  }, []);
  return state;
}

export function useNow(ms = 15_000): number {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(t);
  }, [ms]);
  return now;
}

/** Mantém a janela do tamanho do conteúdo. */
export function useAutoHeight(ref: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => {
      invoke("set_window_height", { height: Math.ceil(el.getBoundingClientRect().height) });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [ref]);
}
