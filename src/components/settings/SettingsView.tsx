import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ClaudeLogo, Icon } from "../../icons/icons";
import type { AppState, Config } from "../../lib/types";
import { Header } from "../Header";
import { Row } from "./Row";
import { Segmented } from "./Segmented";
import { ThresholdBar } from "./ThresholdBar";
import { Toggle } from "./Toggle";

const REPO_URL = "https://github.com/sthevan027/Claude-Glass";
const REFRESH_OPTIONS = [1, 5, 10] as const;

export function SettingsView({ state, onBack }: { state: AppState; onBack: () => void }) {
  const cfg = state.config;
  const save = (patch: Partial<Config>) => invoke<Config>("save_config", { config: { ...cfg, ...patch } });
  const claude = state.snapshots.find((s) => s.provider === "claude");
  const connected = !!claude && claude.notice !== "noCredentials" && claude.notice !== "tokenExpired";

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
        <div className="prov">
          <div className="plogo">
            <ClaudeLogo />
          </div>
          <div className="lbl grow">
            <b>
              Claude {claude?.plan && <span className="muted">· {claude.plan}</span>}
            </b>
            <span>
              {connected ? (
                <>
                  <i className="pulse" />
                  Conectado via Claude Code
                </>
              ) : (
                "Sem login do Claude Code"
              )}
            </span>
          </div>
          <Toggle label="Claude" on={cfg.providers.claude.enabled} onChange={(v) => save({ providers: { claude: { enabled: v } } })} />
        </div>
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
