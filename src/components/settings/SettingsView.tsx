import { invoke } from "@tauri-apps/api/core";
import { Fragment } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Icon } from "../../icons/icons";
import { PROVIDERS, providerStatus, withProviderEnabled } from "../../lib/providers";
import type { AppState, Config } from "../../lib/types";
import { Header } from "../Header";
import { KeyField } from "./KeyField";
import { ProviderMark } from "../ProviderMark";
import { AccentPicker } from "./AccentPicker";
import { RateInput } from "./RateInput";
import { Row } from "./Row";
import { Segmented } from "./Segmented";
import { ThresholdBar } from "./ThresholdBar";
import { Toggle } from "./Toggle";

const REPO_URL = "https://github.com/sthevan027/pacer";
const REFRESH_OPTIONS = [1, 5, 10] as const;

export function SettingsView({ state, onBack }: { state: AppState; onBack: () => void }) {
  const cfg = state.config;
  const save = (patch: Partial<Config>) => invoke<Config>("save_config", { config: { ...cfg, ...patch } });

  return (
    <>
      <Header
        title="Configurações"
        leading={
          <button className="ib" title="Voltar" onClick={onBack}>
            <Icon name="back" />
          </button>
        }
        actions={<span className="ver">v{state.version}</span>}
      />

      <section className="sec">
        <div className="sh">
          <Icon name="sliders" />
          Geral
        </div>
        <Row label="Iniciar com o Windows" hint="Abre sozinho ao ligar o PC">
          <Toggle label="Iniciar com o Windows" on={cfg.startWithWindows} onChange={(v) => save({ startWithWindows: v })} />
        </Row>
        <Row label="Atualizar a cada">
          <Segmented value={cfg.refreshMinutes} options={REFRESH_OPTIONS} format={(v) => `${v}m`} onChange={(v) => save({ refreshMinutes: v })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="palette" />
          Aparência
        </div>
        <Row label="Cor de destaque" hint="Vale pro painel, pro hover da bandeja e pro ícone">
          <AccentPicker value={cfg.accentColor} onChange={(accentColor) => save({ accentColor })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="trend" />
          Valor estimado
        </div>
        <Row label="Cotação do dólar" hint="Só para estimar o consumo a preço de API no painel de uso">
          <RateInput value={cfg.usdBrl} onCommit={(usdBrl) => save({ usdBrl })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="bell" />
          Alertas
        </div>
        <ThresholdBar values={cfg.alerts.thresholds} onChange={(t) => save({ alerts: { ...cfg.alerts, thresholds: t } })} />
        <Row label="Notificações" hint="Avisos do Windows ao passar dos limites">
          <Toggle label="Notificações" on={cfg.alerts.enabled} onChange={(v) => save({ alerts: { ...cfg.alerts, enabled: v } })} />
        </Row>
        <Row label="Avisar se a previsão estourar" hint="Antes de chegar no limite">
          <Toggle label="Avisar se a previsão estourar" on={cfg.alerts.pace} onChange={(v) => save({ alerts: { ...cfg.alerts, pace: v } })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="plug" />
          Provedores
        </div>
        {PROVIDERS.map((p) => {
          const snap = state.snapshots.find((s) => s.provider === p.id);
          const enabled = cfg.providers[p.id].enabled;
          const status = providerStatus(p, snap, enabled);
          return (
            <Fragment key={p.id}>
            <div className="prov">
              <div className={`plogo${p.id === "claude" ? "" : " neutral"}`}>
                <ProviderMark id={p.id} />
              </div>
              <div className="lbl grow">
                <b>
                  {p.name} {snap?.plan && <span className="muted">· {snap.plan}</span>}
                </b>
                <span>
                  {status.connected && <i className="pulse" />}
                  {status.text}
                </span>
              </div>
              <Toggle label={p.name} on={enabled} onChange={(v) => save({ providers: withProviderEnabled(cfg.providers, p.id, v) })} />
            </div>
            {p.apiKey && (
              <KeyField
                provider={p}
                stored={state.storedKeys.includes(p.id)}
                onSaved={() => !enabled && save({ providers: withProviderEnabled(cfg.providers, p.id, true) })}
              />
            )}
            </Fragment>
          );
        })}
        <div className="ghost">
          <Icon name="plus" size={13} />
          Adicionar provedor <span className="dim">(em breve)</span>
        </div>
      </section>

      <div className="foot">
        <span>Pacer v{state.version}</span>
        <a
          href={REPO_URL}
          onClick={(e) => {
            e.preventDefault();
            openUrl(REPO_URL);
          }}
        >
          GitHub
        </a>
      </div>
    </>
  );
}
