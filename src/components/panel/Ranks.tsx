import { formatTokens } from "../../lib/format";
import { billableTokens, type GroupRow, type UsageReport } from "../../lib/usage";
import { colorOf } from "./UsageChart";

/** Ranking não existe na referência do console — é extra nosso, pra ver quem gastou o quê. */
function Rank({
  title,
  hint,
  rows,
  color,
  mono,
}: {
  title: string;
  hint: string;
  rows: GroupRow[];
  color: string;
  mono?: boolean;
}) {
  const max = Math.max(1, ...rows.map((r) => billableTokens(r.totals)));
  return (
    <section className="pncard">
      <div className="pnch">
        <span>{title}</span>
        <span className="muted small">{hint}</span>
      </div>
      {rows.length === 0 ? (
        <p className="muted small">Nada no período.</p>
      ) : (
        <ol className="pnrank">
          {rows.map((r) => (
            <li key={`${r.name}:${r.sub}`}>
              <div className="pnrhead">
                <b className={mono ? "mono" : undefined}>{r.name}</b>
                {r.sub && <span className="muted small">{r.sub}</span>}
                <em>{formatTokens(billableTokens(r.totals))}</em>
              </div>
              <div className="pnrbar">
                <i style={{ width: `${(billableTokens(r.totals) / max) * 100}%`, background: color }} />
              </div>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}

export function Ranks({ report }: { report: UsageReport }) {
  // A primeira família visível dá a cor das barras, pra elas seguirem a cor de destaque.
  const familia = report.filteredFamilies[0] ?? report.availableFamilies[0] ?? "";
  const color = colorOf(report, familia);
  return (
    <div className="pngrid">
      <Rank title="Sessões" hint="entrada + saída" rows={report.topSessions} color={color} mono />
      <Rank title="Projetos" hint="entrada + saída" rows={report.topProjects} color={color} />
    </div>
  );
}
