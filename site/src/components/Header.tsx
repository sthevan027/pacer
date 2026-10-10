import { pt } from "../content/pt";
import { LATEST_URL, REPO_URL } from "../lib/release";

export function Header() {
  const t = pt.nav;
  return (
    <header className="header">
      <div className="container header-row">
        <a className="brand" href="#top">
          <span className="brand-mark" aria-hidden="true">⚡</span> Pacer
        </a>
        <nav className="nav" aria-label="Principal">
          <a href="#top" className="nav-active">{t.home}</a>
          <a href="#como">{t.features}</a>
          <a href="#privacidade">{t.about}</a>
          <a href={REPO_URL} rel="noopener">{t.github}</a>
        </nav>
        <a className="pill-btn" href={LATEST_URL} rel="noopener">{t.download}</a>
      </div>
    </header>
  );
}
