import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { familyColor, familyLabel } from "../../lib/families";
import { formatDay, formatTokens } from "../../lib/format";
import { metricLabel, metricValue, type Metric, type UsageReport } from "../../lib/usage";

interface Props {
  report: UsageReport;
  metric: Metric;
}

/** Cores por índice em `availableFamilies`, não na lista filtrada: assim desligar uma família
 *  não repinta as outras. */
export function colorOf(report: UsageReport, family: string): string {
  return familyColor(report.availableFamilies.indexOf(family), report.availableFamilies.length);
}

function Tip({ active, payload, label }: { active?: boolean; payload?: { name?: string; value?: number }[]; label?: string }) {
  if (!active || !payload?.length) return null;
  const linhas = payload.filter((p) => (p.value ?? 0) > 0);
  return (
    <div className="pntip">
      <b>{label}</b>
      {linhas.length === 0 ? (
        <span className="muted">sem consumo</span>
      ) : (
        linhas.map((p) => (
          <span key={p.name}>
            {familyLabel(String(p.name))} <em>{formatTokens(p.value ?? 0)}</em>
          </span>
        ))
      )}
    </div>
  );
}

export function UsageChart({ report, metric }: Props) {
  const data = report.days.map((d) => {
    const row: Record<string, string | number> = { label: formatDay(d.date) };
    for (const s of d.byFamily) row[s.family] = metricValue(s.totals, metric);
    return row;
  });
  const mostradas = report.filteredFamilies.length
    ? report.availableFamilies.filter((f) => report.filteredFamilies.includes(f))
    : report.availableFamilies;
  // 30 barras apertam os rótulos: mostra um a cada 4 dias (e um a cada 2 em 14).
  const intervalo = report.rangeDays >= 30 ? 4 : report.rangeDays >= 14 ? 1 : 0;

  return (
    <section className="pncard">
      <div className="pnch">
        <span>{metricLabel(metric)} por dia</span>
        <div className="pnlegend">
          {mostradas.map((f) => (
            <span key={f}>
              <i style={{ background: colorOf(report, f) }} />
              {familyLabel(f)}
            </span>
          ))}
        </div>
      </div>
      <div className="pnchart">
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
            <CartesianGrid vertical={false} stroke="rgba(255,255,255,0.06)" />
            <XAxis dataKey="label" interval={intervalo} tick={{ fontSize: 10, fill: "#8b949e" }} axisLine={false} tickLine={false} />
            <YAxis tickFormatter={formatTokens} tick={{ fontSize: 10, fill: "#8b949e" }} axisLine={false} tickLine={false} width={46} />
            <Tooltip content={<Tip />} cursor={{ fill: "rgba(255,255,255,0.04)" }} />
            {mostradas.map((f) => (
              <Bar key={f} dataKey={f} stackId="uso" fill={colorOf(report, f)} />
            ))}
          </BarChart>
        </ResponsiveContainer>
      </div>
    </section>
  );
}
