import { pt } from "../content/pt";

export function Features() {
  return (
    <section id="como" className="container section">
      <h2>{pt.how.title}</h2>
      <div className="cards-3">
        {pt.how.cards.map((c) => (
          <article key={c.title} className="card">
            <div className="shot"><img src={c.img} alt={c.alt} loading="lazy" /></div>
            <h3>{c.title}</h3>
            <p className="muted small">{c.text}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
