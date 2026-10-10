import { AbsoluteFill, Audio, Easing, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import type { CSSProperties, ReactNode } from "react";

export const FPS = 30;
const s = (sec: number) => Math.round(sec * FPS);
export const DURATION = s(21.2);

// Tokens do app/landing.
const C = { bg: "#0d1117", card: "#151b23", card2: "#10151c", surface: "#1c232d", border: "#272f3a", text: "#e6edf3", muted: "#8b949e", warn: "#d29922", crit: "#f85149" };
const ACCENTS = ["#1f6feb", "#8957e5", "#2ea043", "#db61a2"]; // Azul, Roxo, Verde, Rosa (presets do app)
const FONT = `Inter, "Segoe UI", system-ui, sans-serif`;
const MONO = `"DejaVu Sans Mono", ui-monospace, monospace`;

const ease = Easing.bezier(0.16, 1, 0.3, 1);
const inOut = Easing.inOut(Easing.cubic);
const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
/** progresso 0→1 entre dois instantes (em segundos) */
const P = (f: number, a: number, b: number, e: (t: number) => number = ease) => interpolate(f, [s(a), s(b)], [0, 1], { ...clamp, easing: e });

/** Cor de destaque: azul; troca durante "combina com a sua cor" (13,9–15,2 s). */
function accentAt(t: number) {
  if (t < 13.9) return ACCENTS[0];
  if (t < 14.3) return ACCENTS[1];
  if (t < 14.7) return ACCENTS[2];
  if (t < 15.1) return ACCENTS[3];
  return ACCENTS[0];
}

const toneOf = (pct: number, accent: string) => (pct >= 90 ? C.crit : pct >= 75 ? C.warn : accent);

/* ---------- peças ---------- */

const Ring = ({ size, color, stroke }: { size: number; color: string; stroke: number }) => (
  <svg width={size} height={size} viewBox="0 0 1024 1024">
    <circle cx="512" cy="512" r="300" fill="none" stroke="#2d333b" strokeWidth={stroke} />
    <path d="M512 212 A300 300 0 1 1 252 662" fill="none" stroke={color} strokeWidth={stroke} strokeLinecap="round" />
    <rect x="496" y="150" width="32" height="150" rx="16" fill="#e6edf3" />
  </svg>
);

const Bar = ({ pct, forecast, color, h = 12 }: { pct: number; forecast: number; color: string; h?: number }) => (
  <div style={{ position: "relative", height: h, borderRadius: 99, background: "#2a313b" }}>
    <div style={{ height: "100%", width: `${pct}%`, borderRadius: 99, background: color }} />
    <div style={{ position: "absolute", top: -h * 0.45, left: `${Math.min(forecast, 99.4)}%`, width: 3, height: h * 1.9, borderRadius: 2, background: "rgba(230,237,243,.85)" }} />
  </div>
);

/** Painel do app (print 1), com dados fictícios. `t` = segundos do vídeo. */
const Panel = ({ t, accent, frame }: { t: number; accent: string; frame: number }) => {
  const sess = 12 * P(frame, 3.0, 4.2);
  const weekly = 97 * P(frame, 3.4, 6.0, Easing.inOut(Easing.quad));
  const wTone = toneOf(weekly, accent);
  const callout = P(frame, 5.6, 6.1);
  const sessForecast = 46 * P(frame, 3.6, 4.6);
  const wForecast = 100 * P(frame, 4.8, 5.6);
  const cells = Array.from({ length: 35 }, (_, i) => ({ i, lvl: [0, 2, 3, 1, 4, 2, 3, 0, 1, 4, 2, 3, 1, 4, 0][(i * 7 + 3) % 15] }));
  const shade = ["#1c232d", "#10315f", "#13438a", "#1f6feb", "#58a6ff"];
  const tint = (lvl: number) => (lvl === 0 ? shade[0] : `color-mix(in oklab, ${accent} ${25 + lvl * 18}%, #151b23)`);
  return (
    <div style={{ width: 520, background: "#12171d", border: `1px solid ${C.border}`, borderRadius: 26, padding: 26, fontFamily: FONT, color: C.text, boxShadow: `0 30px 90px -20px rgba(0,0,0,.7), 0 0 120px -30px ${accent}88` }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div>
          <div style={{ fontSize: 30, fontWeight: 600 }}>Pacer</div>
          <div style={{ fontSize: 17, color: C.muted, marginTop: 4 }}>Uso dos planos de IA</div>
        </div>
        <div style={{ display: "flex", gap: 22, fontSize: 28, color: C.muted, fontFamily: "DejaVu Sans" }}><span>↻</span><span>⚙</span></div>
      </div>
      <div style={{ marginTop: 18, display: "inline-flex", alignItems: "center", gap: 10, padding: "8px 16px", borderRadius: 10, background: C.card, border: `1px solid ${C.border}`, borderBottom: `3px solid ${accent}`, fontSize: 21, fontWeight: 600 }}>
        <span style={{ width: 12, height: 12, borderRadius: 99, background: wTone }} /> {Math.round(weekly)}%
      </div>

      <div style={{ marginTop: 18, background: C.card, border: `1px solid ${C.border}`, borderRadius: 18, padding: 22 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12, fontSize: 24, fontWeight: 600 }}>
          Claude <span style={{ fontSize: 15, fontWeight: 500, color: C.muted, border: `1px solid ${C.border}`, borderRadius: 7, padding: "2px 9px", background: C.surface }}>Pro</span>
        </div>
        <div style={{ marginTop: 22 }}>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 19 }}><b>Sessão (5h)</b><span style={{ color: C.muted }}>Redefine em 3h 41m</span></div>
          <div style={{ marginTop: 12 }}><Bar pct={sess} forecast={sessForecast} color={accent} /></div>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 16, color: C.muted, marginTop: 10 }}><span>{Math.round(sess)}% usado</span><span>~{Math.round(sessForecast)}% na redefinição</span></div>
        </div>
        <div style={{ marginTop: 26 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 19 }}>
            <b>Semanal</b>
            <span style={{ color: C.crit, fontWeight: 700, opacity: callout, transform: `translateX(${(1 - callout) * 16}px)` }}>↗ Nesse ritmo, acaba em 4h 57m</span>
          </div>
          <div style={{ marginTop: 12 }}><Bar pct={weekly} forecast={wForecast} color={wTone} /></div>
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 16, color: C.muted, marginTop: 10 }}><span>{Math.round(weekly)}% usado</span><span>Redefine em 7h 41m</span></div>
        </div>
      </div>

      <div style={{ marginTop: 18, background: C.card, border: `1px solid ${C.border}`, borderRadius: 18, padding: 22 }}>
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: 22, fontWeight: 600 }}>Atividade <span style={{ fontSize: 16, fontWeight: 400, color: C.muted }}>30 dias</span></div>
        <div style={{ marginTop: 16, display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 7 }}>
          {cells.map(({ i, lvl }) => {
            const col = i % 5, row = Math.floor(i / 5);
            const pop = interpolate(frame, [s(4.3) + (col + row) * 2, s(4.3) + (col + row) * 2 + 10], [0, 1], { ...clamp, easing: ease });
            return <div key={i} style={{ height: 22, borderRadius: 5, background: tint(lvl), opacity: pop, transform: `scale(${0.5 + pop * 0.5})` }} />;
          })}
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", fontSize: 16, color: C.muted, marginTop: 14 }}><span>Hoje: 695k tokens</span><span>30 dias: 2,6B</span></div>
      </div>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 15, color: C.muted, marginTop: 16, padding: "0 6px" }}><span>Atualizado agora</span><span>Próxima em 4m</span></div>
      <span style={{ display: "none" }}>{t}</span>
    </div>
  );
};

/** Popup da bandeja (print 2). */
const TrayPopup = ({ accent }: { accent: string }) => (
  <div style={{ width: 600, background: "#12171d", border: `1px solid ${C.border}`, borderRadius: 20, padding: 26, fontFamily: FONT, color: C.text, boxShadow: `0 30px 80px -10px rgba(0,0,0,.8), 0 0 90px -20px ${accent}77` }}>
    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 21 }}><b>Sessão (5h)</b><span style={{ color: C.muted }}>Redefine em 3h 40m</span></div>
    <div style={{ marginTop: 12 }}><Bar pct={12} forecast={46} color={accent} h={14} /></div>
    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 17, color: C.muted, marginTop: 10 }}><span>12% usado</span><span>~45% na redefinição</span></div>
    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 21, marginTop: 24 }}><b>Semanal</b><span style={{ color: C.crit, fontWeight: 700 }}>↗ Nesse ritmo, acaba em 4h 57m</span></div>
    <div style={{ marginTop: 12 }}><Bar pct={97} forecast={100} color={C.crit} h={14} /></div>
    <div style={{ display: "flex", justifyContent: "space-between", fontSize: 17, color: C.muted, marginTop: 10 }}><span>97% usado</span><span>Redefine em 7h 40m</span></div>
  </div>
);

const Taskbar = ({ iconPulse, hover }: { iconPulse: number; hover: number }) => (
  <div style={{ position: "absolute", left: 0, right: 0, bottom: 0, height: 72, background: "rgba(18,22,28,.94)", borderTop: `1px solid ${C.border}`, display: "flex", alignItems: "center", justifyContent: "flex-end", gap: 26, padding: "0 28px", fontFamily: FONT, color: C.text }}>
    <span style={{ color: C.muted, fontSize: 22, fontFamily: "DejaVu Sans" }}>⌃</span>
    <div style={{ width: 26, height: 26, borderRadius: 99, background: "#2a313b" }} />
    <div style={{ position: "relative", width: 52, height: 52, borderRadius: 12, display: "flex", alignItems: "center", justifyContent: "center", background: `rgba(255,255,255,${0.04 + hover * 0.1})`, border: `1px solid rgba(255,255,255,${0.06 + hover * 0.14})` }}>
      <div style={{ position: "absolute", inset: -8 - iconPulse * 26, borderRadius: 24, border: `3px solid ${C.crit}`, opacity: (1 - iconPulse) * 0.9 }} />
      <svg width="34" height="34" viewBox="0 0 34 34"><circle cx="17" cy="17" r="12" fill="none" stroke={C.crit} strokeWidth="4.5" /></svg>
    </div>
    <div style={{ width: 26, height: 26, borderRadius: 6, background: "#2a313b" }} />
    <div style={{ fontSize: 15, color: C.muted, lineHeight: 1.25, textAlign: "center" }}>POR<br />PTB2</div>
    <div style={{ width: 30, height: 26, borderRadius: 6, background: "#2a313b" }} />
    <div style={{ fontSize: 17, lineHeight: 1.25, textAlign: "right" }}>14:32<br />10/10/2026</div>
  </div>
);

const Toast = ({ accent: _a }: { accent: string }) => (
  <div style={{ width: 540, background: "#1b2129", border: `1px solid ${C.border}`, borderRadius: 16, padding: "20px 24px", display: "flex", gap: 20, alignItems: "center", fontFamily: FONT, color: C.text, boxShadow: "0 24px 60px -10px rgba(0,0,0,.8)" }}>
    <Ring size={64} color={C.crit} stroke={88} />
    <div>
      <div style={{ fontSize: 16, color: C.muted }}>Pacer</div>
      <div style={{ fontSize: 24, fontWeight: 600, marginTop: 2 }}>Semanal em 95%</div>
      <div style={{ fontSize: 19, color: C.crit, marginTop: 2 }}>Nesse ritmo, acaba em 4h 57m</div>
    </div>
  </div>
);

const Cursor = ({ x, y, press }: { x: number; y: number; press: number }) => (
  <svg width="44" height="44" viewBox="0 0 24 24" style={{ position: "absolute", left: x, top: y, transform: `scale(${1 - press * 0.15})`, filter: "drop-shadow(0 4px 6px rgba(0,0,0,.6))", zIndex: 50 }}>
    <path d="M4 2 L4 19 L8.6 14.8 L11.6 21.6 L14.2 20.4 L11.2 13.7 L17.5 13.4 Z" fill="#fff" stroke="#111" strokeWidth="1.2" strokeLinejoin="round" />
  </svg>
);

const Caption = ({ children, from, to, style }: { children: ReactNode; from: number; to: number; style?: CSSProperties }) => {
  const f = useCurrentFrame();
  const i = P(f, from, from + 0.5), o = P(f, to - 0.3, to);
  return (
    <div style={{ position: "absolute", fontFamily: FONT, fontWeight: 600, letterSpacing: -2, lineHeight: 1.05, opacity: i * (1 - o), transform: `translateY(${(1 - i) * 26 - o * 14}px)`, ...style }}>
      {children}
    </div>
  );
};

const Chip = ({ children, at, accent }: { children: ReactNode; at: number; accent: string }) => {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  const k = spring({ frame: f - s(at), fps, config: { damping: 14, stiffness: 120 } });
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 22, padding: "26px 40px", borderRadius: 22, background: C.card, border: `1px solid ${C.border}`, fontFamily: FONT, fontSize: 46, fontWeight: 600, opacity: Math.min(k, 1), transform: `translateY(${(1 - k) * 60}px) scale(${0.9 + k * 0.1})`, boxShadow: `0 0 80px -30px ${accent}` }}>
      <span style={{ width: 54, height: 54, borderRadius: 99, background: accent, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 32, color: "#fff" }}>✓</span>
      {children}
    </div>
  );
};

/* ---------- composição ---------- */

export const Hero = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / FPS;
  const accent = accentAt(t);

  const drift = (frame / DURATION) * 64;
  const intro = P(frame, 0, 1.2);

  // Cena 1 — gancho: anel
  const ring = P(frame, 0, 2.0);
  const ringOut = P(frame, 0, 0.5) * (1 - P(frame, 2.0, 2.8));

  // Painel e área de trabalho
  const panelIn = spring({ frame: frame - s(2.4), fps, config: { damping: 18, stiffness: 90 } });
  const desktopIn = P(frame, 7.4, 8.3);
  const desktopOut = P(frame, 15.3, 16.0);
  const stageAlpha = 1 - desktopOut; // painel + desktop somem juntos para as cenas finais

  // Câmera: zoom na bandeja (9,1–9,9) e volta (11,9–12,6)
  const zk = P(frame, 9.0, 9.9, inOut) - P(frame, 11.9, 12.6, inOut);
  const zoom = 1 + 0.8 * zk;
  const panelDim = 1 - 0.75 * zk;

  // Cursor até o ícone da bandeja (1581, 1044 no desktop)
  const cx = interpolate(frame, [s(8.9), s(10.0), s(11.8), s(12.5)], [1050, 1549, 1549, 1250], { ...clamp, easing: inOut });
  const cy = interpolate(frame, [s(8.9), s(10.0), s(11.8), s(12.5)], [620, 1040, 1040, 700], { ...clamp, easing: inOut });
  const cursorShow = P(frame, 8.9, 9.2) * (1 - P(frame, 12.3, 12.7));
  const hover = P(frame, 10.0, 10.2);
  const pulse = interpolate((frame - s(10.0)) % 28, [0, 28], [0, 1], clamp) * (t > 10.0 && t < 11.9 ? 1 : 0);
  const popup = spring({ frame: frame - s(10.1), fps, config: { damping: 16, stiffness: 130 } }) * (1 - P(frame, 11.9, 12.3));

  const toast = P(frame, 12.3, 12.9) * (1 - P(frame, 14.9, 15.4));

  // Cena 6 — chips, Cena 7 — CTA
  const chipsOut = 1 - P(frame, 17.2, 17.7);
  const cta = P(frame, 17.5, 18.4);
  const btnClick = P(frame, 19.15, 19.3) * (1 - P(frame, 19.3, 19.45));
  const dl = P(frame, 19.5, 20.5, Easing.inOut(Easing.quad));
  const cx2 = interpolate(frame, [s(18.3), s(19.15)], [1350, 950], { ...clamp, easing: inOut });
  const cy2 = interpolate(frame, [s(18.3), s(19.15)], [880, 690], { ...clamp, easing: inOut });
  const cursor2 = P(frame, 18.3, 18.6) * (1 - P(frame, 20.8, 21.1));

  const musicVol = (f: number) => interpolate(f, [0, 20, s(1), s(19.5), s(21.0)], [0, 0.22, 0.14, 0.18, 0], clamp);

  return (
    <AbsoluteFill style={{ background: C.bg, fontFamily: FONT, color: C.text, overflow: "hidden" }}>
      {/* fundo */}
      <AbsoluteFill style={{ opacity: intro, backgroundImage: "linear-gradient(to right, rgba(230,237,243,.05) 1px, transparent 1px), linear-gradient(to bottom, rgba(230,237,243,.05) 1px, transparent 1px)", backgroundSize: "64px 64px", backgroundPosition: `${drift}px ${drift}px`, WebkitMaskImage: "radial-gradient(ellipse at 50% 30%, #000 25%, transparent 75%)" }} />
      <AbsoluteFill style={{ background: `radial-gradient(60% 55% at ${30 + Math.sin(frame / 90) * 6}% 85%, ${accent}66, transparent 70%)` }} />

      {/* Cena 1 — gancho */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: ringOut }}>
        <svg width="760" height="760" viewBox="0 0 760 760" style={{ position: "absolute", transform: `scale(${0.92 + ring * 0.08})` }}>
          <circle cx="380" cy="380" r="330" fill="none" stroke="#2d333b" strokeWidth="26" />
          <circle cx="380" cy="380" r="330" fill="none" stroke={ring > 0.7 ? C.warn : accent} strokeWidth="26" strokeLinecap="round" strokeDasharray={2 * Math.PI * 330} strokeDashoffset={2 * Math.PI * 330 * (1 - ring * 0.82)} transform="rotate(-90 380 380)" />
          <rect x="373" y="26" width="14" height="62" rx="7" fill={C.text} transform={`rotate(${ring * 295} 380 380)`} />
        </svg>
        <div style={{ position: "absolute", fontSize: 96, textAlign: "center" }}>
          <Caption from={0.15} to={2.5} style={{ position: "relative" }}>Seu limite está<br /><span style={{ color: C.warn }}>chegando.</span></Caption>
        </div>
      </AbsoluteFill>

      {/* Desktop + painel + bandeja (com câmera) */}
      <AbsoluteFill style={{ opacity: stageAlpha }}>
        <AbsoluteFill style={{ transform: `translate(${-300 * zk}px, ${-324 * zk}px) scale(${zoom})`, transformOrigin: "1556px 1044px" }}>
          {/* papel de parede */}
          <AbsoluteFill style={{ opacity: desktopIn, background: "radial-gradient(70% 80% at 25% 20%, #1c2a36 0%, #0c1319 70%)" }}>
            <AbsoluteFill style={{ background: "radial-gradient(40% 50% at 80% 30%, rgba(88,166,255,.10), transparent 70%)" }} />
          </AbsoluteFill>
          <div style={{ position: "absolute", inset: 0, opacity: desktopIn, transform: `translateY(${(1 - desktopIn) * 72}px)` }}><Taskbar iconPulse={pulse} hover={hover} /></div>

          {/* painel do app na área de trabalho */}
          <div style={{ position: "absolute", left: 1230, top: 56, opacity: panelIn * panelDim, transform: `translateX(${(1 - panelIn) * 160}px) scale(${0.95 + panelIn * 0.05})`, transformOrigin: "top right" }}>
            <Panel t={t} accent={accent} frame={frame} />
          </div>

          {/* popup da bandeja */}
          <div style={{ position: "absolute", left: 1285, top: 668, opacity: Math.min(popup, 1), transform: `translateY(${(1 - popup) * 40}px)` }}>
            <TrayPopup accent={accent} />
          </div>

          {/* notificação */}
          <div style={{ position: "absolute", left: 1350, top: 880, opacity: toast, transform: `translateX(${(1 - toast) * 120}px)` }}>
            <Toast accent={accent} />
          </div>

          <div style={{ opacity: cursorShow }}><Cursor x={cx} y={cy} press={P(frame, 10.0, 10.1) * (1 - P(frame, 10.1, 10.25)) * 0} /></div>
        </AbsoluteFill>
      </AbsoluteFill>

      {/* legendas (fora da câmera) */}
      <Caption from={2.45} to={4.7} style={{ left: 150, top: 330, fontSize: 84, width: 900 }}>Quanto você<br />já usou.</Caption>
      <Caption from={4.75} to={7.45} style={{ left: 150, top: 330, fontSize: 84, width: 900 }}>E quando<br /><span style={{ color: C.crit }}>acaba.</span></Caption>
      <Caption from={7.6} to={9.3} style={{ left: 150, top: 330, fontSize: 84, width: 900 }}>Na sua área<br />de trabalho.</Caption>
      <Caption from={9.4} to={12.1} style={{ left: 150, top: 300, fontSize: 84, width: 900 }}>Na bandeja,<br /><span style={{ color: accent }}>passe o mouse.</span></Caption>
      <Caption from={12.25} to={13.85} style={{ left: 150, top: 330, fontSize: 84, width: 900 }}>Avisa antes<br /><span style={{ color: C.warn }}>de estourar.</span></Caption>
      <Caption from={13.95} to={15.5} style={{ left: 150, top: 330, fontSize: 84, width: 900 }}>E combina com<br /><span style={{ color: accent }}>a sua cor.</span></Caption>

      {/* Cena 6 — Grátis e código aberto */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: chipsOut * P(frame, 15.5, 15.8) }}>
        <div style={{ display: "grid", gap: 26, justifyItems: "center" }}>
          <Chip at={15.65} accent={accent}>Grátis</Chip>
          <Chip at={16.15} accent={accent}>Código aberto · MIT</Chip>
          <Chip at={16.65} accent={accent}>Sem telemetria</Chip>
        </div>
      </AbsoluteFill>

      {/* Cena 7 — CTA */}
      <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", opacity: cta, transform: `translateY(${(1 - cta) * 30}px)` }}>
        <div style={{ display: "flex", alignItems: "center", gap: 28 }}>
          <Ring size={130} color={accent} stroke={88} />
          <div style={{ fontSize: 150, fontWeight: 600, letterSpacing: -6 }}>Pacer</div>
        </div>
        <div style={{ marginTop: 18, fontSize: 40, color: C.muted }}>Baixe agora e comece a usar em dois minutos.</div>
        <div style={{ position: "relative", marginTop: 52, width: 640, height: 96 }}>
          <div style={{ position: "absolute", inset: 0, borderRadius: 22, background: dl > 0.01 ? C.surface : accent, border: dl > 0.01 ? `1px solid ${C.border}` : "none", boxShadow: `0 0 ${60 + Math.sin(frame / 6) * 30}px -8px ${accent}cc`, transform: `scale(${1 - btnClick * 0.04})`, overflow: "hidden", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 38, fontWeight: 600, color: "#fff" }}>
            {dl <= 0.01 ? "▦  Baixar para Windows" : (
              <>
                <div style={{ position: "absolute", left: 0, top: 0, bottom: 0, width: `${dl * 100}%`, background: accent, opacity: 0.9 }} />
                <span style={{ position: "relative", fontSize: 28 }}>{dl < 1 ? `Baixando Pacer_2.1.0_x64-setup.exe  ${Math.round(dl * 100)}%` : "✓  Pronto — 2,2 MB"}</span>
              </>
            )}
          </div>
        </div>
        <div style={{ marginTop: 26, fontSize: 28, color: C.muted, fontFamily: MONO }}>github.com/sthevan027/pacer</div>
      </AbsoluteFill>
      <div style={{ opacity: cursor2 }}><Cursor x={cx2} y={cy2} press={btnClick} /></div>

      <Audio src={staticFile("narracao.mp3")} volume={1} />
      <Audio src={staticFile("trilha.mp3")} volume={musicVol} />
    </AbsoluteFill>
  );
};
