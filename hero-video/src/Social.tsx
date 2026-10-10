import { AbsoluteFill, Audio, Easing, Sequence, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig } from "remotion";
import type { ReactNode } from "react";
import { ACCENTS, Bar, C, Cursor, FONT, MONO, Panel, Ring, Taskbar, Toast, TrayPopup, clamp, ease, inOut, s } from "./Hero";

const FPS = 30;
export const SOCIAL_DURATION = s(14.0);

// Zona segura (px): a interface do TikTok/Reels cobre o topo, a base e a lateral direita.
const SAFE = { top: 190, bottom: 1480 };

/** Cortes secos: cada cena só existe entre a e b (segundos). `lt` = segundos locais. */
const Scene = ({ a, b, children }: { a: number; b: number; children: (lt: number, f: number) => ReactNode }) => {
  const frame = useCurrentFrame();
  if (frame < s(a) || frame >= s(b)) return null;
  const f = frame - s(a);
  const punch = interpolate(f, [0, 8], [1.07, 1], { ...clamp, easing: ease });
  return (
    <AbsoluteFill style={{ transform: `scale(${punch})` }}>
      {children(f / FPS, f)}
      {/* flash do corte */}
      <AbsoluteFill style={{ background: "#fff", opacity: interpolate(f, [0, 4], [0.22, 0], clamp), pointerEvents: "none" }} />
    </AbsoluteFill>
  );
};

/** Legenda grande estilo TikTok: pop com mola, contorno escuro para ler sobre qualquer fundo. */
const Cap = ({ lines, top = SAFE.top + 20, size = 120 }: { lines: ReactNode[]; top?: number; size?: number }) => {
  const f = useCurrentFrame();
  const { fps } = useVideoConfig();
  return (
    <div style={{ position: "absolute", left: 40, right: 140, top, textAlign: "center", fontFamily: FONT, fontWeight: 800, fontSize: size, lineHeight: 1.02, letterSpacing: -3, color: "#fff", textShadow: "0 6px 0 rgba(0,0,0,.35), 0 0 40px rgba(0,0,0,.6)" }}>
      {lines.map((l, i) => {
        const k = spring({ frame: f % 100000, fps, config: { damping: 12, stiffness: 180 }, delay: 0 });
        return <div key={i} style={{ transform: `scale(${0.82 + 0.18 * Math.min(k, 1.15)})`, opacity: Math.min(k * 1.6, 1) }}>{l}</div>;
      })}
    </div>
  );
};

const Hl = ({ c, children }: { c: string; children: ReactNode }) => <span style={{ color: c }}>{children}</span>;

const accentAt = (t: number) => {
  const k = Math.max(0, t - 8.9);
  if (t < 8.9 || t >= 10.2) return ACCENTS[0];
  return ACCENTS[Math.min(3, Math.floor(k / 0.33))];
};

/** Recorte da área de trabalho (x 840–1920, y 440–1080): popup + barra de tarefas. */
const TrayRegion = ({ lt, accent }: { lt: number; accent: string }) => {
  const { fps } = useVideoConfig();
  const pop = spring({ frame: Math.round((lt - 0.25) * fps), fps, config: { damping: 15, stiffness: 140 } });
  const pulse = ((lt * 30) % 28) / 28;
  const cx = interpolate(lt, [0, 0.4], [330, 708], { ...clamp, easing: inOut });
  const cy = interpolate(lt, [0, 0.4], [250, 590], { ...clamp, easing: inOut });
  return (
    <div style={{ position: "relative", width: 1080, height: 660, overflow: "hidden" }}>
      <Taskbar iconPulse={pulse} hover={Math.min(1, Math.max(0, (lt - 0.35) * 6))} />
      <div style={{ position: "absolute", left: 445, top: 238, opacity: Math.min(pop, 1), transform: `translateY(${(1 - Math.min(pop, 1)) * 40}px)` }}>
        <TrayPopup accent={accent} />
      </div>
      <Cursor x={cx} y={cy} press={0} />
    </div>
  );
};

const Chip = ({ children, at, lt, accent }: { children: ReactNode; at: number; lt: number; accent: string }) => {
  const { fps } = useVideoConfig();
  const k = spring({ frame: Math.round((lt - at) * fps), fps, config: { damping: 13, stiffness: 150 } });
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 28, padding: "30px 46px", borderRadius: 28, background: C.card, border: `2px solid ${accent}88`, fontFamily: FONT, fontSize: 64, fontWeight: 800, opacity: Math.min(k, 1), transform: `translateX(${(1 - Math.min(k, 1)) * 200}px) scale(${0.9 + 0.1 * Math.min(k, 1.1)})`, boxShadow: `0 0 90px -30px ${accent}` }}>
      <span style={{ width: 70, height: 70, borderRadius: 99, background: accent, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 44, color: "#fff" }}>✓</span>
      {children}
    </div>
  );
};

export const Social = () => {
  const frame = useCurrentFrame();
  const t = frame / FPS;
  const accent = accentAt(t);

  const drift = (frame / SOCIAL_DURATION) * 64;
  const cuts = [2.1, 6.0, 7.1, 8.8, 10.2, 11.8];

  const music = (f: number) => interpolate(f, [0, 15, s(12.8), s(14.0)], [0, 0.2, 0.2, 0], clamp);

  return (
    <AbsoluteFill style={{ background: C.bg, fontFamily: FONT, color: C.text, overflow: "hidden" }}>
      {/* fundo */}
      <AbsoluteFill style={{ backgroundImage: "linear-gradient(to right, rgba(230,237,243,.05) 1px, transparent 1px), linear-gradient(to bottom, rgba(230,237,243,.05) 1px, transparent 1px)", backgroundSize: "64px 64px", backgroundPosition: `${drift}px ${drift}px`, WebkitMaskImage: "radial-gradient(ellipse at 50% 40%, #000 25%, transparent 78%)" }} />
      <AbsoluteFill style={{ background: `radial-gradient(70% 45% at 50% 62%, ${accent}55, transparent 70%)` }} />

      {/* 1 — gancho */}
      <Scene a={0} b={2.1}>
        {(lt) => {
          const slam = interpolate(lt, [0.08, 0.7], [0, 1], { ...clamp, easing: Easing.out(Easing.back(1.6)) });
          const val = Math.round(97 * Math.min(slam, 1));
          const red = val >= 90;
          const shake = lt < 0.9 ? Math.sin(lt * 90) * (1 - lt / 0.9) * 14 : 0;
          return (
            <AbsoluteFill style={{ transform: `translate(${shake}px, ${shake * 0.6}px)` }}>
              <AbsoluteFill style={{ background: `radial-gradient(60% 40% at 50% 62%, ${red ? C.crit : accent}44, transparent 70%)` }} />
              <Cap lines={["Seu limite do", "Claude vai", <Hl key="a" c={C.crit}>acabar?</Hl>]} size={124} />
              <div style={{ position: "absolute", left: 0, right: 100, top: 780, display: "flex", justifyContent: "center", alignItems: "center" }}>
                <svg width="720" height="720" viewBox="0 0 720 720" style={{ position: "absolute" }}>
                  <circle cx="360" cy="360" r="300" fill="none" stroke="#2d333b" strokeWidth="46" />
                  <circle cx="360" cy="360" r="300" fill="none" stroke={red ? C.crit : C.warn} strokeWidth="46" strokeLinecap="round" strokeDasharray={2 * Math.PI * 300} strokeDashoffset={2 * Math.PI * 300 * (1 - 0.82 * slam)} transform="rotate(-90 360 360)" />
                </svg>
                <div style={{ fontFamily: MONO, fontWeight: 700, fontSize: 210, color: red ? C.crit : "#fff", letterSpacing: -8 }}>{val}%</div>
              </div>
            </AbsoluteFill>
          );
        }}
      </Scene>

      {/* 2+3 — painel: "quanto você já usou" → "nesse ritmo, acaba em 4h 57m" */}
      <Scene a={2.1} b={6.0}>
        {(lt, f) => {
          const k2 = interpolate(lt, [2.5, 2.8], [0, 1], { ...clamp, easing: Easing.out(Easing.back(1.4)) });
          const sc = 1.7 + 0.6 * k2;
          const ty = (440 * sc - 450) * k2;
          const pulse = lt > 2.5 ? 1 + 0.04 * Math.sin((lt - 2.5) * 14) * Math.max(0, 1 - (lt - 2.5) / 1.2) : 1;
          return (
            <AbsoluteFill>
              {lt < 2.5 ? (
                <Cap lines={["Veja quanto", <Hl key="a" c={accent}>você já usou</Hl>]} />
              ) : (
                <Cap lines={["Nesse ritmo,", <Hl key="a" c={C.crit}>acaba em 4h 57m</Hl>]} size={104} />
              )}
              <div style={{ position: "absolute", left: 0, right: 0, top: 560, height: SAFE.bottom - 560, overflow: "hidden" }}>
                <div style={{ position: "absolute", left: 220, top: 0, transformOrigin: "50% 0", transform: `translateY(${-ty}px) scale(${sc * pulse})` }}>
                  <Panel t={t} accent={accent} frame={s(3.0) + Math.round(f * 1.5)} />
                </div>
              </div>
            </AbsoluteFill>
          );
        }}
      </Scene>

      {/* 4 — bandeja */}
      <Scene a={6.0} b={7.1}>
        {(lt) => (
          <AbsoluteFill>
            <Cap lines={["Na bandeja,", <Hl key="a" c={accent}>passe o mouse</Hl>]} size={108} />
            <div style={{ position: "absolute", left: 0, top: 640, transform: "scale(1.6)", transformOrigin: "100% 100%" }}>
              <TrayRegion lt={lt} accent={accent} />
            </div>
          </AbsoluteFill>
        )}
      </Scene>

      {/* 5 — notificação */}
      <Scene a={7.1} b={8.8}>
        {(lt) => {
          const k = spring({ frame: Math.round((lt - 0.1) * 30), fps: 30, config: { damping: 14, stiffness: 120 } });
          const shake = lt > 0.3 && lt < 0.55 ? Math.sin(lt * 120) * 8 : 0;
          return (
            <AbsoluteFill>
              <AbsoluteFill style={{ background: `radial-gradient(55% 28% at 45% 48%, ${C.crit}44, transparent 70%)` }} />
              <Cap lines={["Avisa antes", <Hl key="a" c={C.warn}>de estourar</Hl>]} />
              <div style={{ position: "absolute", left: 40, top: 760, width: 540, transformOrigin: "0 0", transform: `translate(${(1 - Math.min(k, 1)) * 1100 + shake}px, 0) scale(1.82)` }}>
                <Toast accent={accent} />
              </div>
              <div style={{ position: "absolute", left: 60, right: 160, top: 1190, textAlign: "center", fontSize: 44, color: C.muted, fontFamily: FONT, opacity: Math.min(1, Math.max(0, (lt - 0.6) * 3)) }}>
                Notificação do Windows em 80% e 95%
              </div>
            </AbsoluteFill>
          );
        }}
      </Scene>

      {/* 6 — cor */}
      <Scene a={8.8} b={10.2}>
        {() => (
          <AbsoluteFill>
            <Cap lines={["Combina com", <Hl key="a" c={accent}>a sua cor</Hl>]} size={112} />
            <div style={{ position: "absolute", left: 40, right: 140, top: 640, display: "flex", justifyContent: "center", gap: 34 }}>
              {ACCENTS.map((c) => (
                <div key={c} style={{ width: 150, height: 150, borderRadius: 99, background: c, boxShadow: accent === c ? `0 0 0 8px ${C.bg}, 0 0 0 14px #fff, 0 0 90px ${c}` : "none", transform: `scale(${accent === c ? 1.15 : 0.9})` }} />
              ))}
            </div>
            <div style={{ position: "absolute", left: 90, right: 190, top: 930, display: "grid", gap: 36 }}>
              <div style={{ background: "#12171d", border: `1px solid ${C.border}`, borderRadius: 28, padding: 40 }}>
                <div style={{ display: "flex", justifyContent: "space-between", fontFamily: FONT, fontSize: 36, marginBottom: 20 }}><b>Sessão (5h)</b><span style={{ color: C.muted }}>12% usado</span></div>
                <Bar pct={12} forecast={46} color={accent} h={26} />
              </div>
              <div style={{ background: "#12171d", border: `1px solid ${C.border}`, borderRadius: 28, padding: 40 }}>
                <div style={{ display: "flex", justifyContent: "space-between", fontFamily: FONT, fontSize: 36, marginBottom: 20 }}><b>Semanal</b><span style={{ color: C.crit, fontWeight: 700 }}>vermelho fica fixo</span></div>
                <Bar pct={97} forecast={100} color={C.crit} h={26} />
              </div>
            </div>
          </AbsoluteFill>
        )}
      </Scene>

      {/* 7 — grátis e código aberto */}
      <Scene a={10.2} b={11.8}>
        {(lt) => (
          <AbsoluteFill>
            <Cap lines={[<Hl key="a" c={accent}>Grátis</Hl>, "e código aberto"]} size={116} />
            <div style={{ position: "absolute", left: 60, right: 160, top: 700, display: "grid", gap: 34 }}>
              <Chip at={0.05} lt={lt} accent={accent}>Windows 10/11</Chip>
              <Chip at={0.4} lt={lt} accent={accent}>Código aberto · MIT</Chip>
              <Chip at={0.75} lt={lt} accent={accent}>Sem telemetria</Chip>
            </div>
          </AbsoluteFill>
        )}
      </Scene>

      {/* 8 — CTA */}
      <Scene a={11.8} b={14.0}>
        {(lt) => {
          const k = interpolate(lt, [0, 0.4], [0, 1], { ...clamp, easing: ease });
          const bounce = Math.sin(lt * 9) * 18;
          const glow = 60 + Math.sin(lt * 8) * 30;
          return (
            <AbsoluteFill style={{ alignItems: "center", opacity: k }}>
              <div style={{ position: "absolute", top: 260, display: "flex", flexDirection: "column", alignItems: "center", gap: 10, width: 940 }}>
                <Ring size={280} color={accent} stroke={88} />
                <div style={{ fontSize: 190, fontWeight: 800, letterSpacing: -9, fontFamily: FONT }}>Pacer</div>
                <div style={{ fontSize: 46, color: C.muted }}>Grátis · Código aberto · Windows 10/11</div>
              </div>
              <div style={{ position: "absolute", top: 940, width: 860, height: 150, borderRadius: 34, background: accent, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 64, fontWeight: 800, color: "#fff", boxShadow: `0 0 ${glow}px ${accent}`, transform: `scale(${1 + Math.sin(lt * 8) * 0.025})` }}>
                Baixar agora
              </div>
              <div style={{ position: "absolute", top: 1130 + bounce, fontSize: 130, color: accent, fontFamily: "DejaVu Sans" }}>↓</div>
              <div style={{ position: "absolute", top: 1290, fontSize: 84, fontWeight: 800, letterSpacing: -2, textShadow: "0 6px 0 rgba(0,0,0,.35)" }}>
                <Hl c={accent}>Link</Hl> na bio
              </div>
            </AbsoluteFill>
          );
        }}
      </Scene>

      {/* áudio */}
      <Audio src={staticFile("social-voz.mp3")} volume={1} />
      <Audio src={staticFile("trilha.mp3")} volume={music} />
      <Sequence from={s(0.05)} durationInFrames={s(1)}><Audio src={staticFile("sfx-impacto.mp3")} volume={0.8} /></Sequence>
      {cuts.map((c) => (
        <Sequence key={c} from={s(c) - 3} durationInFrames={s(0.8)}><Audio src={staticFile("sfx-whoosh.mp3")} volume={0.45} /></Sequence>
      ))}
    </AbsoluteFill>
  );
};
