import { describe, expect, it } from "vitest";
import { moveThreshold } from "./thresholds";

describe("moveThreshold", () => {
  it("move um ponto livremente dentro dos limites", () => {
    expect(moveThreshold([80, 95], 0, 60)).toEqual([60, 95]);
    expect(moveThreshold([80, 95], 1, 99)).toEqual([80, 99]);
  });

  it("não deixa um ponto encostar no outro (senão o dedup do backend apaga um)", () => {
    expect(moveThreshold([80, 95], 1, 80)).toEqual([80, 81]);
    expect(moveThreshold([80, 95], 1, 10)).toEqual([80, 81]);
    expect(moveThreshold([80, 95], 0, 99)).toEqual([94, 95]);
  });

  it("respeita 1% e 100% nas pontas", () => {
    expect(moveThreshold([80, 95], 0, 0)).toEqual([1, 95]);
    expect(moveThreshold([80, 95], 1, 200)).toEqual([80, 100]);
  });

  it("funciona com um único ponto", () => {
    expect(moveThreshold([80], 0, 120)).toEqual([100]);
  });
});
