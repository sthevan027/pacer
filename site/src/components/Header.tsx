import type { MouseEvent } from "react";
import { pt } from "../content/pt";
import { Logo } from "./Logo";
import { installerUrl, REPO_URL, type ReleaseInfo } from "../lib/release";

/** Rola até a seção sem mexer no endereço (nada de "#secao" na URL). O href continua para quem está sem JS. */
function goTo(id: string) {
  return (e: MouseEvent<HTMLAnchorElement>) => {
    const el = document.getElementById(id);
    if (!el) return;
    e.preventDefault();
    el.scrollIntoView();
  };
}

export function Header({ release }: { release?: ReleaseInfo }) {
  const t = pt.nav;
  return (
    <header className="header">
      <div className="container header-row">
        <a className="brand" href="#top" onClick={goTo("top")}>
          <Logo /> Pacer
        </a>
        <nav className="nav" aria-label="Principal">
          <a href="#top" className="nav-active" onClick={goTo("top")}>{t.home}</a>
          <a href="#como" onClick={goTo("como")}>{t.features}</a>
          <a href="#privacidade" onClick={goTo("privacidade")}>{t.about}</a>
          <a href={REPO_URL} rel="noopener">{t.github}</a>
        </nav>
        <a className="pill-btn" href={installerUrl(release)} rel="noopener">{t.download}</a>
      </div>
    </header>
  );
}
