import { familyLabel } from "../../lib/families";
import { formatBrl, formatStamp, formatTokens } from "../../lib/format";
import { RECENT_LIMIT, type UsageReport } from "../../lib/usage";
import { colorOf } from "./UsageChart";

export function RequestsTable({ report }: { report: UsageReport }) {
  const cortadas = report.requestCount - report.recent.length;
  return (
    <section className="pncard">
      <div className="pnch">
        <span>Requisições recentes</span>
        <span className="muted small">
          {report.requestCount} no período
          {cortadas > 0 && ` · mostrando as ${RECENT_LIMIT} mais novas`}
        </span>
      </div>
      <table className="pntab">
        <thead>
          <tr>
            <th>Quando</th>
            <th>Tier</th>
            <th>Modelo</th>
            <th>Sessão</th>
            <th>Projeto</th>
            <th className="num">Entrada</th>
            <th className="num">Saída</th>
            <th className="num">Cache</th>
            <th className="num">Valor</th>
          </tr>
        </thead>
        <tbody>
          {report.recent.map((r, i) => (
            // o mesmo id de mensagem pode repetir entre sessões; o índice fecha a chave
            <tr key={`${r.ts}:${r.session}:${i}`}>
              <td className="mono">{formatStamp(r.ts)}</td>
              <td>
                <i className="pndot" style={{ background: colorOf(report, r.family) }} />
                {familyLabel(r.family)}
              </td>
              <td className="mono">{r.model}</td>
              <td className="mono">{r.session}</td>
              <td>{r.project}</td>
              <td className="num">{formatTokens(r.input)}</td>
              <td className="num">{formatTokens(r.output)}</td>
              <td className="num">{formatTokens(r.cacheRead + r.cacheWrite)}</td>
              <td className="num">{r.costBrl === null ? "—" : formatBrl(r.costBrl)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {report.recent.length === 0 && <p className="muted small">Nenhuma requisição no período.</p>}
    </section>
  );
}
