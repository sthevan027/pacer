import { invoke } from "@tauri-apps/api/core";
import { Icon } from "../icons/icons";
import type { AppState } from "../lib/types";
import { ActivityGrid } from "./ActivityGrid";
import { Footer } from "./Footer";
import { Header } from "./Header";
import { ProviderCard } from "./ProviderCard";
import { ProviderTabs } from "./ProviderTabs";

export function MainView({ state, now, onSettings }: { state: AppState; now: number; onSettings: () => void }) {
  const snap = state.snapshots[0];
  return (
    <>
      <Header
        title="Pacer"
        sub="Uso dos planos de IA"
        draggable={!state.config.lockPosition}
        actions={
          <>
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
          <ProviderTabs snapshots={state.snapshots} active={snap.provider} />
          <ProviderCard snap={snap} now={now} />
          <ActivityGrid days={snap.activity} />
        </>
      ) : (
        <p className="muted note">Nenhum provedor ativo. Ative um nas configurações.</p>
      )}
      <Footer snap={snap} now={now} />
    </>
  );
}
