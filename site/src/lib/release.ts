export const REPO = "sthevan027/pacer";
export const REPO_URL = `https://github.com/${REPO}`;
export const RELEASES_URL = `${REPO_URL}/releases`;
/**
 * Instalador usado só enquanto a API de releases não respondeu (ou falhou): o clique já baixa um
 * instalador funcional em vez de abrir a página da release. Com a API no ar, vale sempre o mais recente.
 * Atualize junto com uma release, se quiser que o fallback acompanhe.
 */
export const FALLBACK_INSTALLER_URL = `${RELEASES_URL}/download/v2.1.0/Pacer_2.1.0_x64-setup.exe`;

/** Link de download direto: o instalador da última release, ou o fallback. */
export function installerUrl(release?: ReleaseInfo): string {
  return release?.installer?.url ?? FALLBACK_INSTALLER_URL;
}

const API_URL = `https://api.github.com/repos/${REPO}/releases?per_page=4`;
const CACHE_KEY = "pacer:releases:v1";

export interface ReleaseInfo {
  version: string;
  title: string;
  date: string;
  url: string;
  /** Resumo do que mudou, extraído das notas da release (até 5 itens; `**negrito**` marca o destaque). */
  highlights: string[];
  installer?: { url: string; sizeBytes: number };
}

interface RawAsset {
  name: string;
  size: number;
  browser_download_url: string;
}
interface RawRelease {
  tag_name: string;
  name: string | null;
  html_url: string;
  published_at: string | null;
  body?: string | null;
  draft: boolean;
  prerelease: boolean;
  assets: RawAsset[];
}

/** O instalador chama `Pacer_<versão>_x64-setup.exe` — a versão faz parte do nome. */
export function pickInstaller(assets: RawAsset[]): RawAsset | undefined {
  return assets.find((a) => /_x64-setup\.exe$/i.test(a.name));
}

/** Seções das notas que não são "o que mudou": instalação, requisitos, assinatura/hash. */
const SKIP_SECTION = /instala|requisit|assinad|sha-?256|⚠/i;

/** Lê os itens de lista das notas, parando na primeira seção que não é de mudanças. */
export function parseHighlights(body: string | null | undefined, max = 5): string[] {
  if (!body) return [];
  const items: string[] = [];
  let current: string | null = null;
  const flush = () => {
    if (current) items.push(clean(current));
    current = null;
  };
  for (const line of body.split(/\r?\n/)) {
    if (/^#{1,6}\s/.test(line)) {
      flush();
      if (SKIP_SECTION.test(line)) break;
      continue;
    }
    const bullet = line.match(/^\s*[-*]\s+(.*)$/);
    if (bullet) {
      flush();
      current = bullet[1];
    } else if (current && /^\s+\S/.test(line)) {
      current += " " + line.trim(); // continuação de um item
    } else {
      flush();
    }
  }
  flush();
  return items.filter(Boolean).slice(0, max);
}

function clean(text: string): string {
  return text
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "") // imagens
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1") // links → texto
    .replace(/\s*\(#\d+\)/g, "") // referências de PR
    .replace(/`([^`]+)`/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
}

export function toReleaseInfo(raw: RawRelease): ReleaseInfo {
  const asset = pickInstaller(raw.assets ?? []);
  return {
    version: raw.tag_name,
    title: raw.name && raw.name !== raw.tag_name ? raw.name : `Pacer ${raw.tag_name.replace(/^v/, "")}`,
    date: raw.published_at ?? "",
    url: raw.html_url,
    highlights: parseHighlights(raw.body),
    installer: asset ? { url: asset.browser_download_url, sizeBytes: asset.size } : undefined,
  };
}

export function parseReleases(raw: unknown): ReleaseInfo[] {
  if (!Array.isArray(raw)) return [];
  return (raw as RawRelease[])
    .filter((r) => r && !r.draft && !r.prerelease && typeof r.tag_name === "string")
    .map(toReleaseInfo);
}

type StorageLike = Pick<Storage, "getItem" | "setItem">;

function readCache(storage?: StorageLike): ReleaseInfo[] | null {
  try {
    const text = storage?.getItem(CACHE_KEY);
    const list = text ? parseReleases(JSON.parse(text)) : [];
    return list.length ? list : null;
  } catch {
    return null;
  }
}

/** Busca as últimas releases (uma chamada serve botão e novidades), com cache de sessão. */
export async function fetchReleases(
  fetchFn: typeof fetch = fetch,
  storage?: StorageLike,
): Promise<ReleaseInfo[]> {
  const cached = readCache(storage);
  if (cached) return cached;
  const res = await fetchFn(API_URL, { headers: { Accept: "application/vnd.github+json" } });
  if (!res.ok) throw new Error(`GitHub API ${res.status}`);
  const raw = await res.json();
  const list = parseReleases(raw);
  try {
    storage?.setItem(CACHE_KEY, JSON.stringify(raw));
  } catch {
    /* sem cache, tudo bem */
  }
  return list;
}

export function formatSize(bytes: number): string {
  return `${(bytes / 1024 / 1024).toLocaleString("pt-BR", { maximumFractionDigits: 1, minimumFractionDigits: 1 })} MB`;
}

export function formatDate(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString("pt-BR", { timeZone: "UTC" });
}
