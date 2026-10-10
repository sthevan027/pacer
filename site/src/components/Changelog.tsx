import { pt } from "../content/pt";
import { formatDate, RELEASES_URL, type ReleaseInfo } from "../lib/release";

export function Changelog({ releases }: { releases?: ReleaseInfo[] }) {
  const t = pt.news;
  const rows = releases?.length
    ? releases.slice(0, 3).map((r) => ({ version: r.version, title: r.title, date: r.date, url: r.url }))
    : t.fallback.map((r) => ({ ...r, url: RELEASES_URL }));
  return (
    <section className="container section news">
      <h2 className="h-md">{t.title}</h2>
      <p className="muted small mt-sm">{t.subtitle}</p>
      <ul className="card list">
        {rows.map((r) => (
          <li key={r.version} className="list-row">
            <a className="tag mono" href={r.url} rel="noopener">{r.version}</a>
            <span className="grow">{r.title}</span>
            <span className="mono tiny muted">{formatDate(r.date)}</span>
          </li>
        ))}
      </ul>
      <a className="link small" href={RELEASES_URL} rel="noopener">{t.all}</a>
    </section>
  );
}
