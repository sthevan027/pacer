/**
 * Tier do modelo (opus/sonnet/haiku...) e a cor de cada um.
 *
 * Decisão do dono do projeto: o painel inteiro segue a cor de destaque escolhida, sem paleta
 * fixa por tier. Só que a pilha do gráfico precisa distinguir as famílias — então a variação é
 * na mistura com o branco, não na matiz: todos os tons continuam sendo a cor de destaque.
 */

export const FAMILY_ORDER = ["opus", "sonnet", "haiku", "fable", "mythos", "outro"] as const;

const LABELS: Record<string, string> = {
  opus: "Opus",
  sonnet: "Sonnet",
  haiku: "Haiku",
  fable: "Fable",
  mythos: "Mythos",
  outro: "Outros",
};

export function familyLabel(family: string): string {
  return LABELS[family] ?? family;
}

/** De 100% (o próprio accent) até 45%: abaixo disso o tom mais claro some no fundo escuro. */
export function rampPercent(index: number, count: number): number {
  if (count <= 1) return 100;
  const step = (100 - 45) / (count - 1);
  return Math.round(100 - index * step);
}

/** Cor CSS da família na posição `index` de `count`. Usável direto em `fill`/`stroke` do SVG. */
export function familyColor(index: number, count: number): string {
  return `color-mix(in srgb, var(--accent) ${rampPercent(index, count)}%, white)`;
}
