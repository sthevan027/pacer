import { pt } from "../content/pt";

export function Install() {
  const t = pt.install;
  return (
    <section id="instalacao" className="container section">
      <h2>{t.title}</h2>
      <p className="muted mt-sm">{t.subtitle}</p>
      <div className="install-grid">
        <ol className="steps">
          {t.steps.map((s, i) => (
            <li key={s.title} className="card step">
              <span className="step-n mono">{i + 1}</span>
              <div>
                <h3>{s.title}</h3>
                <p className="muted small">{s.text}</p>
              </div>
            </li>
          ))}
        </ol>
        <div className="card pad-lg">
          <h3>{t.warnTitle}</h3>
          <p className="muted small mt-sm">{t.warnText}</p>
          <div className="smartscreen" aria-hidden="true">
            <p className="smart-title">{t.smartTitle}</p>
            <p className="small">{t.smartText}</p>
            <p className="small underline">{t.smartMore}</p>
            <div className="smart-actions small">
              <span className="smart-btn smart-btn-on">{t.smartRun}</span>
              <span className="smart-btn">{t.smartCancel}</span>
            </div>
          </div>
        </div>
      </div>
      <div className="reqs">
        <strong className="small">{t.requirements}</strong>
        {t.reqs.map((r) => (
          <span key={r} className="chip">{r}</span>
        ))}
      </div>
    </section>
  );
}
