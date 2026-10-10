import { familyLabel } from "../../lib/families";
import { formatBrl, formatTokens } from "../../lib/format";
import { cacheWrite, type UsageReport } from "../../lib/usage";
import { colorOf } from "./UsageChart";

export function TotalsTable({ report }: { report: UsageReport }) {
  const t = report.totals;
  return (
    <section className="pncard">
      <div className="pnch">
        <span>Por modelo</span>
        {/* Sem cache: é ele que inflava o número que o widget mostra. Aqui as partes aparecem
            separadas, como na referência do console. */}
        <span className="muted small">cache em coluna à parte</span>
      </div>
      <table className="pntab">
        <thead>
          <tr>
            <th>Modelo</th>
            <th>Tier</th>
            <th className="num">Entrada</th>
            <th className="num">Saída</th>
            <th className="num">Cache lido</th>
            <th className="num">Cache criado</th>
            <th className="num">Requisições</th>
            <th className="num">Valor</th>
          </tr>
        </thead>
        <tbody>
          {report.byModel.map((m) => (
            <tr key={m.model}>
              <td className="mono">{m.model}</td>
              <td>
                <i className="pndot" style={{ background: colorOf(report, m.family) }} />
                {familyLabel(m.family)}
              </td>
              <td className="num">{formatTokens(m.totals.input)}</td>
              <td className="num">{formatTokens(m.totals.output)}</td>
              <td className="num">{formatTokens(m.totals.cacheRead)}</td>
              <td className="num">{formatTokens(cacheWrite(m.totals))}</td>
              <td className="num">{m.totals.messages}</td>
              <td className="num">{m.costBrl === null ? "—" : formatBrl(m.costBrl)}</td>
            </tr>
          ))}
          <tr className="pnsum">
            <td colSpan={2}>Total</td>
            <td className="num">{formatTokens(t.input)}</td>
            <td className="num">{formatTokens(t.output)}</td>
            <td className="num">{formatTokens(t.cacheRead)}</td>
            <td className="num">{formatTokens(cacheWrite(t))}</td>
            <td className="num">{t.messages}</td>
            <td className="num">{formatBrl(report.totalCostBrl)}</td>
          </tr>
        </tbody>
      </table>

      {report.unpricedModels.length > 0 && (
        <p className="pnwarn">
          Sem preço na tabela: <b>{report.unpricedModels.join(", ")}</b>. O valor em R$ está subestimado — ele não
          foi chutado nem tratado como zero.
        </p>
      )}
    </section>
  );
}
