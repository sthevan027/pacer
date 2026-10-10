import { pt } from "../content/pt";
import { REPO_URL } from "../lib/release";

export function Footer() {
  return (
    <footer className="footer">
      <div className="container footer-row tiny muted">
        <span>{pt.footer.made} · <a href={REPO_URL} rel="noopener">GitHub</a></span>
        <span>{pt.footer.disclaimer}</span>
      </div>
    </footer>
  );
}
