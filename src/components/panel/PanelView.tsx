import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import type { CSSProperties } from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { Icon } from "../../icons/icons";
import { formatAgo } from "../../lib/format";
import { useNow, usePacer } from "../../lib/hooks";
import type { Range, UsageReport } from "../../lib/usage";
import { Filters } from "./Filters";
import { Ranks } from "./Ranks";
import { RequestsTable } from "./RequestsTable";
import { TotalsTable } from "./TotalsTable";
import { colorOf, UsageChart } from "./UsageChart";

type Tab = "overview" | "requests";

/**
 * Painel de uso: janela própria (`index.html?view=panel`), aberta por clique e destruída ao
 * fechar. Carregado por `import()` dinâmico no App.tsx — o gráfico traz uma biblioteca, e ela
 * não pode entrar no chunk que o widget carrega.
 *
 * Ao contrário do widget, aqui NÃO se usa `useAutoHeight`: a janela é redimensionável de verdade.
 */
export default function PanelView() {
  const state = usePacer();
  const now = useNow();
  const [report, setReport] = useState<UsageReport | null>(null);
  const [range, setRange] = useState<Range>(30);
  const [selected, setSelected] = useState<string[]>([]);
  const [metric, setMetric] = useState<"tokens" | "cacheRead" | "cacheWrite" | "total">("tokens");
  const [tab, setTab] = useState<Tab>("overview");
  const [busy, setBusy] = useState(true);
  const [erro, setErro] = useState<string | null>(null);
  const [aviso, setAviso] = useState<string | null>(null);
  // evita que uma resposta antiga (filtro anterior) sobrescreva a atual
  const geracao = useRef(0);
  const caixa = useRef<HTMLDivElement>(null);
  const jaAbriu = useRef(false);

  const chave = selected.join(",");
  const load = useCallback(async () => {
    const minha = ++geracao.current;
    setBusy(true);
    try {
      const r = await invoke<UsageReport>("get_usage_report", {
        rangeDays: range,
        families: chave ? chave.split(",") : [],
      });
      if (minha !== geracao.current) return;
      setReport(r);
      setErro(null);
    } catch (e) {
      if (minha === geracao.current) setErro(String(e));
    } finally {
      if (minha === geracao.current) setBusy(false);
    }
  }, [range, chave]);

  useEffect(() => {
    void load();
  }, [load]);

  // O painel abre no topo. Sem isto ele aparecia rolado no fim: enquanto o relatório não chega o
  // conteúdo é curto, e o crescimento do gráfico depois disso deixa o contêiner rolado lá embaixo.
  // Só na primeira carga — o painel se atualiza sozinho a cada 10 s e não pode roubar o scroll.
  useEffect(() => {
    if (!report || jaAbriu.current) return;
    jaAbriu.current = true;
    caixa.current?.scrollTo({ top: 0 });
  }, [report]);

  // O scheduler roda a cada 10 s e emite `snapshot`; o log é relido antes disso, então o painel
  // acompanha sozinho em vez de exigir clicar em atualizar.
  useEffect(() => {
    const off = listen("snapshot", () => void load());
    return () => {
      off.then((f) => f());
    };
  }, [load]);

  const snap = state?.snapshots[0];
  const corDe = useCallback(
    (f: string) => (report ? colorOf(report, f) : "var(--accent)"),
    [report],
  );

  const onToggleFamily = (f: string) => {
    setSelected((s) => (s.includes(f) ? s.filter((x) => x !== f) : [...s, f]));
  };

  const exportar = async () => {
    setAviso(null);
    try {
      const path = await invoke<string>("export_usage_csv", {
        rangeDays: range,
        families: chave ? chave.split(",") : [],
      });
      setAviso(path);
      await revealItemInDir(path);
    } catch (e) {
      setErro(String(e));
    }
  };

  const style = state ? ({ "--accent": state.config.accentColor } as CSSProperties) : undefined;

  return (
    <div className="pn" ref={caixa} style={style}>
      <header className="pntop">
        <h1>Uso</h1>
        <span className="pnlive">
          {snap?.fetchedAt ? (
            <>
              <i className="pulse" />
              atualizado {formatAgo(snap.fetchedAt, now)}
            </>
          ) : (
            "só logs locais"
          )}
        </span>
        <div className="grow" />
        <button className="pnbtn" onClick={() => invoke("refresh_now")} title="Buscar uso agora">
          <Icon name="refresh" />
          Atualizar
        </button>
        <button className="pnbtn" onClick={exportar} title="Grava um CSV em Downloads com os nomes de projeto e sessão">
          <Icon name="download" />
          Exportar CSV
        </button>
      </header>

      <Filters
        range={range}
        onRange={setRange}
        metric={metric}
        onMetric={setMetric}
        available={report?.availableFamilies ?? []}
        selected={selected}
        onToggleFamily={onToggleFamily}
        colorOfFamily={corDe}
        busy={busy}
      />

      {erro && <p className="pnwarn">{erro}</p>}
      {aviso && <p className="pnnfo">CSV gravado em {aviso}</p>}

      {!report ? (
        <p className="muted">Carregando…</p>
      ) : (
        <>
          <nav className="pntabs">
            <button className={tab === "overview" ? "on" : undefined} onClick={() => setTab("overview")}>
              Visão geral
            </button>
            <button className={tab === "requests" ? "on" : undefined} onClick={() => setTab("requests")}>
              Registros
            </button>
          </nav>

          {tab === "overview" ? (
            <>
              <UsageChart report={report} metric={metric} />
              <TotalsTable report={report} />
              <Ranks report={report} />
            </>
          ) : (
            <RequestsTable report={report} />
          )}
        </>
      )}

      <footer className="pnfoot">
        <span>
          Valor estimado a preço de API (US$ {report?.usdBrl ?? "—"}), não é o que você paga na assinatura. Ajuste a
          cotação nas Configurações.
        </span>
        <span className="dim">Tudo lido do seu disco, nada sai da máquina.</span>
      </footer>
    </div>
  );
}
