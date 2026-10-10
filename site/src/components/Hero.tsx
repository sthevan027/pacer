import { useState } from "react";
import { pt } from "../content/pt";
import { formatSize, LATEST_URL, REPO_URL, type ReleaseInfo } from "../lib/release";
import { isWindows } from "../lib/platform";
import { UsageWidget } from "./UsageWidget";

function DownloadButton({ release }: { release?: ReleaseInfo }) {
  const t = pt.hero;
  const [windows] = useState(() => typeof navigator === "undefined" || isWindows(navigator));
  if (!windows) {
    return (
      <div>
        <p className="windows-only">{t.ctaWindowsOnly}</p>
        <a className="cta cta-ghost" href={REPO_URL} rel="noopener">{t.ctaGithub}</a>
      </div>
    );
  }
  const installer = release?.installer;
  const meta = installer
    ? `${release!.version} · ${formatSize(installer.sizeBytes)} · ${t.fallbackMeta}`
    : release
      ? `${release.version} · ${t.fallbackMeta}`
      : t.fallbackMeta;
  return (
    <a className="cta" href={installer?.url ?? LATEST_URL} rel="noopener">
      <span aria-hidden="true">▦</span> {t.cta}
      <span className="cta-meta mono">{meta}</span>
    </a>
  );
}

export function Hero({ release }: { release?: ReleaseInfo }) {
  const t = pt.hero;
  return (
    <section className="hero" id="top">
      <div className="grid-bg" aria-hidden="true" />
      <div className="container hero-grid">
        <div>
          <span className="badge"><span className="accent" aria-hidden="true">▦</span> {t.badge}</span>
          <h1>{t.titleA}<span className="accent">{t.titleB}</span></h1>
          <p className="lead">{t.subtitle}</p>
          <DownloadButton release={release} />
          <p className="tiny muted hero-meta">{t.meta}</p>
        </div>
        <figure className="stage">
          <div className="stage-art">
            <div className="stage-widget"><UsageWidget /></div>
          </div>
          <figcaption className="stage-cap tiny muted">{pt.widget.caption}</figcaption>
        </figure>
      </div>
    </section>
  );
}
