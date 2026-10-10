import { AbsoluteFill, Audio, Easing, Sequence, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import type { CSSProperties, ReactNode } from "react";

export const FPS = 30;
export const DURATION = 465; // 15,5 s
const s = (sec: number) => Math.round(sec * FPS);

// Tokens do app/landing.
const C = { bg: "#0d1117", card: "#151b23", surface: "#1c232d", border: "#272f3a", text: "#e6edf3", muted: "#8b949e", warn: "#d29922", crit: "#f85149" };
const ACCENTS = ["#1f6feb", "#8957e5", "#2ea043", "#db61a2"]; // Azul, Roxo, Verde, Rosa (presets do app)
const FONT = `Inter, "Segoe UI", system-ui, sans-serif`;
const MONO = `"DejaVu Sans Mono", ui-monospace, monospace`;

const ease = Easing.bezier(0.16, 1, 0.3, 1);
const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
const prog = (f: number, a: number, b: number) => interpolate(f, [a, b], [0, 1], { ...clamp, easing: ease });

/** Cor de destaque ao longo do vídeo: azul até "combina com a sua cor" (10,2 s), troca a cada ~0,3 s, volta ao azul. */
function useAccent(frame: number) {
  const t = frame / FPS;
  if (t < 10.4) return ACCENTS[0];
  if (t < 10.75) return ACCENTS[1];
  if (t < 11.1) return ACCENTS[2];
  if (t < 11.45) return ACCENTS[3];
  return ACCENTS[0];
}

const Bar = ({ label, pct, forecast, reset, tone, accent, fill }: { label: string; pct: number; forecast: number; reset: string; tone?: "warn"; accent: string; fill: number }) => (
  <div>
    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 22, marginBottom: 12 }}>
      <span style={{ color: C.muted }}>{label}</span>
      <span style={{ fontFamily: MONO, fontSize: 20 }}>{Math.round(pct * fill)}% <span style={{ color: C.muted }}>· reinicia {reset}</span></span>
    </div>
    <div style={{ position: "relative", height: 14, borderRadius: 99, background: C.surface }}>
      <div style={{ height: "100%", width: `${pct * fill}%`, borderRadius: 99, background: tone === "warn" ? C.warn : accent }} />
      <div style={{ position: "absolute", top: -6, left: `${Math.min(forecast, 99) * fill}%`, width: 3, height: 26, borderRadius: 2, background: "rgba(230,237,243,.8)", opacity: fill }} />
    </div>
  </div>
);

const Widget = ({ frame, accent, calloutAt }: { frame: number; accent: string; calloutAt: number }) => {
  const fill = prog(frame, 20, 70);
  const callout = prog(frame, calloutAt, calloutAt + 14);
  return (
    <div style={{ width: 620, background: C.card, border: `1px solid ${C.border}`, borderRadius: 28, padding: 34, boxShadow: `0 0 120px -20px ${accent}99`, fontFamily: FONT, color: C.text }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 28 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12, fontSize: 26, fontWeight: 600 }}>
          <span style={{ width: 16, height: 16, borderRadius: 99, background: accent }} /> Pacer
        </div>
        <span style={{ fontFamily: MONO, fontSize: 18, color: C.muted }}>v2.1.0</span>
      </div>
      <div style={{ display: "grid", gap: 30 }}>
        <Bar label="Sessão (5h)" pct={42} forecast={71} reset="em 2h 14m" accent={accent} fill={fill} />
        <Bar label="Semanal" pct={81} forecast={100} reset="em 2d 9h" tone="warn" accent={accent} fill={fill} />
      </div>
      <div style={{ marginTop: 28, padding: "14px 18px", fontSize: 22, color: C.warn, border: `1px solid ${C.warn}66`, background: `${C.warn}1a`, borderRadius: 14, opacity: callout, transform: `translateY(${(1 - callout) * 14}px)` }}>
        Nesse ritmo, acaba em 1d 7h.
      </div>
    </div>
  );
};

const Caption = ({ children, from, to, accentWord }: { children: ReactNode; from: number; to: number; accentWord?: ReactNode }) => {
  const f = useCurrentFrame();
  const inP = prog(f, from, from + 14);
  const outP = prog(f, to - 8, to);
  const o = inP * (1 - outP);
  return (
    <div style={{ opacity: o, transform: `translateY(${(1 - inP) * 24 - outP * 12}px)`, fontFamily: FONT, fontWeight: 600, letterSpacing: -2, lineHeight: 1.05 }}>
      {children}
      {accentWord}
    </div>
  );
};

const Dock = ({ children, style }: { children: ReactNode; style?: CSSProperties }) => <div style={{ position: "absolute", ...style }}>{children}</div>;

const Tray = ({ frame, accent }: { frame: number; accent: string }) => {
  const hover = prog(frame, 10, 28);
  return (
    <div style={{ fontFamily: FONT, color: C.text }}>
      <div style={{ width: 560, background: C.card, border: `1px solid ${C.border}`, borderRadius: 22, padding: 28, opacity: hover, transform: `translateY(${(1 - hover) * 20}px)`, boxShadow: `0 0 90px -20px ${accent}88`, marginBottom: 18 }}>
        <div style={{ display: "grid", gap: 24 }}>
          <Bar label="Sessão (5h)" pct={42} forecast={71} reset="em 2h 14m" accent={accent} fill={1} />
          <Bar label="Semanal" pct={81} forecast={100} reset="em 2d 9h" tone="warn" accent={accent} fill={1} />
        </div>
      </div>
      <div style={{ display: "flex", justifyContent: "flex-end", alignItems: "center", gap: 18, height: 64, padding: "0 22px", background: "#10151c", border: `1px solid ${C.border}`, borderRadius: 14 }}>
        <span style={{ color: C.muted, fontSize: 20, fontFamily: MONO }}>POR</span>
        <span style={{ color: C.muted, fontSize: 20, fontFamily: MONO }}>14:32</span>
        <svg width="34" height="34" viewBox="0 0 1024 1024" style={{ outline: `2px solid ${accent}`, outlineOffset: 6, borderRadius: 8 }}>
          <circle cx="512" cy="512" r="300" fill="none" stroke="#2d333b" strokeWidth="88" />
          <path d="M512 212 A300 300 0 1 1 252 662" fill="none" stroke={accent} strokeWidth="88" strokeLinecap="round" />
          <rect x="496" y="150" width="32" height="150" rx="16" fill="#e6edf3" />
        </svg>
      </div>
    </div>
  );
};

const Swatches = ({ frame, accent }: { frame: number; accent: string }) => {
  const o = prog(frame, 0, 12);
  return (
    <div style={{ display: "flex", gap: 22, opacity: o }}>
      {ACCENTS.map((c) => (
        <div key={c} style={{ width: 56, height: 56, borderRadius: 99, background: c, boxShadow: accent === c ? `0 0 0 5px ${C.bg}, 0 0 0 8px ${C.text}` : "none", transform: `scale(${accent === c ? 1.12 : 1})` }} />
      ))}
    </div>
  );
};

export const Hero = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const accent = useAccent(frame);

  // Fundo: grade + brilho que acompanha a cor.
  const drift = (frame / DURATION) * 48;
  const intro = prog(frame, 0, 40);

  // Anel de abertura (gauge) enche e some quando o widget entra.
  const ring = prog(frame, 0, s(2.0));
  const ringOut = 1 - prog(frame, s(2.1), s(2.9));
  const R = 330, circ = 2 * Math.PI * R;

  const widgetIn = spring({ frame: frame - s(2.4), fps, config: { damping: 18, stiffness: 90 } });
  const widgetOut = prog(frame, s(7.0), s(7.5));
  const trayIn = prog(frame, s(7.3), s(7.8));
  const trayOut = prog(frame, s(11.4), s(11.9));
  const endIn = prog(frame, s(11.9), s(12.6));

  // Voz em volume cheio; trilha baixa e mais alta no início/fim.
  const musicVol = (f: number) => interpolate(f, [0, 20, s(1), s(13.6), s(15.3)], [0, 0.22, 0.14, 0.14, 0], clamp);

  return (
    <AbsoluteFill style={{ background: C.bg, fontFamily: FONT, color: C.text, overflow: "hidden" }}>
      <AbsoluteFill style={{ opacity: intro, backgroundImage: "linear-gradient(to right, rgba(230,237,243,.05) 1px, transparent 1px), linear-gradient(to bottom, rgba(230,237,243,.05) 1px, transparent 1px)", backgroundSize: "64px 64px", backgroundPosition: `${drift}px ${drift}px`, WebkitMaskImage: "radial-gradient(ellipse at 50% 30%, #000 25%, transparent 75%)" }} />
      <AbsoluteFill style={{ background: `radial-gradient(60% 55% at ${30 + Math.sin(frame / 90) * 6}% 85%, ${accent}66, transparent 70%)` }} />

      {/* Cena A — gancho */}
      <Sequence from={0} durationInFrames={s(3.0)}>
        <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: ringOut }}>
          <svg width="760" height="760" viewBox="0 0 760 760" style={{ position: "absolute", transform: `scale(${0.92 + ring * 0.08})` }}>
            <circle cx="380" cy="380" r={R} fill="none" stroke="#2d333b" strokeWidth="26" />
            <circle cx="380" cy="380" r={R} fill="none" stroke={accent} strokeWidth="26" strokeLinecap="round" strokeDasharray={circ} strokeDashoffset={circ * (1 - ring * 0.82)} transform="rotate(-90 380 380)" />
            <rect x="373" y="26" width="14" height="62" rx="7" fill={C.text} transform={`rotate(${ring * 295} 380 380)`} />
          </svg>
          <div style={{ position: "absolute", fontSize: 96, textAlign: "center" }}>
            <Caption from={s(0.15)} to={s(2.6)}>
              Seu limite está<br /><span style={{ color: accent }}>chegando.</span>
            </Caption>
          </div>
        </AbsoluteFill>
      </Sequence>

      {/* Cena B — widget e previsão */}
      <Sequence from={s(2.3)} durationInFrames={s(5.4)}>
        <AbsoluteFill style={{ opacity: 1 - widgetOut }}>
          <Dock style={{ left: 150, top: 300 }}>
            <Caption from={s(0.1)} to={s(4.6)}>
              <div style={{ fontSize: 76, width: 780 }}>Quanto você já usou.</div>
              <div style={{ fontSize: 76, width: 780, color: C.muted, marginTop: 8 }}>E quando <span style={{ color: accent }}>acaba</span>.</div>
            </Caption>
          </Dock>
          <Dock style={{ right: 150, top: 250, transform: `translateX(${(1 - widgetIn) * 140}px) scale(${0.94 + widgetIn * 0.06})`, opacity: widgetIn }}>
            <Widget frame={frame - s(2.3)} accent={accent} calloutAt={s(3.7)} />
          </Dock>
        </AbsoluteFill>
      </Sequence>

      {/* Cena C — bandeja + cor */}
      <Sequence from={s(7.2)} durationInFrames={s(4.8)}>
        <AbsoluteFill style={{ opacity: trayIn * (1 - trayOut) }}>
          <Dock style={{ left: 150, top: 250 }}>
            <Caption from={0} to={s(4.3)}>
              <div style={{ fontSize: 76, width: 780 }}>Na bandeja.</div>
              <div style={{ fontSize: 76, width: 780, color: C.muted, marginTop: 8 }}>Avisa antes de <span style={{ color: accent }}>estourar</span>.</div>
            </Caption>
            <div style={{ marginTop: 56 }}><Swatches frame={frame - s(7.2) - s(2.4)} accent={accent} /></div>
          </Dock>
          <Dock style={{ right: 150, bottom: 150 }}><Tray frame={frame - s(7.2)} accent={accent} /></Dock>
        </AbsoluteFill>
      </Sequence>

      {/* Cena D — fechamento */}
      <Sequence from={s(11.8)}>
        <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: endIn, transform: `translateY(${(1 - endIn) * 20}px)` }}>
          <div style={{ display: "flex", alignItems: "center", gap: 28 }}>
            <svg width="120" height="120" viewBox="0 0 1024 1024"><rect width="1024" height="1024" rx="224" fill="#181c22" /><circle cx="512" cy="512" r="300" fill="none" stroke="#2d333b" strokeWidth="88" /><path d="M512 212 A300 300 0 1 1 252 662" fill="none" stroke={accent} strokeWidth="88" strokeLinecap="round" /><rect x="496" y="150" width="32" height="150" rx="16" fill="#e6edf3" /></svg>
            <div style={{ fontSize: 140, fontWeight: 600, letterSpacing: -5 }}>Pacer</div>
          </div>
          <div style={{ marginTop: 28, fontSize: 40, color: C.muted }}>Grátis · Código aberto · Windows 10/11</div>
          <div style={{ marginTop: 44, padding: "22px 44px", borderRadius: 18, background: accent, fontSize: 36, fontWeight: 500, color: "#fff", boxShadow: `0 0 100px -10px ${accent}aa` }}>Baixar agora</div>
        </AbsoluteFill>
      </Sequence>

      <Audio src={staticFile("narracao.mp3")} volume={1} />
      <Audio src={staticFile("trilha.mp3")} volume={musicVol} />
    </AbsoluteFill>
  );
};
