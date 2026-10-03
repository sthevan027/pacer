import { useState } from "react";
import { formatDay, formatTokens } from "../lib/format";
import { buildGrid, type Cell } from "../lib/grid";
import type { DayActivity } from "../lib/types";

const DOW = ["seg", "", "qua", "", "sex", "", "dom"];

export function ActivityGrid({ days }: { days: DayActivity[] }) {
  const weeks = buildGrid(days);
  const [hover, setHover] = useState<Cell | null>(null);
  const total = days.reduce((sum, d) => sum + d.tokens, 0);
  const today = days[days.length - 1];

  return (
    <section className="sec">
      <div className="ctitle">
        <span>Atividade</span>
        <span className="muted light small">30 dias</span>
      </div>
      <div className="grid" style={{ gridTemplateColumns: `20px repeat(${weeks.length}, 1fr)` }}>
        {DOW.map((d, i) => (
          <b key={`dow-${i}`} style={{ gridColumn: 1, gridRow: i + 1 }}>
            {d}
          </b>
        ))}
        {weeks.flatMap((week, wi) =>
          week.map((c, di) => (
            <i
              key={`${wi}-${di}`}
              className={c ? `lv${c.level}` : "empty"}
              style={{ gridColumn: wi + 2, gridRow: di + 1, animationDelay: `${0.3 + wi * 0.06 + di * 0.02}s` }}
              onMouseEnter={() => c && setHover(c)}
              onMouseLeave={() => setHover(null)}
            />
          )),
        )}
      </div>
      <div className="hmeta">
        <span>
          {hover
            ? `${formatDay(hover.date)} · ${formatTokens(hover.tokens)} tokens · ${hover.messages} msgs`
            : `Hoje: ${formatTokens(today?.tokens ?? 0)} tokens`}
        </span>
        <span>30 dias: {formatTokens(total)}</span>
      </div>
    </section>
  );
}
