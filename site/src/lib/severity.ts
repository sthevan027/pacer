/** Mesma regra de src/lib/severity.ts do app: a cor da barra segue o uso, nunca a previsão. */
const ORANGE = [0xd2, 0x99, 0x22];
const RED = [0xf8, 0x51, 0x49];

const hex = (rgb: number[]) => `#${rgb.map((c) => c.toString(16).padStart(2, "0")).join("")}`;

/**
 * Cor de destaque (configurável) abaixo de 70%; de 70 a 90% um degradê laranja→vermelho que avança
 * com o uso; vermelho a partir de 90%. Aviso e crítico nunca mudam com a cor de destaque.
 */
export function usageColor(usedPct: number): string {
  if (usedPct < 70) return "var(--accent)";
  if (usedPct >= 90) return hex(RED);
  const t = (usedPct - 70) / 20;
  return hex(ORANGE.map((a, i) => Math.round(a + (RED[i] - a) * t)));
}
