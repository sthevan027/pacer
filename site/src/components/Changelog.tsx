import { useState, type ReactNode } from "react";
import { pt } from "../content/pt";
import { formatDate, RELEASES_URL, type ReleaseInfo } from "../lib/release";

/** `**destaque**` → <strong>. */
function inline(text: string): ReactNode[] {
  return text.split(/\*\*(.+?)\*\*/g).map((part, i) => (i % 2 ? <strong key={i}>{part}</strong> : part));
}

interface Row {
  version: string;
  title: string;
  date: string;
  url: string;
  highlights: string[];
}

export function Changelog({ releases }: { releases?: ReleaseInfo[] }) {
  const t = pt.news;
  const [open, setOpen] = useState<string | null>(null);
  const rows: Row[] = releases?.length
    ? releases.slice(0, 3)
    : t.fallback.map((r) => ({ ...r, url: `${RELEASES_URL}/tag/${r.version}` }));

  return (
    <section className="container section news">
      <h2 className="h-md">{t.title}</h2>
      <p className="muted small mt-sm">{t.subtitle}</p>
      <ul className="card list">
        {rows.map((r) => {
          const isOpen = open === r.version;
          const panelId = `notes-${r.version}`;
          return (
            <li key={r.version} className={`list-item${isOpen ? " is-open" : ""}`}>
              <button
                type="button"
                className="list-row"
                aria-expanded={isOpen}
                aria-controls={panelId}
                onClick={() => setOpen(isOpen ? null : r.version)}
              >
                <span className="tag mono">{r.version}</span>
                <span className="grow">{r.title}</span>
                <span className="row-hint tiny muted">{isOpen ? "" : t.expand}</span>
                <span className="mono tiny muted">{formatDate(r.date)}</span>
                <span className="chev" aria-hidden="true">▸</span>
              </button>
              <div id={panelId} className="notes" role="region" aria-label={`${t.title} ${r.version}`}>
                <div className="notes-inner">
                  {r.highlights.length ? (
                    <ul className="notes-list small">
                      {r.highlights.map((h, i) => (
                        <li key={i}>{inline(h)}</li>
                      ))}
                    </ul>
                  ) : (
                    <p className="muted small">{t.empty}</p>
                  )}
                  <a className="link tiny" href={r.url} rel="noopener">{t.fullNotes}</a>
                </div>
              </div>
            </li>
          );
        })}
      </ul>
      <a className="link small" href={RELEASES_URL} rel="noopener">{t.all}</a>
    </section>
  );
}
