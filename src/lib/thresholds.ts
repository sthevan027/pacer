/** Move o ponto `index` para `raw`, sem passar de 1–100 e sem encostar em outro ponto
 *  (pontos iguais seriam fundidos pelo backend e um limiar sumiria). */
export function moveThreshold(values: number[], index: number, raw: number): number[] {
  const cur = values[index];
  const others = values.filter((_, j) => j !== index);
  const below = others.filter((o) => o < cur);
  const above = others.filter((o) => o > cur);
  const min = below.length ? Math.max(...below) + 1 : 1;
  const max = above.length ? Math.min(...above) - 1 : 100;
  const v = Math.min(max, Math.max(min, Math.round(raw)));
  return values.map((x, j) => (j === index ? v : x));
}
