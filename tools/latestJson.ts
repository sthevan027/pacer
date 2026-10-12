export interface LatestJsonInput {
  version: string;
  notes: string;
  pubDate: Date;
  /** Texto do arquivo .sig do instalador já assinado. */
  signature: string;
  /** "dono/repo" onde a release é publicada. */
  repo: string;
}

export interface LatestJson {
  version: string;
  notes: string;
  pub_date: string;
  platforms: { "windows-x86_64": { signature: string; url: string } };
}

/** Nome do instalador NSIS gerado pelo Tauri para uma versão (productName "Pacer"). */
export function installerName(version: string): string {
  return `Pacer_${version}_x64-setup.exe`;
}

/** Tag Git que a release precisa ter: a URL do instalador no latest.json depende dela. */
export function releaseTag(version: string): string {
  return `v${version.trim().replace(/^v/, "")}`;
}

const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

/** O manifesto que o tauri-plugin-updater lê em releases/latest/download/latest.json. */
export function buildLatestJson({ version, notes, pubDate, signature, repo }: LatestJsonInput): LatestJson {
  const clean = version.trim().replace(/^v/, "");
  if (!SEMVER.test(clean)) throw new Error(`versão "${version}" não é x.y.z`);
  const sig = signature.trim();
  if (!sig) throw new Error("assinatura vazia: assine o build com TAURI_SIGNING_PRIVATE_KEY antes");
  return {
    version: clean,
    notes,
    pub_date: pubDate.toISOString(),
    platforms: {
      "windows-x86_64": {
        signature: sig,
        url: `https://github.com/${repo}/releases/download/${releaseTag(clean)}/${installerName(clean)}`,
      },
    },
  };
}
