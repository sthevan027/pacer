import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { Icon } from "../icons/icons";
import { pickActive } from "../lib/providers";
import type { AppState } from "../lib/types";
import { ActivityGrid } from "./ActivityGrid";
import { Footer } from "./Footer";
import { Header } from "./Header";
import { ProviderCard } from "./ProviderCard";
import { ProviderTabs } from "./ProviderTabs";

export function MainView({ state, now, onSettings }: { state: AppState; now: number; onSettings: () => void }) {
  const [chosen, setChosen] = useState<string | null>(null);
  const snap = pickActive(state.snapshots, chosen);
  return (
    <>
      <Header
        title="Pacer"
        sub="Uso dos planos de IA"
        actions={
          <>
            <button className="ib" title="Painel de uso" onClick={() => invoke("open_panel")}>
              <Icon name="chart" />
            </button>
            <button className="ib" title="Atualizar agora" onClick={() => invoke("refresh_now")}>
              <Icon name="refresh" />
            </button>
            <button className="ib" title="Configurações" onClick={onSettings}>
              <Icon name="gear" />
            </button>
          </>
        }
      />
      {snap ? (
        <>
          <ProviderTabs snapshots={state.snapshots} active={snap.provider} onSelect={setChosen} />
          <ProviderCard snap={snap} now={now} />
          {snap.activity.length > 0 && <ActivityGrid days={snap.activity} />}
        </>
      ) : (
        <p className="muted note">Nenhum provedor ativo. Ative um nas configurações.</p>
      )}
      <Footer snap={snap} now={now} />
    </>
  );
}
