import { pt } from "../content/pt";
import { Logo } from "./Logo";
import { installerUrl, REPO_URL, type ReleaseInfo } from "../lib/release";

export function Header({ release }: { release?: ReleaseInfo }) {
  const t = pt.nav;
  return (
    <header className="header">
      <div className="container header-row">
        <a className="brand" href="#top">
          <Logo /> Pacer
        </a>
        <nav className="nav" aria-label="Principal">
          <a href="#top" className="nav-active">{t.home}</a>
          <a href="#como">{t.features}</a>
          <a href="#privacidade">{t.about}</a>
          <a href={REPO_URL} rel="noopener">{t.github}</a>
        </nav>
        <a className="pill-btn" href={installerUrl(release)} rel="noopener">{t.download}</a>
      </div>
    </header>
  );
}
