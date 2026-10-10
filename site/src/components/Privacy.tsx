import { pt } from "../content/pt";

export function Privacy() {
  return (
    <section id="privacidade" className="container section">
      <h2>{pt.privacy.title}</h2>
      <ul className="checks">
        {pt.privacy.items.map((t) => (
          <li key={t} className="card check"><span className="accent" aria-hidden="true">✓</span>{t}</li>
        ))}
      </ul>
    </section>
  );
}
