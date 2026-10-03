# Pacer (etapa A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recriar o Claude Glass como **Pacer** — widget flutuante para Windows (Tauri 2 + React) que mostra o uso do plano Claude, a projeção do ritmo até a redefinição e a atividade dos últimos 30 dias.

**Architecture:** Backend Rust faz todo o trabalho (credenciais, API, logs, projeção, agendamento, alertas, janela, bandeja) e emite um `Snapshot` por provedor; o React só desenha. Provedores implementam um trait comum para que a etapa B adicione outros sem mexer na tela.

**Tech Stack:** Tauri 2, Rust (serde, chrono, reqwest/rustls, tokio, async-trait, window-vibrancy), React + TypeScript + Vite, Bun, Vitest, mockito, tempfile.

**Spec:** `docs/superpowers/specs/2026-10-03-pacer-design.md`

## Global Constraints

- Nome do produto: **Pacer**; identificador: `dev.sthevan.pacer`; versão: **2.0.0**.
- Branch de trabalho: `pacer` em `D:\Projetos\Claude-Glass` (nunca trabalhar no `C:`).
- Gerenciador JS: **Bun** (`bun install`, `bun add`, `bun run`, `bunx`). Sem `package-lock.json`.
- Interface em **pt-BR**; sem emojis na UI; ícones de traço; um único azul de destaque `#1f6feb`.
- Credenciais do Claude Code são **somente leitura**: nunca renovar token nem escrever em `~/.claude/.credentials.json`.
- Token e arquivos nunca chegam ao WebView: o React só recebe `Snapshot`/`Config`.
- API: `GET https://api.anthropic.com/api/oauth/usage`, `anthropic-beta: oauth-2025-04-20`, intervalo 1/5/10 min (padrão 5), nunca mais de 1 chamada por minuto, backoff dobrando até 30 min em 429.
- Logs: `~/.claude/projects/**/*.jsonl` (respeita `CLAUDE_CONFIG_DIR`), 30 dias, deduplicação por `message.id:requestId`.
- Cores de severidade: azul `< 75%`, laranja `75–90%`, vermelho `> 90%` ou projeção `> 100%`.
- Config em `%APPDATA%\dev.sthevan.pacer\config.json`, gravação atômica, arquivo inválido → padrões + `.bak`.
- Commits terminam com `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **`.credentials.json` sendo reescrito pelo Claude Code no meio da leitura** (JSON parcial) → o Pacer mantém o último dado e não mostra "faça login". Teste na Task 8.
2. **Virada de dia no fuso local** (UTC−3: uso às 22h local cai no dia local, não no dia UTC seguinte) → mapa e "Hoje" corretos. Teste na Task 7.
3. **Resposta da API com janelas `null`, campos desconhecidos ou `utilization` ausente** (`seven_day_opus: null`, `extra_usage`, `tangelo`) → sem crash, janela omitida. Teste na Task 6.
4. **Barra de tarefas no topo/lateral ou segundo monitor com coordenadas deslocadas** → widget ancorado dentro da área útil daquele monitor. Teste na Task 11.
5. **Pasta de logs grande e sem mudanças, varrida a cada 10 s** → só arquivos alterados são relidos; arquivos apagados saem do cache. Teste na Task 7.

---

## File Structure

```
package.json, bun.lock, index.html, vite.config.ts, tsconfig*.json   (scaffold Tauri)
app-icon.svg                          ícone-fonte (gera src-tauri/icons via `tauri icon`)
src/
  main.tsx                            monta <App/>, importa styles.css
  App.tsx                             alterna tela principal ↔ configurações, Esc, altura automática
  styles.css                          visual v3
  lib/types.ts                        espelho dos tipos Rust
  lib/format.ts (+ .test.ts)          "1d 7h", "4,2M", "há 3 min", "12 set"
  lib/grid.ts (+ .test.ts)            grade 7×N dos 30 dias
  lib/severity.ts (+ .test.ts)        mesma regra do Rust
  lib/hooks.ts                        usePacer, useNow, useAutoHeight
  icons/icons.tsx, icons/claude.svg   ícones de traço + logo Claude
  components/Header.tsx, ProviderTabs.tsx, ProviderCard.tsx, UsageBar.tsx,
             ActivityGrid.tsx, Footer.tsx, MainView.tsx
  components/settings/SettingsView.tsx, Row.tsx, Toggle.tsx, Segmented.tsx, ThresholdBar.tsx
src-tauri/
  Cargo.toml, tauri.conf.json, capabilities/default.json, build.rs
  src/main.rs                         chama pacer_lib::run()
  src/lib.rs                          setup, plugins, comandos
  src/snapshot.rs                     Snapshot, UsageWindow, Pace, DayActivity, Notice, Severity
  src/pacing.rs                       projeção
  src/config.rs                       Config + load/save
  src/state.rs                        Shared (estado compartilhado)
  src/scheduler.rs                    Backoff + laço de atualização
  src/commands.rs                     get_state, save_config, refresh_now, hide_window
  src/alerts.rs                       AlertState + notificação
  src/window.rs                       ancoragem, clamp, acrílico, set_window_height
  src/tray.rs                         ícone-anel gerado + menu
  src/providers/mod.rs                trait Provider, FetchResult
  src/providers/claude/mod.rs         ClaudeProvider
  src/providers/claude/creds.rs       leitura das credenciais
  src/providers/claude/api.rs         chamada e parsing da API
  src/providers/claude/logs.rs        LogScanner
  tests/fixtures/usage_full.json, usage_minimal.json
```

Comandos de teste usados em todo o plano:
- Rust: `cargo test --manifest-path src-tauri/Cargo.toml <filtro>`
- Frontend: `bun run test`

---

### Task 0: Pré-requisito — Visual Studio Build Tools (C++)

O Rust no Windows (`x86_64-pc-windows-msvc`) precisa do linker MSVC, ausente desde a formatação.

- [ ] **Step 1: Confirmar com o Sthevan e instalar** (pede UAC, ~5–7 GB)

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

- [ ] **Step 2: Verificar**

Run (PowerShell): `& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property displayName`
Expected: `Visual Studio Build Tools 2022`

Run: `cargo new --vcs none "$env:TEMP\pacer-linker-check"; cargo run --manifest-path "$env:TEMP\pacer-linker-check\Cargo.toml"`
Expected: `Hello, world!` (depois apagar a pasta).

---

### Task 1: Scaffold Tauri + React + Bun, remover Electron

**Files:**
- Delete: `main.js`, `preload.js`, `usage.js`, `auth.js`, `config.json`, `package.json`, `package-lock.json`, `renderer/`, `scripts/`, `test/`, `assets/`
- Create: scaffold inteiro (raiz + `src/` + `src-tauri/`)
- Modify: `.gitignore`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `package.json`, `index.html`, `src/main.tsx`, `src/App.tsx`

**Interfaces:**
- Produces: crate `pacer_lib` com `pub fn run()`; script `bun run test` (Vitest); janela `main` 300 px sem borda.

- [ ] **Step 1: Remover o código Electron**

```bash
cd /d/Projetos/Claude-Glass
git rm -r -q main.js preload.js usage.js auth.js config.json package.json package-lock.json renderer scripts test assets
```

- [ ] **Step 2: Gerar o scaffold numa subpasta e mover para a raiz**

```bash
cd /d/Projetos/Claude-Glass
bunx create-tauri-app@latest pacer -m bun -t react-ts -y
cp -r pacer/. . && rm -rf pacer
rm -f src/App.css src/assets/react.svg public/tauri.svg public/vite.svg README.md
```

Se o CLI perguntar algo mesmo com `-y`, responder: identifier `dev.sthevan.pacer`, frontend TypeScript, React, manager bun.

- [ ] **Step 3: Restaurar/estender o `.gitignore`**

O `cp` sobrescreveu o `.gitignore`. Garantir que ele contenha (acrescentar o que faltar):

```gitignore
node_modules/
dist/
src-tauri/target/
src-tauri/gen/schemas/
*.local
.superpowers/
*.pfx
```

- [ ] **Step 4: Dependências Rust** — substituir `[dependencies]`/`[dev-dependencies]` em `src-tauri/Cargo.toml` (manter `[package]`, `[lib]` com `name = "pacer_lib"` e `[build-dependencies]` do scaffold; ajustar `version = "2.0.0"`, `description = "Pacer - uso dos planos de IA"`):

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
tauri-plugin-notification = "2"
tauri-plugin-autostart = "2"
window-vibrancy = "0.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
tokio = { version = "1", features = ["sync", "time", "macros"] }
async-trait = "0.1"
dirs = "6"

[dev-dependencies]
tempfile = "3"
mockito = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

- [ ] **Step 5: `src-tauri/tauri.conf.json`** (substituir inteiro):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Pacer",
  "version": "2.0.0",
  "identifier": "dev.sthevan.pacer",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Pacer",
        "width": 300,
        "height": 560,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "shadow": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.ico"]
  }
}
```

- [ ] **Step 6: `src-tauri/capabilities/default.json`** (substituir inteiro):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Permissões da janela principal do Pacer",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "opener:default",
    "notification:default",
    "autostart:default"
  ]
}
```

- [ ] **Step 7: `src-tauri/src/lib.rs` mínimo** (substituir inteiro; as tasks seguintes o completam):

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            app.get_webview_window("main").expect("janela main").show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Pacer");
}
```

`src-tauri/src/main.rs` deve conter apenas:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    pacer_lib::run()
}
```

- [ ] **Step 8: Frontend mínimo + Vitest**

```bash
bun add -d vitest
```

Em `package.json`, garantir `"name": "pacer"`, `"version": "2.0.0"` e o script `"test": "vitest run"`.

`index.html` — trocar o `<title>` por `<title>Pacer</title>` e remover a linha `<link rel="icon" ...>`.

`src/main.tsx` (substituir inteiro):

```tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

`src/App.tsx` (temporário, substituído na Task 13):

```tsx
export default function App() {
  return <div className="wg">Pacer</div>;
}
```

`src/styles.css` (temporário): `body { margin: 0; background: transparent; color: #e6edf3; }`

- [ ] **Step 9: Verificar build**

Run: `bun install && bun run build`
Expected: `vite build` termina sem erros, gera `dist/`.

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: `Finished` sem erros.

Run: `bun run tauri dev` → aparece uma janela pequena escrito "Pacer". Fechar com Ctrl+C no terminal.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: troca Electron por scaffold Tauri 2 + React + Bun

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Tipos comuns e severidade (`snapshot.rs`)

**Files:**
- Create: `src-tauri/src/snapshot.rs`
- Modify: `src-tauri/src/lib.rs` (adicionar `pub mod snapshot;` no topo)

**Interfaces:**
- Produces: `Snapshot`, `UsageWindow`, `Pace`, `DayActivity`, `Notice`, `Severity`, `fn severity(used_pct: f64, pace: Option<&Pace>) -> Severity`, `Snapshot::max_severity(&self) -> Severity`. Serialização em camelCase; `Notice`/`Severity` como strings camelCase (`"noCredentials"`, `"critical"`).

- [ ] **Step 1: Escrever os testes** — criar `src-tauri/src/snapshot.rs` só com o módulo de testes no fim (o código vem no Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn pace(p: f64) -> Pace {
        Pace { projected_pct: p, limit_at: None }
    }

    #[test]
    fn severity_thresholds() {
        assert_eq!(severity(74.9, None), Severity::Ok);
        assert_eq!(severity(75.0, None), Severity::Warn);
        assert_eq!(severity(90.0, None), Severity::Warn);
        assert_eq!(severity(90.1, None), Severity::Critical);
    }

    #[test]
    fn projection_over_100_is_critical_even_with_low_usage() {
        assert_eq!(severity(30.0, Some(&pace(101.0))), Severity::Critical);
        assert_eq!(severity(30.0, Some(&pace(100.0))), Severity::Ok);
    }

    #[test]
    fn serializes_camel_case() {
        let s = Snapshot {
            provider: "claude".into(),
            name: "Claude".into(),
            plan: Some("Pro".into()),
            windows: vec![UsageWindow {
                id: "five_hour".into(),
                label: "Sessão (5h)".into(),
                used_pct: 38.0,
                resets_at: Some(Utc.with_ymd_and_hms(2026, 10, 3, 17, 0, 0).unwrap()),
                pace: Some(pace(61.0)),
            }],
            activity: vec![DayActivity { date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(), tokens: 10, messages: 2 }],
            today_tokens: 10,
            fetched_at: None,
            next_refresh_at: None,
            stale: false,
            notice: Some(Notice::NoCredentials),
        };
        let v = serde_json::to_value(&s).unwrap();
        assert_eq!(v["todayTokens"], 10);
        assert_eq!(v["windows"][0]["usedPct"], 38.0);
        assert_eq!(v["windows"][0]["resetsAt"], "2026-10-03T17:00:00Z");
        assert_eq!(v["windows"][0]["pace"]["projectedPct"], 61.0);
        assert_eq!(v["activity"][0]["date"], "2026-10-03");
        assert_eq!(v["notice"], "noCredentials");
        assert_eq!(serde_json::to_value(Severity::Critical).unwrap(), "critical");
    }

    #[test]
    fn max_severity_picks_worst_window() {
        let w = |p: f64| UsageWindow { id: "x".into(), label: "x".into(), used_pct: p, resets_at: None, pace: None };
        let mut s = Snapshot {
            provider: "claude".into(), name: "Claude".into(), plan: None,
            windows: vec![w(10.0), w(80.0)], activity: vec![], today_tokens: 0,
            fetched_at: None, next_refresh_at: None, stale: false, notice: None,
        };
        assert_eq!(s.max_severity(), Severity::Warn);
        s.windows.clear();
        assert_eq!(s.max_severity(), Severity::Ok);
    }
}
```

E adicionar `pub mod snapshot;` em `src-tauri/src/lib.rs`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml snapshot`
Expected: FAIL — erros de compilação `cannot find type Snapshot`.

- [ ] **Step 3: Implementar** — acima do módulo de testes em `snapshot.rs`:

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pace {
    pub projected_pct: f64,
    pub limit_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub id: String,
    pub label: String,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub pace: Option<Pace>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayActivity {
    pub date: NaiveDate,
    pub tokens: u64,
    pub messages: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Notice {
    NoCredentials,
    TokenExpired,
    RateLimited,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Ok,
    Warn,
    Critical,
}

/// Azul < 75%, laranja 75–90%, vermelho > 90% ou projeção > 100%.
pub fn severity(used_pct: f64, pace: Option<&Pace>) -> Severity {
    let projected_over = pace.is_some_and(|p| p.projected_pct > 100.0);
    if used_pct > 90.0 || projected_over {
        Severity::Critical
    } else if used_pct >= 75.0 {
        Severity::Warn
    } else {
        Severity::Ok
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub provider: String,
    pub name: String,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    /// 30 itens, do mais antigo até hoje.
    pub activity: Vec<DayActivity>,
    pub today_tokens: u64,
    /// Última busca remota bem-sucedida (None = nunca / só logs).
    pub fetched_at: Option<DateTime<Utc>>,
    /// Preenchido pelo scheduler.
    pub next_refresh_at: Option<DateTime<Utc>>,
    pub stale: bool,
    pub notice: Option<Notice>,
}

impl Snapshot {
    pub fn max_severity(&self) -> Severity {
        self.windows
            .iter()
            .map(|w| severity(w.used_pct, w.pace.as_ref()))
            .max()
            .unwrap_or(Severity::Ok)
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml snapshot`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/snapshot.rs src-tauri/src/lib.rs
git commit -m "feat: tipos comuns do Snapshot e regra de severidade

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Projeção do ritmo (`pacing.rs`)

**Files:**
- Create: `src-tauri/src/pacing.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod pacing;`)

**Interfaces:**
- Consumes: `snapshot::Pace`
- Produces: `pub fn project(used_pct: f64, length: chrono::Duration, resets_at: DateTime<Utc>, now: DateTime<Utc>) -> Option<Pace>`

- [ ] **Step 1: Escrever os testes** (`src-tauri/src/pacing.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn at(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 3, h, m, 0).unwrap()
    }

    // Sessão de 5h que começou 10:00 e redefine 15:00.
    const FIVE_H: i64 = 5;

    #[test]
    fn no_projection_in_first_5_percent_of_window() {
        // 5% de 5h = 15 min; às 10:10 passou só ~3%
        assert_eq!(project(10.0, Duration::hours(FIVE_H), at(15, 0), at(10, 10)), None);
    }

    #[test]
    fn projects_linearly_to_reset() {
        // metade da janela, 40% usado → 80% projetado
        let p = project(40.0, Duration::hours(FIVE_H), at(15, 0), at(12, 30)).unwrap();
        assert!((p.projected_pct - 80.0).abs() < 1e-9);
        assert_eq!(p.limit_at, None);
    }

    #[test]
    fn computes_when_limit_is_hit() {
        // metade da janela (9000 s), 60% usado → 120%; faltam 40% a 60%/9000s → 6000 s
        let now = at(12, 30);
        let p = project(60.0, Duration::hours(FIVE_H), at(15, 0), now).unwrap();
        assert!((p.projected_pct - 120.0).abs() < 1e-9);
        assert_eq!(p.limit_at, Some(now + Duration::seconds(6000)));
    }

    #[test]
    fn zero_usage_projects_zero() {
        let p = project(0.0, Duration::hours(FIVE_H), at(15, 0), at(12, 30)).unwrap();
        assert_eq!(p.projected_pct, 0.0);
        assert_eq!(p.limit_at, None);
    }

    #[test]
    fn already_at_limit_means_limit_now() {
        let now = at(12, 30);
        let p = project(100.0, Duration::hours(FIVE_H), at(15, 0), now).unwrap();
        assert_eq!(p.limit_at, Some(now));
    }

    #[test]
    fn reset_in_the_past_or_clock_skew_gives_none() {
        assert_eq!(project(50.0, Duration::hours(FIVE_H), at(15, 0), at(15, 1)), None);
        // resets_at mais longe que a própria janela (relógio adiantado)
        assert_eq!(project(50.0, Duration::hours(FIVE_H), at(23, 0), at(12, 0)), None);
    }
}
```

Adicionar `pub mod pacing;` em `lib.rs`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml pacing`
Expected: FAIL — `cannot find function project`.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use crate::snapshot::Pace;
use chrono::{DateTime, Duration, Utc};

/// Projeta o uso até a redefinição mantendo o ritmo médio da janela.
/// Retorna None nos primeiros 5% da janela (pouco dado) ou se o relógio não fecha.
pub fn project(used_pct: f64, length: Duration, resets_at: DateTime<Utc>, now: DateTime<Utc>) -> Option<Pace> {
    let total = length.num_seconds() as f64;
    let remaining = (resets_at - now).num_seconds() as f64;
    if total <= 0.0 || remaining <= 0.0 {
        return None;
    }
    let elapsed = total - remaining;
    if elapsed < total * 0.05 {
        return None;
    }
    if used_pct <= 0.0 {
        return Some(Pace { projected_pct: 0.0, limit_at: None });
    }
    let projected_pct = used_pct * total / elapsed;
    let limit_at = if used_pct >= 100.0 {
        Some(now)
    } else if projected_pct > 100.0 {
        let rate = used_pct / elapsed; // % por segundo
        let secs = ((100.0 - used_pct) / rate).round() as i64;
        Some(now + Duration::seconds(secs))
    } else {
        None
    };
    Some(Pace { projected_pct, limit_at })
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml pacing`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/pacing.rs src-tauri/src/lib.rs
git commit -m "feat: projecao do ritmo de consumo ate a redefinicao

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Configuração (`config.rs`)

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod config;`)

**Interfaces:**
- Produces: `Config { start_with_windows, lock_position, refresh_minutes: u32, alerts: AlertsConfig, providers: ProvidersConfig }`, `AlertsConfig { enabled, thresholds: Vec<u8>, pace }`, `ProvidersConfig { claude: ProviderToggle }`, `ProviderToggle { enabled }`, `Config::normalized(self) -> Config`, `Config::provider_enabled(&self, id: &str) -> bool`, `pub fn load(path: &Path) -> Config`, `pub fn save(path: &Path, cfg: &Config) -> io::Result<()>`. JSON em camelCase.

- [ ] **Step 1: Escrever os testes** (`src-tauri/src/config.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let c = load(&dir.path().join("config.json"));
        assert_eq!(c, Config::default());
        assert_eq!(c.refresh_minutes, 5);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
        assert!(c.start_with_windows && !c.lock_position && c.providers.claude.enabled);
    }

    #[test]
    fn partial_file_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"refreshMinutes":10,"alerts":{"pace":false}}"#).unwrap();
        let c = load(&p);
        assert_eq!(c.refresh_minutes, 10);
        assert!(!c.alerts.pace);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
    }

    #[test]
    fn invalid_values_are_normalized() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"refreshMinutes":7,"alerts":{"thresholds":[95,0,80,80,120]}}"#).unwrap();
        let c = load(&p);
        assert_eq!(c.refresh_minutes, 5);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
    }

    #[test]
    fn corrupt_file_becomes_bak_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, "{ quebrado").unwrap();
        assert_eq!(load(&p), Config::default());
        assert!(!p.exists());
        assert_eq!(fs::read_to_string(dir.path().join("config.json.bak")).unwrap(), "{ quebrado");
    }

    #[test]
    fn save_roundtrip_without_leftover_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sub").join("config.json");
        let mut c = Config::default();
        c.lock_position = true;
        c.refresh_minutes = 1;
        save(&p, &c).unwrap();
        assert_eq!(load(&p), c);
        assert!(!dir.path().join("sub").join("config.json.tmp").exists());
        let raw = fs::read_to_string(&p).unwrap();
        assert!(raw.contains("\"lockPosition\": true"));
    }

    #[test]
    fn provider_enabled_lookup() {
        let mut c = Config::default();
        assert!(c.provider_enabled("claude"));
        c.providers.claude.enabled = false;
        assert!(!c.provider_enabled("claude"));
        assert!(!c.provider_enabled("desconhecido"));
    }
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config`
Expected: FAIL — `cannot find type Config`.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertsConfig {
    pub enabled: bool,
    pub thresholds: Vec<u8>,
    pub pace: bool,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self { enabled: true, thresholds: vec![80, 95], pace: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderToggle {
    pub enabled: bool,
}

impl Default for ProviderToggle {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProvidersConfig {
    pub claude: ProviderToggle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub start_with_windows: bool,
    pub lock_position: bool,
    pub refresh_minutes: u32,
    pub alerts: AlertsConfig,
    pub providers: ProvidersConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            lock_position: false,
            refresh_minutes: 5,
            alerts: AlertsConfig::default(),
            providers: ProvidersConfig::default(),
        }
    }
}

impl Config {
    pub fn normalized(mut self) -> Self {
        if ![1, 5, 10].contains(&self.refresh_minutes) {
            self.refresh_minutes = 5;
        }
        self.alerts.thresholds.retain(|t| (1..=100).contains(t));
        self.alerts.thresholds.sort_unstable();
        self.alerts.thresholds.dedup();
        self
    }

    pub fn provider_enabled(&self, id: &str) -> bool {
        match id {
            "claude" => self.providers.claude.enabled,
            _ => false,
        }
    }
}

/// Lê a config; arquivo ausente → padrões; inválido → padrões e o original vira `.bak`.
pub fn load(path: &Path) -> Config {
    let Ok(raw) = fs::read_to_string(path) else {
        return Config::default();
    };
    match serde_json::from_str::<Config>(&raw) {
        Ok(c) => c.normalized(),
        Err(_) => {
            let _ = fs::rename(path, path.with_extension("json.bak"));
            Config::default()
        }
    }
}

/// Gravação atômica: escreve num `.tmp` e renomeia por cima.
pub fn save(path: &Path, cfg: &Config) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(cfg)?)?;
    fs::rename(&tmp, path)
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml config`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: configuracao com gravacao atomica e recuperacao de arquivo corrompido

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Credenciais do Claude Code (`creds.rs`)

**Files:**
- Create: `src-tauri/src/providers/mod.rs` (por ora só `pub mod claude;`), `src-tauri/src/providers/claude/mod.rs` (por ora só `pub mod creds;`), `src-tauri/src/providers/claude/creds.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod providers;`)

**Interfaces:**
- Produces: `pub struct Creds { pub access_token: String, pub expires_at: DateTime<Utc>, pub plan: Option<String> }` (Debug sem o token), `pub enum CredsError { Missing, Unreadable(String) }`, `pub fn claude_dir() -> PathBuf`, `pub fn read(claude_dir: &Path) -> Result<Creds, CredsError>`, `Creds::is_expired(&self, now) -> bool`, `pub fn plan_label(sub: Option<&str>, tier: Option<&str>) -> Option<String>`.

- [ ] **Step 1: Escrever os testes** (`src-tauri/src/providers/claude/creds.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use std::fs;

    fn write(dir: &Path, body: &str) {
        fs::write(dir.join(".credentials.json"), body).unwrap();
    }

    #[test]
    fn missing_file_is_missing() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(read(d.path()), Err(CredsError::Missing));
    }

    #[test]
    fn file_without_claude_oauth_is_missing() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"mcpOAuth":{}}"#);
        assert_eq!(read(d.path()), Err(CredsError::Missing));
    }

    #[test]
    fn half_written_file_is_unreadable_not_missing() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"claudeAiOauth":{"accessTo"#);
        assert!(matches!(read(d.path()), Err(CredsError::Unreadable(_))));
    }

    #[test]
    fn reads_token_expiry_and_plan() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat-x","refreshToken":"r","expiresAt":1791043200000,"scopes":[],"subscriptionType":"pro","rateLimitTier":"default_claude_ai"}}"#);
        let c = read(d.path()).unwrap();
        assert_eq!(c.access_token, "sk-ant-oat-x");
        assert_eq!(c.expires_at, Utc.timestamp_millis_opt(1791043200000).unwrap());
        assert_eq!(c.plan.as_deref(), Some("Pro"));
    }

    #[test]
    fn debug_does_not_leak_token() {
        let c = Creds { access_token: "segredo".into(), expires_at: Utc::now(), plan: None };
        assert!(!format!("{c:?}").contains("segredo"));
    }

    #[test]
    fn expiry_has_one_minute_margin() {
        let exp = Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap();
        let c = Creds { access_token: "t".into(), expires_at: exp, plan: None };
        assert!(!c.is_expired(exp - Duration::seconds(61)));
        assert!(c.is_expired(exp - Duration::seconds(59)));
    }

    #[test]
    fn plan_labels() {
        assert_eq!(plan_label(Some("max"), Some("default_claude_max_20x")).as_deref(), Some("Max 20x"));
        assert_eq!(plan_label(Some("max"), Some("default_claude_max_5x")).as_deref(), Some("Max 5x"));
        assert_eq!(plan_label(Some("max"), None).as_deref(), Some("Max"));
        assert_eq!(plan_label(Some("pro"), Some("default_claude_ai")).as_deref(), Some("Pro"));
        assert_eq!(plan_label(Some("team"), None).as_deref(), Some("Team"));
        assert_eq!(plan_label(None, None), None);
    }
}
```

Criar `src-tauri/src/providers/mod.rs` com `pub mod claude;`, `src-tauri/src/providers/claude/mod.rs` com `pub mod creds;` e adicionar `pub mod providers;` em `lib.rs`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml creds`
Expected: FAIL — `cannot find function read`.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawOauth {
    access_token: String,
    expires_at: i64,
    subscription_type: Option<String>,
    rate_limit_tier: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFile {
    claude_ai_oauth: Option<RawOauth>,
}

#[derive(Clone, PartialEq)]
pub struct Creds {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub plan: Option<String>,
}

impl fmt::Debug for Creds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Creds")
            .field("access_token", &"<oculto>")
            .field("expires_at", &self.expires_at)
            .field("plan", &self.plan)
            .finish()
    }
}

impl Creds {
    /// Considera vencido 1 minuto antes, pra não mandar token no limite.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now + Duration::seconds(60) >= self.expires_at
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CredsError {
    /// Sem arquivo ou sem login do Claude.ai nele.
    Missing,
    /// Arquivo existe mas não deu pra ler/parsear (ex.: Claude Code reescrevendo agora).
    Unreadable(String),
}

pub fn claude_dir() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".claude"))
}

/// Somente leitura: o Pacer nunca renova nem escreve neste arquivo.
pub fn read(claude_dir: &Path) -> Result<Creds, CredsError> {
    let path = claude_dir.join(".credentials.json");
    let raw = match fs::read_to_string(&path) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(CredsError::Missing),
        Err(e) => return Err(CredsError::Unreadable(e.to_string())),
    };
    let file: RawFile = serde_json::from_str(&raw).map_err(|e| CredsError::Unreadable(e.to_string()))?;
    let o = file.claude_ai_oauth.ok_or(CredsError::Missing)?;
    let expires_at = DateTime::from_timestamp_millis(o.expires_at)
        .ok_or_else(|| CredsError::Unreadable("expiresAt inválido".into()))?;
    Ok(Creds {
        access_token: o.access_token,
        expires_at,
        plan: plan_label(o.subscription_type.as_deref(), o.rate_limit_tier.as_deref()),
    })
}

pub fn plan_label(sub: Option<&str>, tier: Option<&str>) -> Option<String> {
    let tier = tier.unwrap_or("").to_ascii_lowercase();
    if tier.contains("max_20x") {
        return Some("Max 20x".into());
    }
    if tier.contains("max_5x") {
        return Some("Max 5x".into());
    }
    let label = match sub.map(str::to_ascii_lowercase).as_deref() {
        Some("max") => "Max",
        Some("pro") => "Pro",
        Some("team") => "Team",
        Some("enterprise") => "Enterprise",
        Some("free") => "Free",
        _ => return None,
    };
    Some(label.into())
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml creds`
Expected: 7 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/providers src-tauri/src/lib.rs
git commit -m "feat: leitura somente-leitura das credenciais do Claude Code

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: API de uso da Anthropic (`api.rs`)

**Files:**
- Create: `src-tauri/src/providers/claude/api.rs`, `src-tauri/tests/fixtures/usage_full.json`, `src-tauri/tests/fixtures/usage_minimal.json`
- Modify: `src-tauri/src/providers/claude/mod.rs` (`pub mod api;`)

**Interfaces:**
- Produces: `pub const USAGE_URL: &str`, `pub struct ApiWindow { pub id: &'static str, pub label: &'static str, pub used_pct: f64, pub resets_at: Option<DateTime<Utc>>, pub length: chrono::Duration }`, `pub fn parse_usage(body: &str) -> Result<Vec<ApiWindow>, serde_json::Error>`, `pub enum ApiError { Unauthorized, RateLimited, Network(String), BadResponse(String) }`, `pub fn user_agent(cli_version: &str) -> String`, `pub async fn fetch_usage(client: &reqwest::Client, url: &str, token: &str, ua: &str) -> Result<Vec<ApiWindow>, ApiError>`.

Observação: o endpoint responde 429 sem um `User-Agent` no formato do Claude Code (`claude-cli/<versão> (external, cli)`), como documentado no ai-usagebar. Usamos a versão instalada no PC (2.1.288).

- [ ] **Step 1: Fixtures**

`src-tauri/tests/fixtures/usage_full.json`:

```json
{
  "five_hour": { "utilization": 62.0, "resets_at": "2026-05-23T13:30:00Z" },
  "seven_day": { "utilization": 27.0, "resets_at": "2026-05-27T13:00:00Z" },
  "seven_day_sonnet": { "utilization": 4.0, "resets_at": "2026-05-23T14:24:00+00:00" },
  "seven_day_opus": null,
  "tangelo": null,
  "extra_usage": { "is_enabled": true, "monthly_limit": 5000, "used_credits": 250.0, "currency": "USD", "decimal_places": 2 }
}
```

`src-tauri/tests/fixtures/usage_minimal.json`:

```json
{
  "five_hour": { "utilization": 15.0, "resets_at": "2026-05-23T13:30:00Z" },
  "seven_day": { "utilization": 8.0, "resets_at": null }
}
```

- [ ] **Step 2: Escrever os testes** (`src-tauri/src/providers/claude/api.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    const FULL: &str = include_str!("../../../tests/fixtures/usage_full.json");
    const MINIMAL: &str = include_str!("../../../tests/fixtures/usage_minimal.json");

    #[test]
    fn parses_full_response_ignoring_nulls_and_unknown_fields() {
        let ws = parse_usage(FULL).unwrap();
        let ids: Vec<_> = ws.iter().map(|w| w.id).collect();
        assert_eq!(ids, ["five_hour", "seven_day", "seven_day_sonnet"]);
        assert_eq!(ws[0].label, "Sessão (5h)");
        assert_eq!(ws[0].used_pct, 62.0);
        assert_eq!(ws[0].length, Duration::hours(5));
        assert_eq!(ws[0].resets_at, Some(Utc.with_ymd_and_hms(2026, 5, 23, 13, 30, 0).unwrap()));
        assert_eq!(ws[2].label, "Sonnet · semanal");
        assert_eq!(ws[2].resets_at, Some(Utc.with_ymd_and_hms(2026, 5, 23, 14, 24, 0).unwrap()));
    }

    #[test]
    fn parses_minimal_response_with_null_reset() {
        let ws = parse_usage(MINIMAL).unwrap();
        assert_eq!(ws.len(), 2);
        assert_eq!(ws[1].label, "Semanal");
        assert_eq!(ws[1].resets_at, None);
    }

    #[test]
    fn window_without_utilization_is_skipped_and_values_clamped() {
        let ws = parse_usage(r#"{"five_hour":{"resets_at":null},"seven_day":{"utilization":130.0}}"#).unwrap();
        assert_eq!(ws.len(), 1);
        assert_eq!(ws[0].used_pct, 100.0);
    }

    #[test]
    fn user_agent_format() {
        assert_eq!(user_agent("2.1.288"), "claude-cli/2.1.288 (external, cli)");
    }

    async fn serve(status: usize, body: &str) -> (mockito::ServerGuard, mockito::Mock) {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/usage")
            .match_header("authorization", "Bearer tok")
            .match_header("anthropic-beta", "oauth-2025-04-20")
            .match_header("user-agent", "ua-teste")
            .with_status(status)
            .with_body(body)
            .create_async()
            .await;
        (server, mock)
    }

    #[tokio::test]
    async fn fetch_ok() {
        let (server, mock) = serve(200, MINIMAL).await;
        let ws = fetch_usage(&reqwest::Client::new(), &format!("{}/usage", server.url()), "tok", "ua-teste").await.unwrap();
        mock.assert_async().await;
        assert_eq!(ws.len(), 2);
    }

    #[tokio::test]
    async fn fetch_maps_status_codes() {
        let client = reqwest::Client::new();
        for (status, expected) in [
            (401, ApiError::Unauthorized),
            (403, ApiError::Unauthorized),
            (429, ApiError::RateLimited),
            (500, ApiError::BadResponse("HTTP 500".into())),
        ] {
            let (server, _m) = serve(status, "{}").await;
            let err = fetch_usage(&client, &format!("{}/usage", server.url()), "tok", "ua-teste").await.unwrap_err();
            assert_eq!(err, expected, "status {status}");
        }
    }

    #[tokio::test]
    async fn fetch_network_error() {
        let err = fetch_usage(&reqwest::Client::new(), "http://127.0.0.1:9/usage", "tok", "ua").await.unwrap_err();
        assert!(matches!(err, ApiError::Network(_)));
    }
}
```

Adicionar `pub mod api;` em `providers/claude/mod.rs`.

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml api::`
Expected: FAIL — `cannot find function parse_usage`.

- [ ] **Step 4: Implementar** (acima dos testes):

```rust
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

pub const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const BETA: &str = "oauth-2025-04-20";

#[derive(Deserialize)]
struct RawWindow {
    utilization: Option<f64>,
    resets_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct RawUsage {
    five_hour: Option<RawWindow>,
    seven_day: Option<RawWindow>,
    seven_day_opus: Option<RawWindow>,
    seven_day_sonnet: Option<RawWindow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiWindow {
    pub id: &'static str,
    pub label: &'static str,
    pub used_pct: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub length: Duration,
}

/// Janelas `null`, sem `utilization` ou desconhecidas são ignoradas.
pub fn parse_usage(body: &str) -> Result<Vec<ApiWindow>, serde_json::Error> {
    let raw: RawUsage = serde_json::from_str(body)?;
    let specs = [
        ("five_hour", "Sessão (5h)", Duration::hours(5), raw.five_hour),
        ("seven_day", "Semanal", Duration::days(7), raw.seven_day),
        ("seven_day_opus", "Opus · semanal", Duration::days(7), raw.seven_day_opus),
        ("seven_day_sonnet", "Sonnet · semanal", Duration::days(7), raw.seven_day_sonnet),
    ];
    Ok(specs
        .into_iter()
        .filter_map(|(id, label, length, w)| {
            let w = w?;
            let used = w.utilization?;
            Some(ApiWindow { id, label, used_pct: used.clamp(0.0, 100.0), resets_at: w.resets_at, length })
        })
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    Unauthorized,
    RateLimited,
    Network(String),
    BadResponse(String),
}

pub fn user_agent(cli_version: &str) -> String {
    format!("claude-cli/{cli_version} (external, cli)")
}

pub async fn fetch_usage(client: &reqwest::Client, url: &str, token: &str, ua: &str) -> Result<Vec<ApiWindow>, ApiError> {
    let resp = client
        .get(url)
        .bearer_auth(token)
        .header("anthropic-beta", BETA)
        .header("User-Agent", ua)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;
    match resp.status().as_u16() {
        401 | 403 => return Err(ApiError::Unauthorized),
        429 => return Err(ApiError::RateLimited),
        s if !(200..300).contains(&s) => return Err(ApiError::BadResponse(format!("HTTP {s}"))),
        _ => {}
    }
    let body = resp.text().await.map_err(|e| ApiError::Network(e.to_string()))?;
    parse_usage(&body).map_err(|e| ApiError::BadResponse(e.to_string()))
}
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml api::`
Expected: 7 passed.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/providers/claude src-tauri/tests
git commit -m "feat: cliente da API de uso da Anthropic com mapeamento de erros

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Leitura dos logs do Claude Code (`logs.rs`)

**Files:**
- Create: `src-tauri/src/providers/claude/logs.rs`
- Modify: `src-tauri/src/providers/claude/mod.rs` (`pub mod logs;`)

**Interfaces:**
- Consumes: `snapshot::DayActivity`
- Produces: `pub struct Activity { pub days: Vec<DayActivity>, pub today_tokens: u64 }`, `pub struct LogScanner` com `pub fn new() -> Self`, `pub fn scan<Tz: TimeZone>(&mut self, projects_dir: &Path, now: DateTime<Tz>) -> Activity`, `pub fn last_parsed(&self) -> usize` (arquivos relidos na última varredura).

- [ ] **Step 1: Escrever os testes** (`src-tauri/src/providers/claude/logs.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use std::fs;
    use std::io::Write;

    fn brt() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    // "agora" = 03/10/2026 15:00 em UTC−3
    fn now() -> DateTime<FixedOffset> {
        brt().with_ymd_and_hms(2026, 10, 3, 15, 0, 0).unwrap()
    }

    fn line(ts: &str, id: &str, req: &str, tokens: u64) -> String {
        format!(
            r#"{{"type":"assistant","timestamp":"{ts}","requestId":"{req}","message":{{"id":"{id}","model":"claude-opus-4-5","usage":{{"input_tokens":{tokens},"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
        )
    }

    fn day(a: &Activity, y: i32, m: u32, d: u32) -> &DayActivity {
        let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
        a.days.iter().find(|x| x.date == date).expect("dia fora da janela")
    }

    #[test]
    fn missing_dir_gives_30_empty_days_ending_today() {
        let d = tempfile::tempdir().unwrap();
        let a = LogScanner::new().scan(&d.path().join("nao-existe"), now());
        assert_eq!(a.days.len(), 30);
        assert_eq!(a.days[0].date, NaiveDate::from_ymd_opt(2026, 9, 4).unwrap());
        assert_eq!(a.days[29].date, NaiveDate::from_ymd_opt(2026, 10, 3).unwrap());
        assert_eq!(a.today_tokens, 0);
    }

    #[test]
    fn counts_tokens_and_messages_skipping_noise() {
        let d = tempfile::tempdir().unwrap();
        let proj = d.path().join("proj-a");
        fs::create_dir_all(&proj).unwrap();
        let body = [
            line("2026-10-03T12:00:00Z", "m1", "r1", 100),
            line("2026-10-03T13:00:00Z", "m2", "r2", 50),
            r#"{"type":"user","timestamp":"2026-10-03T13:00:00Z","message":{"content":"oi"}}"#.to_string(),
            r#"{"type":"assistant","timestamp":"2026-10-03T13:00:00Z","message":{"id":"s","model":"<synthetic>","usage":{"input_tokens":999}}}"#.to_string(),
            "{ linha quebrada".to_string(),
            String::new(),
        ]
        .join("\n");
        fs::write(proj.join("s.jsonl"), body).unwrap();
        fs::write(proj.join("ignorado.txt"), line("2026-10-03T12:00:00Z", "x", "y", 7)).unwrap();
        let a = LogScanner::new().scan(d.path(), now());
        let today = day(&a, 2026, 10, 3);
        assert_eq!(today.tokens, 150);
        assert_eq!(today.messages, 2);
        assert_eq!(a.today_tokens, 150);
    }

    #[test]
    fn deduplicates_same_message_across_files() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.jsonl"), line("2026-10-02T12:00:00Z", "m1", "r1", 100)).unwrap();
        fs::write(d.path().join("b.jsonl"), line("2026-10-02T12:00:00Z", "m1", "r1", 100)).unwrap();
        let a = LogScanner::new().scan(d.path(), now());
        assert_eq!(day(&a, 2026, 10, 2).tokens, 100);
        assert_eq!(day(&a, 2026, 10, 2).messages, 1);
    }

    #[test]
    fn groups_by_local_day_not_utc_day() {
        let d = tempfile::tempdir().unwrap();
        // 02:30 UTC de 03/10 = 23:30 de 02/10 em UTC−3
        fs::write(d.path().join("a.jsonl"), line("2026-10-03T02:30:00Z", "m1", "r1", 40)).unwrap();
        let a = LogScanner::new().scan(d.path(), now());
        assert_eq!(day(&a, 2026, 10, 2).tokens, 40);
        assert_eq!(a.today_tokens, 0);
    }

    #[test]
    fn ignores_entries_outside_30_days() {
        let d = tempfile::tempdir().unwrap();
        let body = [line("2026-09-03T15:00:00Z", "old", "r", 500), line("2026-09-04T15:00:00Z", "first", "r", 5)].join("\n");
        fs::write(d.path().join("a.jsonl"), body).unwrap();
        let a = LogScanner::new().scan(d.path(), now());
        assert_eq!(a.days.iter().map(|x| x.tokens).sum::<u64>(), 5);
        assert_eq!(a.days[0].tokens, 5);
    }

    #[test]
    fn rescans_only_changed_files_and_forgets_deleted() {
        let d = tempfile::tempdir().unwrap();
        let a_path = d.path().join("a.jsonl");
        let b_path = d.path().join("b.jsonl");
        fs::write(&a_path, line("2026-10-03T12:00:00Z", "m1", "r1", 10)).unwrap();
        fs::write(&b_path, line("2026-10-03T12:00:00Z", "m2", "r2", 20)).unwrap();
        let mut s = LogScanner::new();
        assert_eq!(s.scan(d.path(), now()).today_tokens, 30);
        assert_eq!(s.last_parsed(), 2);

        assert_eq!(s.scan(d.path(), now()).today_tokens, 30);
        assert_eq!(s.last_parsed(), 0);

        let mut f = fs::OpenOptions::new().append(true).open(&a_path).unwrap();
        writeln!(f).unwrap();
        writeln!(f, "{}", line("2026-10-03T13:00:00Z", "m3", "r3", 5)).unwrap();
        drop(f);
        assert_eq!(s.scan(d.path(), now()).today_tokens, 35);
        assert_eq!(s.last_parsed(), 1);

        fs::remove_file(&b_path).unwrap();
        assert_eq!(s.scan(d.path(), now()).today_tokens, 15);
    }
}
```

Adicionar `pub mod logs;` em `providers/claude/mod.rs`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml logs`
Expected: FAIL — `cannot find struct LogScanner`.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use crate::snapshot::DayActivity;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const DAYS: i64 = 30;

#[derive(Debug, Clone)]
struct Entry {
    ts: DateTime<Utc>,
    tokens: u64,
    key: Option<String>,
}

struct CachedFile {
    modified: SystemTime,
    len: u64,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub days: Vec<DayActivity>,
    pub today_tokens: u64,
}

#[derive(Default)]
pub struct LogScanner {
    cache: HashMap<PathBuf, CachedFile>,
    last_parsed: usize,
}

fn parse_line(line: &str) -> Option<Entry> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("type")?.as_str()? != "assistant" {
        return None;
    }
    let msg = v.get("message")?;
    let usage = msg.get("usage")?;
    if msg.get("model").and_then(Value::as_str) == Some("<synthetic>") {
        return None;
    }
    let ts = v.get("timestamp")?.as_str()?.parse::<DateTime<Utc>>().ok()?;
    let n = |k: &str| usage.get(k).and_then(Value::as_u64).unwrap_or(0);
    let tokens = n("input_tokens") + n("output_tokens") + n("cache_read_input_tokens") + n("cache_creation_input_tokens");
    let key = msg.get("id").and_then(Value::as_str).map(|id| {
        let req = v.get("requestId").and_then(Value::as_str).unwrap_or("");
        format!("{id}:{req}")
    });
    Some(Entry { ts, tokens, key })
}

fn walk(dir: &Path, cutoff: SystemTime, out: &mut Vec<(PathBuf, fs::Metadata)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let Ok(md) = entry.metadata() else { continue };
        if md.is_dir() {
            walk(&path, cutoff, out);
        } else if path.extension().is_some_and(|x| x == "jsonl") && md.modified().is_ok_and(|m| m >= cutoff) {
            out.push((path, md));
        }
    }
}

impl LogScanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_parsed(&self) -> usize {
        self.last_parsed
    }

    /// Agrega tokens/mensagens por dia no fuso de `now`, dos últimos 30 dias.
    /// Só relê arquivos cujo mtime/tamanho mudou desde a última varredura.
    pub fn scan<Tz: TimeZone>(&mut self, projects_dir: &Path, now: DateTime<Tz>) -> Activity {
        self.last_parsed = 0;
        let tz = now.timezone();
        let today: NaiveDate = now.date_naive();
        let first = today - Duration::days(DAYS - 1);
        let cutoff: SystemTime = (now.with_timezone(&Utc) - Duration::days(DAYS + 1)).into();

        let mut files = Vec::new();
        walk(projects_dir, cutoff, &mut files);
        let live: HashSet<&PathBuf> = files.iter().map(|(p, _)| p).collect();
        self.cache.retain(|p, _| live.contains(p));

        let mut days: Vec<DayActivity> = (0..DAYS)
            .map(|i| DayActivity { date: first + Duration::days(i), tokens: 0, messages: 0 })
            .collect();
        let mut seen: HashSet<String> = HashSet::new();

        for (path, md) in &files {
            let modified = md.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let len = md.len();
            let changed = self.cache.get(path).is_none_or(|c| c.modified != modified || c.len != len);
            if changed {
                let entries = fs::read_to_string(path)
                    .map(|raw| raw.lines().filter_map(parse_line).collect())
                    .unwrap_or_default();
                self.cache.insert(path.clone(), CachedFile { modified, len, entries });
                self.last_parsed += 1;
            }
            for e in &self.cache[path].entries {
                if let Some(k) = &e.key {
                    if !seen.insert(k.clone()) {
                        continue;
                    }
                }
                let date = e.ts.with_timezone(&tz).date_naive();
                if date < first || date > today {
                    continue;
                }
                let d = &mut days[(date - first).num_days() as usize];
                d.tokens += e.tokens;
                d.messages += 1;
            }
        }

        let today_tokens = days.last().map_or(0, |d| d.tokens);
        Activity { days, today_tokens }
    }
}
```

(`Option::is_none_or` é estável desde Rust 1.82; o PC tem 1.99.)

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml logs`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/providers/claude
git commit -m "feat: leitura incremental dos logs do Claude Code por dia local

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Trait `Provider` e `ClaudeProvider`

**Files:**
- Modify: `src-tauri/src/providers/mod.rs`, `src-tauri/src/providers/claude/mod.rs`

**Interfaces:**
- Consumes: `creds::{read, CredsError, claude_dir}`, `api::{fetch_usage, user_agent, ApiError, ApiWindow, USAGE_URL}`, `logs::{LogScanner, Activity}`, `pacing::project`, `snapshot::*`
- Produces:
  - `pub enum FetchResult { Ok, RateLimited, Failed, Skipped }`
  - `#[async_trait] pub trait Provider: Send { fn id(&self) -> &'static str; fn tick_local(&mut self, now: DateTime<Utc>); async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult; fn snapshot(&self) -> Snapshot; }`
  - `pub struct ClaudeProvider` com `ClaudeProvider::new(claude_dir: PathBuf)` e `ClaudeProvider::with_url(claude_dir: PathBuf, usage_url: String)`
  - `pub const CLAUDE_CLI_VERSION: &str = "2.1.288"`

- [ ] **Step 1: Trait em `src-tauri/src/providers/mod.rs`** (substituir inteiro):

```rust
pub mod claude;

use crate::snapshot::Snapshot;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchResult {
    Ok,
    RateLimited,
    Failed,
    /// Não houve chamada (sem login, token vencido).
    Skipped,
}

#[async_trait]
pub trait Provider: Send {
    fn id(&self) -> &'static str;
    /// Atualização local barata (logs, recálculo da projeção). Chamado a cada ~10 s.
    fn tick_local(&mut self, now: DateTime<Utc>);
    /// Busca remota (API). O scheduler decide quando chamar.
    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult;
    fn snapshot(&self) -> Snapshot;
}
```

- [ ] **Step 2: Escrever os testes** — no fim de `src-tauri/src/providers/claude/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{FetchResult, Provider};
    use crate::snapshot::Notice;
    use std::fs;

    const MINIMAL: &str = include_str!("../../../tests/fixtures/usage_minimal.json");

    fn write_creds(dir: &std::path::Path, expires_in_hours: i64) {
        let exp = (Utc::now() + chrono::Duration::hours(expires_in_hours)).timestamp_millis();
        fs::write(
            dir.join(".credentials.json"),
            format!(r#"{{"claudeAiOauth":{{"accessToken":"tok","expiresAt":{exp},"subscriptionType":"pro"}}}}"#),
        )
        .unwrap();
    }

    async fn server_with(status: usize) -> mockito::ServerGuard {
        let mut s = mockito::Server::new_async().await;
        s.mock("GET", "/usage").with_status(status).with_body(MINIMAL).create_async().await;
        s
    }

    #[tokio::test]
    async fn without_credentials_shows_notice_and_skips() {
        let d = tempfile::tempdir().unwrap();
        let mut p = ClaudeProvider::with_url(d.path().into(), "http://127.0.0.1:9/usage".into());
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
        assert_eq!(s.fetched_at, None);
    }

    #[tokio::test]
    async fn successful_fetch_fills_windows_plan_and_pace() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let server = server_with(200).await;
        let mut p = ClaudeProvider::with_url(d.path().into(), format!("{}/usage", server.url()));
        let now = Utc::now();
        assert_eq!(p.fetch_remote(now).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!(s.plan.as_deref(), Some("Pro"));
        assert_eq!(s.windows.len(), 2);
        assert_eq!(s.windows[0].label, "Sessão (5h)");
        assert_eq!(s.fetched_at, Some(now));
        assert!(!s.stale);
        assert_eq!(s.notice, None);
    }

    #[tokio::test]
    async fn rate_limit_after_success_keeps_data_as_stale() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let ok = server_with(200).await;
        let mut p = ClaudeProvider::with_url(d.path().into(), format!("{}/usage", ok.url()));
        p.fetch_remote(Utc::now()).await;
        let limited = server_with(429).await;
        p.usage_url = format!("{}/usage", limited.url());
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::RateLimited);
        let s = p.snapshot();
        assert_eq!(s.windows.len(), 2);
        assert!(s.stale);
        assert_eq!(s.notice, Some(Notice::RateLimited));
    }

    #[tokio::test]
    async fn half_written_credentials_keep_last_state_without_login_notice() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), 2);
        let ok = server_with(200).await;
        let mut p = ClaudeProvider::with_url(d.path().into(), format!("{}/usage", ok.url()));
        p.fetch_remote(Utc::now()).await;
        fs::write(d.path().join(".credentials.json"), r#"{"claudeAiOauth":{"acc"#).unwrap();
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Failed);
        let s = p.snapshot();
        assert_eq!(s.windows.len(), 2);
        assert_eq!(s.notice, None);
        assert_eq!(s.plan.as_deref(), Some("Pro"));
    }

    #[tokio::test]
    async fn expired_token_shows_notice_without_calling_api() {
        let d = tempfile::tempdir().unwrap();
        write_creds(d.path(), -1);
        let mut server = mockito::Server::new_async().await;
        let never = server.mock("GET", "/usage").expect(0).create_async().await;
        let mut p = ClaudeProvider::with_url(d.path().into(), format!("{}/usage", server.url()));
        assert_eq!(p.fetch_remote(Utc::now()).await, FetchResult::Skipped);
        never.assert_async().await;
        assert_eq!(p.snapshot().notice, Some(Notice::TokenExpired));
    }

    #[tokio::test]
    async fn tick_local_reads_logs() {
        let d = tempfile::tempdir().unwrap();
        let proj = d.path().join("projects").join("x");
        fs::create_dir_all(&proj).unwrap();
        let ts = Utc::now().to_rfc3339();
        fs::write(
            proj.join("s.jsonl"),
            format!(r#"{{"type":"assistant","timestamp":"{ts}","requestId":"r","message":{{"id":"m","usage":{{"input_tokens":42}}}}}}"#),
        )
        .unwrap();
        let mut p = ClaudeProvider::with_url(d.path().into(), "http://127.0.0.1:9/usage".into());
        p.tick_local(Utc::now());
        let s = p.snapshot();
        assert_eq!(s.activity.len(), 30);
        assert_eq!(s.today_tokens, 42);
    }
}
```

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers::claude::tests`
Expected: FAIL — `cannot find struct ClaudeProvider`.

- [ ] **Step 4: Implementar** — topo de `src-tauri/src/providers/claude/mod.rs` (manter os `pub mod` e o bloco de testes):

```rust
pub mod api;
pub mod creds;
pub mod logs;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use api::{ApiError, ApiWindow};
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use creds::CredsError;
use logs::{Activity, LogScanner};
use std::path::PathBuf;

/// Versão do Claude Code usada no User-Agent (o endpoint exige esse formato).
pub const CLAUDE_CLI_VERSION: &str = "2.1.288";

fn to_windows(api: &[ApiWindow], now: DateTime<Utc>) -> Vec<UsageWindow> {
    api.iter()
        .map(|w| UsageWindow {
            id: w.id.into(),
            label: w.label.into(),
            used_pct: w.used_pct,
            resets_at: w.resets_at,
            pace: w.resets_at.and_then(|r| pacing::project(w.used_pct, w.length, r, now)),
        })
        .collect()
}

pub struct ClaudeProvider {
    claude_dir: PathBuf,
    usage_url: String,
    http: reqwest::Client,
    scanner: LogScanner,
    activity: Activity,
    plan: Option<String>,
    api_windows: Vec<ApiWindow>,
    windows: Vec<UsageWindow>,
    fetched_at: Option<DateTime<Utc>>,
    stale: bool,
    notice: Option<Notice>,
}

impl ClaudeProvider {
    pub fn new(claude_dir: PathBuf) -> Self {
        Self::with_url(claude_dir, api::USAGE_URL.into())
    }

    pub fn with_url(claude_dir: PathBuf, usage_url: String) -> Self {
        Self {
            claude_dir,
            usage_url,
            http: reqwest::Client::new(),
            scanner: LogScanner::new(),
            activity: Activity { days: Vec::new(), today_tokens: 0 },
            plan: None,
            api_windows: Vec::new(),
            windows: Vec::new(),
            fetched_at: None,
            stale: false,
            notice: None,
        }
    }

    fn mark_failure(&mut self, notice: Notice) {
        self.stale = !self.api_windows.is_empty();
        self.notice = Some(notice);
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> &'static str {
        "claude"
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        self.activity = self.scanner.scan(&self.claude_dir.join("projects"), now.with_timezone(&Local));
        self.windows = to_windows(&self.api_windows, now);
    }

    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        let creds = match creds::read(&self.claude_dir) {
            Ok(c) => c,
            Err(CredsError::Missing) => {
                self.api_windows.clear();
                self.windows.clear();
                self.plan = None;
                self.stale = false;
                self.notice = Some(Notice::NoCredentials);
                return FetchResult::Skipped;
            }
            // Claude Code reescrevendo o arquivo agora: mantém tudo como está.
            Err(CredsError::Unreadable(_)) => return FetchResult::Failed,
        };
        self.plan = creds.plan.clone();
        if creds.is_expired(now) {
            self.mark_failure(Notice::TokenExpired);
            return FetchResult::Skipped;
        }
        let ua = api::user_agent(CLAUDE_CLI_VERSION);
        match api::fetch_usage(&self.http, &self.usage_url, &creds.access_token, &ua).await {
            Ok(ws) => {
                self.api_windows = ws;
                self.windows = to_windows(&self.api_windows, now);
                self.fetched_at = Some(now);
                self.stale = false;
                self.notice = None;
                FetchResult::Ok
            }
            Err(ApiError::Unauthorized) => {
                self.mark_failure(Notice::TokenExpired);
                FetchResult::Failed
            }
            Err(ApiError::RateLimited) => {
                self.mark_failure(Notice::RateLimited);
                FetchResult::RateLimited
            }
            Err(ApiError::Network(_)) | Err(ApiError::BadResponse(_)) => {
                self.mark_failure(Notice::Offline);
                FetchResult::Failed
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            provider: "claude".into(),
            name: "Claude".into(),
            plan: self.plan.clone(),
            windows: self.windows.clone(),
            activity: self.activity.days.clone(),
            today_tokens: self.activity.today_tokens,
            fetched_at: self.fetched_at,
            next_refresh_at: None,
            stale: self.stale,
            notice: self.notice,
        }
    }
}
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml providers`
Expected: todos os testes de `providers` passam (incluindo os 6 novos).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/providers
git commit -m "feat: trait Provider e ClaudeProvider combinando credenciais, API e logs

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Alertas (`alerts.rs`)

**Files:**
- Create: `src-tauri/src/alerts.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod alerts;`; `.plugin(tauri_plugin_notification::init())` no builder)

**Interfaces:**
- Consumes: `config::AlertsConfig`, `snapshot::Snapshot`
- Produces: `pub struct Alert { pub title: String, pub body: String }`, `#[derive(Default)] pub struct AlertState` com `pub fn evaluate(&mut self, snap: &Snapshot, cfg: &AlertsConfig, now: DateTime<Utc>) -> Vec<Alert>`, `pub fn human_duration(d: chrono::Duration) -> String`, `pub fn notify(app: &tauri::AppHandle, alert: &Alert)`.

- [ ] **Step 1: Escrever os testes** (`src-tauri/src/alerts.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{Pace, UsageWindow};
    use chrono::{Duration, TimeZone};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 3, 12, 0, 0).unwrap()
    }

    fn snap(used: f64, pace: Option<Pace>) -> Snapshot {
        Snapshot {
            provider: "claude".into(),
            name: "Claude".into(),
            plan: None,
            windows: vec![UsageWindow {
                id: "five_hour".into(),
                label: "Sessão (5h)".into(),
                used_pct: used,
                resets_at: Some(now() + Duration::hours(3)),
                pace,
            }],
            activity: vec![],
            today_tokens: 0,
            fetched_at: None,
            next_refresh_at: None,
            stale: false,
            notice: None,
        }
    }

    fn cfg() -> AlertsConfig {
        AlertsConfig { enabled: true, thresholds: vec![80, 95], pace: false }
    }

    #[test]
    fn fires_once_per_threshold_and_rearms_when_below() {
        let mut st = AlertState::default();
        assert!(st.evaluate(&snap(70.0, None), &cfg(), now()).is_empty());
        let a = st.evaluate(&snap(81.0, None), &cfg(), now());
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].title, "Claude · Sessão (5h)");
        assert_eq!(a[0].body, "Passou de 80% — agora em 81%");
        assert!(st.evaluate(&snap(85.0, None), &cfg(), now()).is_empty());
        assert_eq!(st.evaluate(&snap(96.0, None), &cfg(), now()).len(), 1);
        assert!(st.evaluate(&snap(10.0, None), &cfg(), now()).is_empty());
        assert_eq!(st.evaluate(&snap(82.0, None), &cfg(), now()).len(), 1);
    }

    #[test]
    fn jumping_past_both_thresholds_fires_both() {
        let mut st = AlertState::default();
        assert_eq!(st.evaluate(&snap(97.0, None), &cfg(), now()).len(), 2);
    }

    #[test]
    fn disabled_or_stale_never_fires() {
        let mut st = AlertState::default();
        let mut c = cfg();
        c.enabled = false;
        assert!(st.evaluate(&snap(99.0, None), &c, now()).is_empty());
        let mut s = snap(99.0, None);
        s.stale = true;
        assert!(st.evaluate(&s, &cfg(), now()).is_empty());
    }

    #[test]
    fn pace_alert_when_limit_comes_before_reset() {
        let mut st = AlertState::default();
        let mut c = cfg();
        c.pace = true;
        let p = Pace { projected_pct: 130.0, limit_at: Some(now() + Duration::minutes(90)) };
        let a = st.evaluate(&snap(50.0, Some(p.clone())), &c, now());
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].body, "No ritmo atual, o limite acaba em 1h 30m");
        assert!(st.evaluate(&snap(50.0, Some(p)), &c, now()).is_empty());
    }

    #[test]
    fn human_durations() {
        assert_eq!(human_duration(Duration::minutes(31 * 60 + 5)), "1d 7h");
        assert_eq!(human_duration(Duration::minutes(134)), "2h 14m");
        assert_eq!(human_duration(Duration::minutes(14)), "14m");
        assert_eq!(human_duration(Duration::seconds(20)), "menos de 1m");
        assert_eq!(human_duration(Duration::minutes(-5)), "menos de 1m");
    }
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml alerts`
Expected: FAIL — `cannot find struct AlertState`.

- [ ] **Step 3: Implementar** (acima dos testes):

```rust
use crate::config::AlertsConfig;
use crate::snapshot::Snapshot;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Alert {
    pub title: String,
    pub body: String,
}

/// Cada limiar dispara uma vez enquanto o uso ficar acima; rearma quando cai.
#[derive(Default)]
pub struct AlertState {
    armed: HashSet<String>,
}

impl AlertState {
    pub fn evaluate(&mut self, snap: &Snapshot, cfg: &AlertsConfig, now: DateTime<Utc>) -> Vec<Alert> {
        let mut out = Vec::new();
        if !cfg.enabled || snap.stale {
            return out;
        }
        for w in &snap.windows {
            let title = format!("{} · {}", snap.name, w.label);
            for t in &cfg.thresholds {
                let key = format!("{}:{}:{}", snap.provider, w.id, t);
                if w.used_pct >= f64::from(*t) {
                    if self.armed.insert(key) {
                        out.push(Alert { title: title.clone(), body: format!("Passou de {t}% — agora em {:.0}%", w.used_pct) });
                    }
                } else {
                    self.armed.remove(&key);
                }
            }
            if cfg.pace {
                let key = format!("{}:{}:pace", snap.provider, w.id);
                let limit_before_reset = match (&w.pace, w.resets_at) {
                    (Some(p), Some(reset)) if w.used_pct < 100.0 => p.limit_at.filter(|l| *l < reset),
                    _ => None,
                };
                match limit_before_reset {
                    Some(limit) => {
                        if self.armed.insert(key) {
                            out.push(Alert {
                                title: title.clone(),
                                body: format!("No ritmo atual, o limite acaba em {}", human_duration(limit - now)),
                            });
                        }
                    }
                    None => {
                        self.armed.remove(&key);
                    }
                }
            }
        }
        out
    }
}

pub fn human_duration(d: chrono::Duration) -> String {
    let mins = d.num_minutes().max(0);
    let (days, hours, m) = (mins / 1440, (mins % 1440) / 60, mins % 60);
    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {m}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        "menos de 1m".into()
    }
}

pub fn notify(app: &tauri::AppHandle, alert: &Alert) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification().builder().title(&alert.title).body(&alert.body).show();
}
```

Em `lib.rs`, adicionar `pub mod alerts;` e `.plugin(tauri_plugin_notification::init())` logo após o plugin opener.

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml alerts`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/alerts.rs src-tauri/src/lib.rs
git commit -m "feat: alertas por limiar e por previsao estourando

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Backoff, estado compartilhado, scheduler e comandos

**Files:**
- Create: `src-tauri/src/state.rs`, `src-tauri/src/scheduler.rs`, `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (substituir inteiro)

**Interfaces:**
- Consumes: `Provider`, `ClaudeProvider`, `creds::claude_dir`, `Config`, `config::{load, save}`, `AlertState`, `alerts::notify`
- Produces:
  - `state::Shared { snapshots: Mutex<Vec<Snapshot>>, config: Mutex<Config>, config_path: PathBuf, force: AtomicBool, wake: Notify }` com `Shared::new(config_path, config)` e `Shared::request_refresh(&self)`
  - `scheduler::Backoff` com `new(refresh_minutes: u32)`, `set_refresh_minutes(&mut self, m: u32)`, `record(&mut self, r: FetchResult)`, `delay(&self) -> std::time::Duration`
  - `scheduler::spawn(app: AppHandle, shared: Arc<Shared>)`
  - comandos `get_state() -> AppState { snapshots, config, version }`, `save_config(config: Config) -> Result<Config, String>`, `refresh_now()`, `hide_window()`
  - eventos `snapshot` (`Vec<Snapshot>`), `config` (`Config`)
  - `commands::apply_config(app: &AppHandle, cfg: &Config)` (autostart; a trava é completada na Task 11)

- [ ] **Step 1: Escrever os testes do Backoff** (`src-tauri/src/scheduler.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::FetchResult;
    use std::time::Duration;

    const MIN: Duration = Duration::from_secs(60);

    #[test]
    fn starts_at_refresh_interval() {
        assert_eq!(Backoff::new(5).delay(), 5 * MIN);
        assert_eq!(Backoff::new(1).delay(), MIN);
    }

    #[test]
    fn rate_limit_doubles_up_to_30_minutes() {
        let mut b = Backoff::new(5);
        b.record(FetchResult::RateLimited);
        assert_eq!(b.delay(), 10 * MIN);
        b.record(FetchResult::RateLimited);
        assert_eq!(b.delay(), 20 * MIN);
        b.record(FetchResult::RateLimited);
        assert_eq!(b.delay(), 30 * MIN);
        b.record(FetchResult::RateLimited);
        assert_eq!(b.delay(), 30 * MIN);
    }

    #[test]
    fn success_failure_or_skip_return_to_base() {
        for r in [FetchResult::Ok, FetchResult::Failed, FetchResult::Skipped] {
            let mut b = Backoff::new(5);
            b.record(FetchResult::RateLimited);
            b.record(r);
            assert_eq!(b.delay(), 5 * MIN, "{r:?}");
        }
    }

    #[test]
    fn changing_interval_resets_delay() {
        let mut b = Backoff::new(5);
        b.record(FetchResult::RateLimited);
        b.set_refresh_minutes(1);
        assert_eq!(b.delay(), MIN);
    }
}
```

Adicionar `pub mod scheduler;` em `lib.rs`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml scheduler`
Expected: FAIL — `cannot find struct Backoff`.

- [ ] **Step 3: Implementar `state.rs`**:

```rust
use crate::config::Config;
use crate::snapshot::Snapshot;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tokio::sync::Notify;

pub struct Shared {
    pub snapshots: Mutex<Vec<Snapshot>>,
    pub config: Mutex<Config>,
    pub config_path: PathBuf,
    pub force: AtomicBool,
    pub wake: Notify,
}

impl Shared {
    pub fn new(config_path: PathBuf, config: Config) -> Self {
        Self {
            snapshots: Mutex::new(Vec::new()),
            config: Mutex::new(config),
            config_path,
            force: AtomicBool::new(false),
            wake: Notify::new(),
        }
    }

    /// Pede uma busca remota imediata (respeitando o mínimo de 60 s).
    pub fn request_refresh(&self) {
        self.force.store(true, Ordering::SeqCst);
        self.wake.notify_one();
    }

    pub fn take_force(&self) -> bool {
        self.force.swap(false, Ordering::SeqCst)
    }

    /// Acorda o laço sem forçar busca remota (ex.: config mudou).
    pub fn wake_up(&self) {
        self.wake.notify_one();
    }
}
```

- [ ] **Step 4: Implementar `scheduler.rs`** (acima dos testes):

```rust
use crate::alerts::{self, AlertState};
use crate::providers::claude::{creds, ClaudeProvider};
use crate::providers::{FetchResult, Provider};
use crate::state::Shared;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub const MAX_BACKOFF: Duration = Duration::from_secs(30 * 60);
const MIN_GAP: chrono::Duration = chrono::Duration::seconds(60);
const LOCAL_TICK: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct Backoff {
    base: Duration,
    current: Duration,
}

fn base_for(minutes: u32) -> Duration {
    Duration::from_secs(u64::from(minutes.max(1)) * 60)
}

impl Backoff {
    pub fn new(refresh_minutes: u32) -> Self {
        let base = base_for(refresh_minutes);
        Self { base, current: base }
    }

    pub fn set_refresh_minutes(&mut self, minutes: u32) {
        self.base = base_for(minutes);
        self.current = self.base;
    }

    pub fn record(&mut self, r: FetchResult) {
        self.current = match r {
            FetchResult::RateLimited => (self.current * 2).clamp(self.base, MAX_BACKOFF),
            _ => self.base,
        };
    }

    pub fn delay(&self) -> Duration {
        self.current
    }
}

struct Slot {
    provider: Box<dyn Provider>,
    backoff: Backoff,
    next_remote: DateTime<Utc>,
    last_remote: Option<DateTime<Utc>>,
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    tauri::async_runtime::spawn(async move {
        let mut refresh = shared.config.lock().unwrap().refresh_minutes;
        let mut slots = vec![Slot {
            provider: Box::new(ClaudeProvider::new(creds::claude_dir())),
            backoff: Backoff::new(refresh),
            next_remote: Utc::now(),
            last_remote: None,
        }];
        let mut alert_state = AlertState::default();

        loop {
            let forced = shared.take_force();
            let cfg = shared.config.lock().unwrap().clone();
            let now = Utc::now();
            if cfg.refresh_minutes != refresh {
                refresh = cfg.refresh_minutes;
                for s in &mut slots {
                    s.backoff.set_refresh_minutes(refresh);
                    s.next_remote = s.next_remote.min(now + chrono::Duration::from_std(s.backoff.delay()).unwrap());
                }
            }

            let mut snaps = Vec::new();
            for s in &mut slots {
                if !cfg.provider_enabled(s.provider.id()) {
                    continue;
                }
                s.provider.tick_local(now);
                let gap_ok = s.last_remote.is_none_or(|t| now - t >= MIN_GAP);
                if (forced || now >= s.next_remote) && gap_ok {
                    let r = s.provider.fetch_remote(now).await;
                    s.backoff.record(r);
                    s.last_remote = Some(now);
                    s.next_remote = now + chrono::Duration::from_std(s.backoff.delay()).unwrap();
                }
                let mut snap = s.provider.snapshot();
                snap.next_refresh_at = Some(s.next_remote);
                for a in alert_state.evaluate(&snap, &cfg.alerts, now) {
                    alerts::notify(&app, &a);
                }
                snaps.push(snap);
            }

            crate::tray::update(&app, &snaps);
            *shared.snapshots.lock().unwrap() = snaps.clone();
            let _ = app.emit("snapshot", &snaps);

            tokio::select! {
                _ = tokio::time::sleep(LOCAL_TICK) => {}
                _ = shared.wake.notified() => {}
            }
        }
    });
}
```

`crate::tray::update` só existe na Task 11 — para esta task compilar, criar `src-tauri/src/tray.rs` provisório:

```rust
use crate::snapshot::Snapshot;
use tauri::AppHandle;

pub fn update(_app: &AppHandle, _snaps: &[Snapshot]) {}
```

- [ ] **Step 5: Implementar `commands.rs`**:

```rust
use crate::config::{self, Config};
use crate::snapshot::Snapshot;
use crate::state::Shared;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, WebviewWindow};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub snapshots: Vec<Snapshot>,
    pub config: Config,
    pub version: String,
}

#[tauri::command]
pub fn get_state(app: AppHandle, shared: State<'_, Arc<Shared>>) -> AppState {
    AppState {
        snapshots: shared.snapshots.lock().unwrap().clone(),
        config: shared.config.lock().unwrap().clone(),
        version: app.package_info().version.to_string(),
    }
}

#[tauri::command]
pub fn save_config(app: AppHandle, shared: State<'_, Arc<Shared>>, config: Config) -> Result<Config, String> {
    let cfg = config.normalized();
    config::save(&shared.config_path, &cfg).map_err(|e| e.to_string())?;
    *shared.config.lock().unwrap() = cfg.clone();
    apply_config(&app, &cfg);
    let _ = app.emit("config", &cfg);
    shared.wake_up();
    Ok(cfg)
}

#[tauri::command]
pub fn refresh_now(shared: State<'_, Arc<Shared>>) {
    shared.request_refresh();
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

/// Aplica efeitos colaterais da config (início com o Windows; trava na Task 11).
pub fn apply_config(app: &AppHandle, cfg: &Config) {
    if !cfg!(debug_assertions) {
        use tauri_plugin_autostart::ManagerExt;
        let launcher = app.autolaunch();
        let _ = if cfg.start_with_windows { launcher.enable() } else { launcher.disable() };
    }
}
```

- [ ] **Step 6: `lib.rs` completo** (substituir inteiro):

```rust
pub mod alerts;
pub mod commands;
pub mod config;
pub mod pacing;
pub mod providers;
pub mod scheduler;
pub mod snapshot;
pub mod state;
pub mod tray;

use std::sync::Arc;
use tauri::{Manager, WindowEvent};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let config_path = app.path().app_config_dir()?.join("config.json");
            let cfg = config::load(&config_path);
            let shared = Arc::new(state::Shared::new(config_path, cfg.clone()));
            app.manage(shared.clone());

            let win = app.get_webview_window("main").expect("janela main");
            let w = win.clone();
            win.on_window_event(move |e| {
                if let WindowEvent::CloseRequested { api, .. } = e {
                    api.prevent_close();
                    let _ = w.hide();
                }
            });

            commands::apply_config(app.handle(), &cfg);
            win.show()?;
            scheduler::spawn(app.handle().clone(), shared);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::save_config,
            commands::refresh_now,
            commands::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Pacer");
}
```

- [ ] **Step 7: Rodar testes e checar o app**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: todos passam (Backoff: 4 novos).

Run: `bun run tauri dev`, abrir o DevTools (Ctrl+Shift+I) e no console:

```js
await window.__TAURI_INTERNALS__.invoke("get_state")
```

Expected: em até ~10 s, `snapshots[0]` com `provider: "claude"`, `plan: "Pro"`, `windows` com "Sessão (5h)" e "Semanal", `activity` com 30 dias. Conferir que `%APPDATA%\dev.sthevan.pacer\` foi criado (pasta pode ficar vazia até salvar config).

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src
git commit -m "feat: scheduler com backoff, estado compartilhado e comandos IPC

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Janela (ancoragem, acrílico, altura) e bandeja

**Files:**
- Create: `src-tauri/src/window.rs`
- Modify: `src-tauri/src/tray.rs` (substituir o provisório), `src-tauri/src/lib.rs`, `src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: `snapshot::{severity, Severity, Snapshot}`, `state::Shared`
- Produces:
  - `window::Rect { x, y, w, h }`, `window::anchored(area: Rect, w: i32, h: i32, margin: i32) -> (i32, i32)`, `window::clamp(area: Rect, x: i32, y: i32, w: i32, h: i32) -> (i32, i32)`, `window::anchor(&WebviewWindow)`, `window::keep_inside(&WebviewWindow)`, `window::apply_effects(&WebviewWindow)`, comando `set_window_height(height: f64)`
  - `tray::render_icon(pct: f64, sev: Severity) -> Vec<u8>` (RGBA 32×32), `tray::create(&AppHandle) -> tauri::Result<()>`, `tray::update(&AppHandle, &[Snapshot])`, `tray::show_main(&AppHandle)`
  - evento `open-settings` (sem payload) emitido pelo menu da bandeja

- [ ] **Step 1: Testes de geometria** (`src-tauri/src/window.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const W: i32 = 300;
    const H: i32 = 520;

    #[test]
    fn anchors_bottom_right_above_bottom_taskbar() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 }; // taskbar de 48px embaixo
        assert_eq!(anchored(area, W, H, 12), (1920 - 300 - 12, 1032 - 520 - 12));
    }

    #[test]
    fn respects_top_taskbar_and_offset_monitor() {
        let top_bar = Rect { x: 0, y: 48, w: 1920, h: 1032 };
        assert_eq!(anchored(top_bar, W, H, 12), (1608, 48 + 1032 - 520 - 12));
        let second = Rect { x: 1920, y: -200, w: 1280, h: 984 };
        assert_eq!(anchored(second, W, H, 12), (1920 + 1280 - 312, -200 + 984 - 532));
    }

    #[test]
    fn clamp_pulls_window_back_inside() {
        let area = Rect { x: 0, y: 0, w: 1920, h: 1032 };
        assert_eq!(clamp(area, 1800, 900, W, H), (1620, 512));
        assert_eq!(clamp(area, -50, -10, W, H), (0, 0));
        assert_eq!(clamp(area, 100, 100, W, H), (100, 100));
    }

    #[test]
    fn clamp_handles_window_bigger_than_area() {
        let area = Rect { x: 10, y: 10, w: 200, h: 200 };
        assert_eq!(clamp(area, 50, 50, W, H), (10, 10));
    }
}
```

- [ ] **Step 2: Testes do ícone** — em `src-tauri/src/tray.rs`, substituir o provisório por apenas este bloco por enquanto (mantendo `pub fn update` provisório no topo para compilar):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn px(buf: &[u8], x: u32, y: u32) -> [u8; 4] {
        let i = ((y * SIZE + x) * 4) as usize;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn ring_fills_clockwise_from_top() {
        let buf = render_icon(50.0, Severity::Ok);
        assert_eq!(buf.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(px(&buf, 16, 2), [0x1f, 0x6f, 0xeb, 255]); // topo, logo depois do 0° → preenchido
        assert_eq!(px(&buf, 2, 16), [0x6e, 0x76, 0x81, 255]); // esquerda (~270°) → trilho cinza
        assert_eq!(px(&buf, 16, 16)[3], 0); // centro transparente
        assert_eq!(px(&buf, 0, 0)[3], 0); // canto transparente
    }

    #[test]
    fn color_follows_severity() {
        assert_eq!(px(&render_icon(95.0, Severity::Critical), 16, 2), [0xf8, 0x51, 0x49, 255]);
        assert_eq!(px(&render_icon(80.0, Severity::Warn), 16, 2), [0xd2, 0x99, 0x22, 255]);
    }

    #[test]
    fn zero_percent_is_all_track() {
        assert_eq!(px(&render_icon(0.0, Severity::Ok), 16, 2), [0x6e, 0x76, 0x81, 255]);
    }
}
```

Adicionar `pub mod window;` em `lib.rs`.

- [ ] **Step 3: Rodar e ver falhar**

Run: `cargo test --manifest-path src-tauri/Cargo.toml window tray`
Expected: FAIL — `cannot find function anchored` / `render_icon`.

- [ ] **Step 4: Implementar `window.rs`** (acima dos testes):

```rust
use tauri::{LogicalSize, PhysicalPosition, WebviewWindow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub const MARGIN: f64 = 12.0;
pub const WIDTH: f64 = 300.0;

pub fn anchored(area: Rect, w: i32, h: i32, margin: i32) -> (i32, i32) {
    (area.x + area.w - w - margin, area.y + area.h - h - margin)
}

pub fn clamp(area: Rect, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    let max_x = (area.x + area.w - w).max(area.x);
    let max_y = (area.y + area.h - h).max(area.y);
    (x.clamp(area.x, max_x), y.clamp(area.y, max_y))
}

fn work_area(win: &WebviewWindow) -> Option<Rect> {
    let m = win
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| win.primary_monitor().ok().flatten())?;
    let wa = m.work_area();
    Some(Rect { x: wa.position.x, y: wa.position.y, w: wa.size.width as i32, h: wa.size.height as i32 })
}

/// Cola no canto inferior direito da área útil do monitor atual.
pub fn anchor(win: &WebviewWindow) {
    let (Some(area), Ok(size)) = (work_area(win), win.outer_size()) else { return };
    let margin = (MARGIN * win.scale_factor().unwrap_or(1.0)).round() as i32;
    let (x, y) = anchored(area, size.width as i32, size.height as i32, margin);
    let _ = win.set_position(PhysicalPosition::new(x, y));
}

/// Nunca deixa a janela ficar fora da área útil (sob a taskbar ou fora da tela).
pub fn keep_inside(win: &WebviewWindow) {
    let (Some(area), Ok(pos), Ok(size)) = (work_area(win), win.outer_position(), win.outer_size()) else { return };
    let (x, y) = clamp(area, pos.x, pos.y, size.width as i32, size.height as i32);
    if (x, y) != (pos.x, pos.y) {
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
}

/// Acrílico do Windows 11; se falhar, o CSS já tem fundo próprio.
pub fn apply_effects(win: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = window_vibrancy::apply_acrylic(win, Some((24, 28, 34, 180)));
    }
}

/// O frontend informa a altura do conteúdo; a janela cresce para cima (base fixa).
#[tauri::command]
pub fn set_window_height(window: WebviewWindow, height: f64) -> Result<(), String> {
    let height = height.clamp(160.0, 900.0);
    let old_pos = window.outer_position().map_err(|e| e.to_string())?;
    let old_size = window.outer_size().map_err(|e| e.to_string())?;
    window.set_size(LogicalSize::new(WIDTH, height)).map_err(|e| e.to_string())?;
    let new_size = window.outer_size().map_err(|e| e.to_string())?;
    let bottom = old_pos.y + old_size.height as i32;
    let _ = window.set_position(PhysicalPosition::new(old_pos.x, bottom - new_size.height as i32));
    keep_inside(&window);
    Ok(())
}
```

- [ ] **Step 5: Implementar `tray.rs`** (substituir o topo provisório, manter os testes):

```rust
use crate::snapshot::{severity, Severity, Snapshot};
use std::f64::consts::TAU;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub const SIZE: u32 = 32;
const TRACK: [u8; 3] = [0x6e, 0x76, 0x81];

fn color(s: Severity) -> [u8; 3] {
    match s {
        Severity::Ok => [0x1f, 0x6f, 0xeb],
        Severity::Warn => [0xd2, 0x99, 0x22],
        Severity::Critical => [0xf8, 0x51, 0x49],
    }
}

/// Anel 32×32: trilho cinza + arco colorido proporcional ao uso, horário a partir do topo.
pub fn render_icon(pct: f64, sev: Severity) -> Vec<u8> {
    let mut px = vec![0u8; (SIZE * SIZE * 4) as usize];
    let c = (SIZE as f64 - 1.0) / 2.0;
    let (r_out, r_in) = (15.0, 10.0);
    let frac = (pct / 100.0).clamp(0.0, 1.0);
    let col = color(sev);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (dx, dy) = (x as f64 - c, y as f64 - c);
            let d = (dx * dx + dy * dy).sqrt();
            if d > r_out || d < r_in {
                continue;
            }
            let ang = (dx.atan2(-dy) + TAU) % TAU;
            let rgb = if frac > 0.0 && ang / TAU <= frac { col } else { TRACK };
            let i = ((y * SIZE + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    }
    px
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_main(app);
        }
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let refresh = MenuItem::with_id(app, "refresh", "Atualizar agora", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&refresh, &settings, &sep, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(Image::new_owned(render_icon(0.0, Severity::Ok), SIZE, SIZE))
        .tooltip("Pacer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "refresh" => app.state::<std::sync::Arc<crate::state::Shared>>().request_refresh(),
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                toggle_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

static LAST: Mutex<Option<(u8, Severity)>> = Mutex::new(None);

/// Troca o ícone só quando a % arredondada ou a severidade mudam.
pub fn update(app: &AppHandle, snaps: &[Snapshot]) {
    let (pct, sev) = snaps
        .iter()
        .flat_map(|s| s.windows.iter())
        .map(|w| (w.used_pct, severity(w.used_pct, w.pace.as_ref())))
        .fold((0.0_f64, Severity::Ok), |(p, s), (wp, ws)| (p.max(wp), s.max(ws)));
    let key = (pct.round() as u8, sev);
    let mut last = LAST.lock().unwrap();
    if *last == Some(key) {
        return;
    }
    *last = Some(key);
    if let Some(t) = app.tray_by_id("main") {
        let _ = t.set_icon(Some(Image::new_owned(render_icon(pct, sev), SIZE, SIZE)));
        let _ = t.set_tooltip(Some(format!("Pacer · {}%", key.0)));
    }
}
```

- [ ] **Step 6: Ligar no app**

Em `lib.rs`, dentro do `setup`, logo depois de `let win = ...`:

```rust
            window::apply_effects(&win);
            window::anchor(&win);
```

Trocar o `on_window_event` por:

```rust
            win.on_window_event(move |e| match e {
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = w.hide();
                }
                WindowEvent::Moved(_) => window::keep_inside(&w),
                _ => {}
            });
            tray::create(app.handle())?;
```

E adicionar `window::set_window_height` ao `generate_handler!`.

Em `commands.rs`, no fim de `apply_config`:

```rust
    if cfg.lock_position {
        use tauri::Manager;
        if let Some(w) = app.get_webview_window("main") {
            crate::window::anchor(&w);
        }
    }
```

- [ ] **Step 7: Rodar testes e verificar à mão**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: todos passam (window: 4, tray: 3 novos).

Run: `bun run tauri dev` e conferir:
- janela no canto inferior direito, acima da barra de tarefas, com cantos arredondados do Windows 11;
- ícone-anel na bandeja; clique esquerdo esconde/mostra; menu direito com "Atualizar agora", "Configurações", "Sair"; "Sair" fecha o app;
- fechar a janela (Alt+F4) só esconde.

Se o acrílico deixar bordas/cantos estranhos, remover a chamada `window::apply_effects(&win)` e registrar isso no commit — o CSS da Task 13 já tem fundo próprio.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src
git commit -m "feat: janela ancorada acima da taskbar, acrilico e bandeja com anel de uso

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: Biblioteca do frontend (tipos, formatação, grade, severidade)

**Files:**
- Create: `src/lib/types.ts`, `src/lib/severity.ts`, `src/lib/severity.test.ts`, `src/lib/format.ts`, `src/lib/format.test.ts`, `src/lib/grid.ts`, `src/lib/grid.test.ts`

**Interfaces:**
- Produces: tipos `Severity`, `Notice`, `Pace`, `UsageWindow`, `DayActivity`, `Snapshot`, `AlertsConfig`, `Config`, `AppState`; `severity(usedPct, pace)`; `formatDuration(ms)`, `formatTokens(n)`, `formatAgo(iso, now)`, `formatDay(isoDate)`; `Cell`, `levelFor(tokens, max)`, `buildGrid(days)`.

- [ ] **Step 1: Tipos** (`src/lib/types.ts`):

```ts
export type Severity = "ok" | "warn" | "critical";
export type Notice = "noCredentials" | "tokenExpired" | "rateLimited" | "offline";

export interface Pace {
  projectedPct: number;
  limitAt: string | null;
}

export interface UsageWindow {
  id: string;
  label: string;
  usedPct: number;
  resetsAt: string | null;
  pace: Pace | null;
}

export interface DayActivity {
  date: string; // "2026-10-03"
  tokens: number;
  messages: number;
}

export interface Snapshot {
  provider: string;
  name: string;
  plan: string | null;
  windows: UsageWindow[];
  activity: DayActivity[];
  todayTokens: number;
  fetchedAt: string | null;
  nextRefreshAt: string | null;
  stale: boolean;
  notice: Notice | null;
}

export interface AlertsConfig {
  enabled: boolean;
  thresholds: number[];
  pace: boolean;
}

export interface Config {
  startWithWindows: boolean;
  lockPosition: boolean;
  refreshMinutes: number;
  alerts: AlertsConfig;
  providers: { claude: { enabled: boolean } };
}

export interface AppState {
  snapshots: Snapshot[];
  config: Config;
  version: string;
}
```

- [ ] **Step 2: Escrever os testes**

`src/lib/severity.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { severity } from "./severity";

describe("severity", () => {
  it("segue os mesmos limiares do Rust", () => {
    expect(severity(74.9, null)).toBe("ok");
    expect(severity(75, null)).toBe("warn");
    expect(severity(90, null)).toBe("warn");
    expect(severity(90.1, null)).toBe("critical");
  });
  it("projeção acima de 100% é crítica", () => {
    expect(severity(30, { projectedPct: 101, limitAt: null })).toBe("critical");
    expect(severity(30, { projectedPct: 100, limitAt: null })).toBe("ok");
  });
});
```

`src/lib/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { formatAgo, formatDay, formatDuration, formatTokens } from "./format";

const MIN = 60_000;

describe("formatDuration", () => {
  it("formata dias, horas e minutos", () => {
    expect(formatDuration((31 * 60 + 5) * MIN)).toBe("1d 7h");
    expect(formatDuration(134 * MIN)).toBe("2h 14m");
    expect(formatDuration(14 * MIN)).toBe("14m");
    expect(formatDuration(20_000)).toBe("menos de 1m");
    expect(formatDuration(-5 * MIN)).toBe("menos de 1m");
  });
});

describe("formatTokens", () => {
  it("usa vírgula decimal e sufixos", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(999)).toBe("999");
    expect(formatTokens(850_000)).toBe("850k");
    expect(formatTokens(4_200_000)).toBe("4,2M");
    expect(formatTokens(96_000_000)).toBe("96M");
    expect(formatTokens(1_250_000_000)).toBe("1,3B");
  });
});

describe("formatAgo", () => {
  const now = Date.parse("2026-10-03T12:00:00Z");
  it("diz há quanto tempo", () => {
    expect(formatAgo("2026-10-03T11:59:40Z", now)).toBe("agora");
    expect(formatAgo("2026-10-03T11:57:00Z", now)).toBe("há 3 min");
    expect(formatAgo("2026-10-03T09:00:00Z", now)).toBe("há 3 h");
    expect(formatAgo("2026-10-01T12:00:00Z", now)).toBe("há 2 d");
  });
});

describe("formatDay", () => {
  it("formata dia e mês curto sem depender do fuso", () => {
    expect(formatDay("2026-09-12")).toBe("12 set");
    expect(formatDay("2026-01-01")).toBe("1 jan");
  });
});
```

`src/lib/grid.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { buildGrid, levelFor } from "./grid";
import type { DayActivity } from "./types";

function last30(endIso: string, tokens: (i: number) => number): DayActivity[] {
  const [y, m, d] = endIso.split("-").map(Number);
  const end = Date.UTC(y, m - 1, d);
  return Array.from({ length: 30 }, (_, i) => {
    const date = new Date(end - (29 - i) * 86_400_000).toISOString().slice(0, 10);
    return { date, tokens: tokens(i), messages: i };
  });
}

describe("levelFor", () => {
  it("divide em 5 níveis relativos ao máximo", () => {
    expect(levelFor(0, 100)).toBe(0);
    expect(levelFor(1, 100)).toBe(1);
    expect(levelFor(25, 100)).toBe(1);
    expect(levelFor(26, 100)).toBe(2);
    expect(levelFor(75, 100)).toBe(3);
    expect(levelFor(100, 100)).toBe(4);
    expect(levelFor(5, 0)).toBe(0);
  });
});

describe("buildGrid", () => {
  it("monta colunas por semana com segunda na primeira linha", () => {
    // 30 dias até sáb 03/10/2026 → começa sex 04/09
    const weeks = buildGrid(last30("2026-10-03", (i) => i * 10));
    expect(weeks).toHaveLength(5);
    expect(weeks.every((w) => w.length === 7)).toBe(true);
    expect(weeks[0][3]).toBeNull(); // quinta antes do início
    expect(weeks[0][4]?.date).toBe("2026-09-04");
    expect(weeks[4][5]?.date).toBe("2026-10-03");
    expect(weeks[4][6]).toBeNull(); // domingo depois de hoje
    expect(weeks[4][5]?.level).toBe(4);
    expect(weeks[0][4]?.level).toBe(0);
  });

  it("lista vazia não quebra", () => {
    expect(buildGrid([])).toEqual([]);
  });
});
```

- [ ] **Step 3: Rodar e ver falhar**

Run: `bun run test`
Expected: FAIL — `Failed to resolve import "./severity"` (e os outros).

- [ ] **Step 4: Implementar**

`src/lib/severity.ts`:

```ts
import type { Pace, Severity } from "./types";

/** Mesma regra de src-tauri/src/snapshot.rs. */
export function severity(usedPct: number, pace: Pace | null): Severity {
  if (usedPct > 90 || (pace !== null && pace.projectedPct > 100)) return "critical";
  if (usedPct >= 75) return "warn";
  return "ok";
}
```

`src/lib/format.ts`:

```ts
const MONTHS = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
const oneDecimal = new Intl.NumberFormat("pt-BR", { maximumFractionDigits: 1 });

export function formatDuration(ms: number): string {
  const mins = Math.max(0, Math.floor(ms / 60_000));
  const d = Math.floor(mins / 1440);
  const h = Math.floor((mins % 1440) / 60);
  const m = mins % 60;
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return "menos de 1m";
}

export function formatTokens(n: number): string {
  if (n >= 1e9) return `${oneDecimal.format(n / 1e9)}B`;
  if (n >= 1e6) return `${oneDecimal.format(n / 1e6)}M`;
  if (n >= 1e3) return `${Math.round(n / 1e3)}k`;
  return String(n);
}

export function formatAgo(iso: string, now: number): string {
  const mins = Math.floor((now - Date.parse(iso)) / 60_000);
  if (mins < 1) return "agora";
  if (mins < 60) return `há ${mins} min`;
  if (mins < 1440) return `há ${Math.floor(mins / 60)} h`;
  return `há ${Math.floor(mins / 1440)} d`;
}

/** "2026-09-12" → "12 set" (parse manual, sem fuso). */
export function formatDay(isoDate: string): string {
  const [, m, d] = isoDate.split("-").map(Number);
  return `${d} ${MONTHS[m - 1]}`;
}
```

`src/lib/grid.ts`:

```ts
import type { DayActivity } from "./types";

export type Level = 0 | 1 | 2 | 3 | 4;
export interface Cell extends DayActivity {
  level: Level;
}

export function levelFor(tokens: number, max: number): Level {
  if (tokens <= 0 || max <= 0) return 0;
  return Math.min(4, Math.max(1, Math.ceil((tokens / max) * 4))) as Level;
}

/** Segunda = 0 … domingo = 6, calculado em UTC para não depender do fuso. */
function weekday(isoDate: string): number {
  const [y, m, d] = isoDate.split("-").map(Number);
  return (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
}

/** Colunas = semanas; cada coluna tem 7 posições (seg→dom); fora dos 30 dias = null. */
export function buildGrid(days: DayActivity[]): (Cell | null)[][] {
  if (days.length === 0) return [];
  const max = Math.max(...days.map((d) => d.tokens));
  const slots: (Cell | null)[] = Array(weekday(days[0].date)).fill(null);
  for (const d of days) slots.push({ ...d, level: levelFor(d.tokens, max) });
  while (slots.length % 7 !== 0) slots.push(null);
  const weeks: (Cell | null)[][] = [];
  for (let i = 0; i < slots.length; i += 7) weeks.push(slots.slice(i, i + 7));
  return weeks;
}
```

- [ ] **Step 5: Rodar e ver passar**

Run: `bun run test`
Expected: 3 arquivos, todos os testes passam.

- [ ] **Step 6: Commit**

```bash
git add src/lib
git commit -m "feat: tipos, formatacao pt-BR, grade de 30 dias e severidade no frontend

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Tela principal (visual v3)

**Files:**
- Create: `src/icons/claude.svg`, `src/icons/icons.tsx`, `src/lib/hooks.ts`, `src/components/Header.tsx`, `src/components/ProviderTabs.tsx`, `src/components/UsageBar.tsx`, `src/components/ProviderCard.tsx`, `src/components/ActivityGrid.tsx`, `src/components/Footer.tsx`, `src/components/MainView.tsx`
- Modify: `src/App.tsx`, `src/styles.css` (substituir inteiros)

**Interfaces:**
- Consumes: tipos e funções da Task 12; comandos `get_state`, `refresh_now`, `hide_window`, `set_window_height`; eventos `snapshot`, `config`, `open-settings`.
- Produces: `Icon({ name, size })`, `IconName`, `ClaudeLogo({ size })`, `usePacer(): AppState | null`, `useNow(ms?): number`, `useAutoHeight(ref)`, `Header({ title, sub?, draggable, leading?, actions? })`, `MainView({ state, now, onSettings })`. `App` aceita a view `"settings"` (implementada na Task 14).

- [ ] **Step 1: Logo do Claude** — baixar o SVG do ai-usagebar (MIT; crédito no README da Task 15):

```bash
curl -sL -o src/icons/claude.svg https://raw.githubusercontent.com/akitaonrails/ai-usagebar/HEAD/windows/popover/src/icons/providers/anthropic.svg
```

Abrir o arquivo e garantir: um único `<svg ... viewBox="...">`, sem `width`/`height` fixos, e `fill="currentColor"` no `<svg>` (trocar qualquer `fill="#..."` por `currentColor`).

- [ ] **Step 2: Ícones** (`src/icons/icons.tsx`):

```tsx
import type { ReactNode } from "react";
import claudeSvg from "./claude.svg?raw";

export type IconName = "refresh" | "gear" | "back" | "sliders" | "bell" | "plug" | "plus" | "alert" | "trend";

const PATHS: Record<IconName, ReactNode> = {
  refresh: (
    <>
      <path d="M21 12a9 9 0 1 1-2.64-6.36" />
      <path d="M21 3v6h-6" />
    </>
  ),
  gear: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </>
  ),
  back: <path d="M15 18l-6-6 6-6" />,
  sliders: <path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6" />,
  bell: (
    <>
      <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 0 1-3.46 0" />
    </>
  ),
  plug: (
    <>
      <path d="M9 2v6M15 2v6M6 8h12v4a6 6 0 0 1-12 0z" />
      <path d="M12 18v4" />
    </>
  ),
  plus: <path d="M12 5v14M5 12h14" />,
  alert: (
    <>
      <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
      <path d="M12 9v4M12 17h.01" />
    </>
  ),
  trend: (
    <>
      <path d="M23 6l-9.5 9.5-5-5L1 18" />
      <path d="M17 6h6v6" />
    </>
  ),
};

export function Icon({ name, size = 15 }: { name: IconName; size?: number }) {
  return (
    <svg className="i" width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
      {PATHS[name]}
    </svg>
  );
}

export function ClaudeLogo({ size = 15 }: { size?: number }) {
  return (
    <span className="logo" style={{ width: size, height: size }} aria-hidden="true" dangerouslySetInnerHTML={{ __html: claudeSvg }} />
  );
}
```

- [ ] **Step 3: Hooks** (`src/lib/hooks.ts`):

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type RefObject, useEffect, useState } from "react";
import type { AppState, Config, Snapshot } from "./types";

export function usePacer(): AppState | null {
  const [state, setState] = useState<AppState | null>(null);
  useEffect(() => {
    invoke<AppState>("get_state").then(setState);
    const offSnap = listen<Snapshot[]>("snapshot", (e) => setState((s) => (s ? { ...s, snapshots: e.payload } : s)));
    const offCfg = listen<Config>("config", (e) => setState((s) => (s ? { ...s, config: e.payload } : s)));
    return () => {
      offSnap.then((f) => f());
      offCfg.then((f) => f());
    };
  }, []);
  return state;
}

export function useNow(ms = 15_000): number {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), ms);
    return () => clearInterval(t);
  }, [ms]);
  return now;
}

/** Mantém a janela do tamanho do conteúdo. */
export function useAutoHeight(ref: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => {
      invoke("set_window_height", { height: Math.ceil(el.getBoundingClientRect().height) });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [ref]);
}
```

- [ ] **Step 4: Componentes**

`src/components/Header.tsx`:

```tsx
import type { ReactNode } from "react";

interface Props {
  title: string;
  sub?: string;
  draggable: boolean;
  leading?: ReactNode;
  actions?: ReactNode;
}

export function Header({ title, sub, draggable, leading, actions }: Props) {
  const drag = draggable ? { "data-tauri-drag-region": "" } : {};
  return (
    <header className="top" {...drag}>
      {leading}
      <h4>
        {title}
        {sub && <span className="sub">{sub}</span>}
      </h4>
      {actions}
    </header>
  );
}
```

`src/components/ProviderTabs.tsx`:

```tsx
import { ClaudeLogo } from "../icons/icons";
import type { Snapshot } from "../lib/types";

export function ProviderTabs({ snapshots, active }: { snapshots: Snapshot[]; active: string }) {
  return (
    <div className="tabs">
      {snapshots.map((s) => {
        const max = Math.max(0, ...s.windows.map((w) => w.usedPct));
        return (
          <span key={s.provider} className={`tab${s.provider === active ? " on" : ""}`}>
            <ClaudeLogo size={13} />
            {s.windows.length ? `${Math.round(max)}%` : "—"}
          </span>
        );
      })}
    </div>
  );
}
```

`src/components/UsageBar.tsx`:

```tsx
import { Icon } from "../icons/icons";
import { formatDuration } from "../lib/format";
import { severity } from "../lib/severity";
import type { UsageWindow } from "../lib/types";

export function UsageBar({ w, now }: { w: UsageWindow; now: number }) {
  const sev = severity(w.usedPct, w.pace);
  const resetIn = w.resetsAt ? formatDuration(Date.parse(w.resetsAt) - now) : null;
  const limitIn = w.pace?.limitAt ? formatDuration(Date.parse(w.pace.limitAt) - now) : null;
  const projected = w.pace?.projectedPct ?? null;

  let bottomRight = "";
  if (projected !== null) {
    bottomRight = projected > 100 ? (resetIn ? `Redefine em ${resetIn}` : "") : `~${Math.round(projected)}% na redefinição`;
  }

  return (
    <div className="m">
      <div className="mtop">
        <span>{w.label}</span>
        {sev === "critical" && limitIn ? (
          <span className="warn">
            <Icon name="trend" size={13} />
            Limite em {limitIn}
          </span>
        ) : (
          resetIn && <span className="muted light">Redefine em {resetIn}</span>
        )}
      </div>
      <div className="bar">
        <div className={`fill sev-${sev}`} style={{ width: `${w.usedPct}%` }} />
        {projected !== null && <div className="tick" style={{ left: `${Math.min(projected, 100)}%` }} />}
      </div>
      <div className="mbot">
        <span>{Math.round(w.usedPct)}% usado</span>
        <span>{bottomRight}</span>
      </div>
    </div>
  );
}
```

`src/components/ProviderCard.tsx`:

```tsx
import { ClaudeLogo, Icon } from "../icons/icons";
import { formatAgo } from "../lib/format";
import type { Notice, Snapshot } from "../lib/types";
import { UsageBar } from "./UsageBar";

const NOTICE_TEXT: Record<Exclude<Notice, "noCredentials">, string> = {
  tokenExpired: "O login do Claude Code expirou. Abra o Claude Code para renovar.",
  rateLimited: "Não consegui atualizar (limite de pedidos).",
  offline: "Sem conexão com a Anthropic.",
};

export function ProviderCard({ snap, now }: { snap: Snapshot; now: number }) {
  return (
    <section className={`sec${snap.stale ? " stale" : ""}`}>
      <div className="ctitle">
        <span className="l">
          <ClaudeLogo />
          {snap.name}
          {snap.plan && <span className="badge">{snap.plan}</span>}
        </span>
      </div>
      {snap.notice === "noCredentials" ? (
        <p className="muted note">
          Não achei o login do Claude Code neste PC. Rode <code>claude</code> no terminal e faça login — o Pacer pega
          sozinho. Enquanto isso, mostro só a atividade dos logs.
        </p>
      ) : (
        snap.windows.map((w) => <UsageBar key={w.id} w={w} now={now} />)
      )}
      {snap.notice && snap.notice !== "noCredentials" && (
        <div className="warnline">
          <Icon name="alert" size={13} />
          <span>
            {NOTICE_TEXT[snap.notice]}
            {snap.stale && snap.fetchedAt ? ` Último dado: ${formatAgo(snap.fetchedAt, now)}.` : ""}
          </span>
        </div>
      )}
    </section>
  );
}
```

`src/components/ActivityGrid.tsx`:

```tsx
import { useState } from "react";
import { formatDay, formatTokens } from "../lib/format";
import { buildGrid, type Cell } from "../lib/grid";
import type { DayActivity } from "../lib/types";

const DOW = ["seg", "", "qua", "", "sex", "", "dom"];

export function ActivityGrid({ days }: { days: DayActivity[] }) {
  const weeks = buildGrid(days);
  const [hover, setHover] = useState<Cell | null>(null);
  const total = days.reduce((sum, d) => sum + d.tokens, 0);
  const today = days[days.length - 1];

  return (
    <section className="sec">
      <div className="ctitle">
        <span>Atividade</span>
        <span className="muted light small">30 dias</span>
      </div>
      <div className="grid" style={{ gridTemplateColumns: `20px repeat(${weeks.length}, 1fr)` }}>
        {DOW.map((d, i) => (
          <b key={`dow-${i}`} style={{ gridColumn: 1, gridRow: i + 1 }}>
            {d}
          </b>
        ))}
        {weeks.flatMap((week, wi) =>
          week.map((c, di) => (
            <i
              key={`${wi}-${di}`}
              className={c ? `lv${c.level}` : "empty"}
              style={{ gridColumn: wi + 2, gridRow: di + 1, animationDelay: `${0.3 + wi * 0.06 + di * 0.02}s` }}
              onMouseEnter={() => c && setHover(c)}
              onMouseLeave={() => setHover(null)}
            />
          )),
        )}
      </div>
      <div className="hmeta">
        <span>
          {hover
            ? `${formatDay(hover.date)} · ${formatTokens(hover.tokens)} tokens · ${hover.messages} msgs`
            : `Hoje: ${formatTokens(today?.tokens ?? 0)} tokens`}
        </span>
        <span>30 dias: {formatTokens(total)}</span>
      </div>
    </section>
  );
}
```

`src/components/Footer.tsx`:

```tsx
import { formatAgo, formatDuration } from "../lib/format";
import type { Snapshot } from "../lib/types";

export function Footer({ snap, now }: { snap: Snapshot | undefined; now: number }) {
  if (!snap) return null;
  const left = snap.fetchedAt ? `Atualizado ${formatAgo(snap.fetchedAt, now)}` : "Só dados locais";
  const next = snap.nextRefreshAt ? formatDuration(Date.parse(snap.nextRefreshAt) - now) : null;
  const right = next ? (snap.stale ? `Tento de novo em ${next}` : `Próxima em ${next}`) : "";
  return (
    <div className="foot">
      <span>{left}</span>
      <span>{right}</span>
    </div>
  );
}
```

`src/components/MainView.tsx`:

```tsx
import { invoke } from "@tauri-apps/api/core";
import { Icon } from "../icons/icons";
import type { AppState } from "../lib/types";
import { ActivityGrid } from "./ActivityGrid";
import { Footer } from "./Footer";
import { Header } from "./Header";
import { ProviderCard } from "./ProviderCard";
import { ProviderTabs } from "./ProviderTabs";

export function MainView({ state, now, onSettings }: { state: AppState; now: number; onSettings: () => void }) {
  const snap = state.snapshots[0];
  return (
    <>
      <Header
        title="Pacer"
        sub="Uso dos planos de IA"
        draggable={!state.config.lockPosition}
        actions={
          <>
            <button className="ib" title="Atualizar agora" onClick={() => invoke("refresh_now")}>
              <Icon name="refresh" />
            </button>
            <button className="ib" title="Configurações" onClick={onSettings}>
              <Icon name="gear" />
            </button>
          </>
        }
      />
      {snap ? (
        <>
          <ProviderTabs snapshots={state.snapshots} active={snap.provider} />
          <ProviderCard snap={snap} now={now} />
          <ActivityGrid days={snap.activity} />
        </>
      ) : (
        <p className="muted note">Nenhum provedor ativo. Ative um nas configurações.</p>
      )}
      <Footer snap={snap} now={now} />
    </>
  );
}
```

- [ ] **Step 5: `src/App.tsx`** (substituir inteiro; a linha da SettingsView fica comentada até a Task 14):

```tsx
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { MainView } from "./components/MainView";
import { useAutoHeight, useNow, usePacer } from "./lib/hooks";

type View = "main" | "settings";

export default function App() {
  const state = usePacer();
  const now = useNow();
  const [view, setView] = useState<View>("main");
  const ref = useRef<HTMLDivElement>(null);
  useAutoHeight(ref);

  useEffect(() => {
    const off = listen("open-settings", () => setView("settings"));
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setView((v) => {
        if (v === "settings") return "main";
        invoke("hide_window");
        return v;
      });
    };
    window.addEventListener("keydown", onKey);
    return () => {
      off.then((f) => f());
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <div className="wg" ref={ref}>
      {!state ? (
        <p className="muted note">Carregando…</p>
      ) : view === "main" ? (
        <MainView state={state} now={now} onSettings={() => setView("settings")} />
      ) : (
        <p className="muted note">Configurações (Task 14)</p>
      )}
    </div>
  );
}
```

- [ ] **Step 6: `src/styles.css`** (substituir inteiro):

```css
:root {
  color-scheme: dark;
  --accent: #1f6feb;
  --accent-2: #58a6ff;
  --text: #e6edf3;
  --muted: #8b949e;
  --dim: #6e7681;
  --line: rgba(255, 255, 255, 0.06);
  --warn: #d29922;
  --crit: #f85149;
  --ok: #3fb950;
}
* { box-sizing: border-box; }
html, body { margin: 0; background: transparent; overflow: hidden; }
body {
  font-family: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;
  font-size: 12px;
  color: var(--text);
  user-select: none;
  -webkit-font-smoothing: antialiased;
}
button { font: inherit; color: inherit; background: none; border: 0; padding: 0; cursor: pointer; }
.wg { width: 100%; padding: 14px; background: rgba(24, 28, 34, 0.82); }

svg.i { stroke: currentColor; fill: none; stroke-width: 1.75; stroke-linecap: round; stroke-linejoin: round; flex: none; }
.logo { display: inline-grid; color: #d97757; flex: none; }
.logo svg { width: 100%; height: 100%; fill: currentColor; }
.muted { color: var(--muted); }
.light { font-weight: 400; }
.small { font-size: 11px; }
.dim { opacity: 0.6; }

.top { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; }
.top h4 { margin: 0; font-size: 15px; font-weight: 600; color: #fff; flex: 1; pointer-events: none; }
.top .sub { display: block; font-size: 11px; color: var(--muted); font-weight: 400; margin-top: 1px; }
.ib { width: 26px; height: 26px; border-radius: 6px; display: grid; place-items: center; color: #c9d1d9; transition: background 0.15s; }
.ib:hover { background: rgba(255, 255, 255, 0.07); }
.ib:focus-visible, .tg:focus-visible, .segc button:focus-visible { outline: 2px solid var(--accent-2); outline-offset: 2px; }
.ver { font-size: 10px; color: var(--muted); padding: 1px 6px; border: 1px solid #30363d; border-radius: 99px; }

.tabs { display: flex; gap: 4px; margin-bottom: 10px; }
.tab { display: flex; align-items: center; gap: 6px; padding: 5px 9px; border-radius: 6px; color: var(--muted); font-weight: 600; }
.tab.on { background: rgba(255, 255, 255, 0.06); color: #fff; box-shadow: inset 0 -2px 0 var(--accent); }

.sec {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 2px 12px;
  margin-bottom: 8px;
  transition: border-color 0.2s, background 0.2s, opacity 0.3s;
  animation: rise 0.4s both;
}
.sec:hover { border-color: rgba(255, 255, 255, 0.14); background: rgba(255, 255, 255, 0.045); }
.sec:nth-of-type(2) { animation-delay: 0.05s; }
.sec:nth-of-type(3) { animation-delay: 0.1s; }
.sec:nth-of-type(4) { animation-delay: 0.15s; }
.sec.stale { opacity: 0.7; }
@keyframes rise { from { opacity: 0; transform: translateY(6px); } to { opacity: 1; transform: none; } }

.ctitle { display: flex; justify-content: space-between; align-items: center; padding: 10px 0 2px; font-weight: 600; font-size: 13px; }
.ctitle .l { display: flex; align-items: center; gap: 7px; }
.badge { font-size: 10px; padding: 1px 6px; border-radius: 4px; background: rgba(255, 255, 255, 0.07); color: #c9d1d9; font-weight: 500; }

.m { padding: 8px 0; }
.mtop { display: flex; justify-content: space-between; align-items: center; font-weight: 600; }
.bar { position: relative; height: 6px; border-radius: 99px; background: rgba(255, 255, 255, 0.08); margin: 6px 0 4px; }
.fill {
  position: absolute; inset: 0 auto 0 0; border-radius: 99px;
  transition: width 0.6s cubic-bezier(0.22, 1, 0.36, 1), background-color 0.3s;
  animation: grow 0.9s cubic-bezier(0.22, 1, 0.36, 1) 0.15s both;
}
.fill.sev-ok { background: var(--accent); }
.fill.sev-warn { background: var(--warn); }
.fill.sev-critical { background: var(--crit); }
@keyframes grow { from { width: 0; } }
.tick { position: absolute; top: -3px; width: 2px; height: 12px; margin-left: -1px; background: var(--text); border-radius: 1px; opacity: 0.85; animation: fade 0.3s 1s both; }
@keyframes fade { from { opacity: 0; } }
.mbot { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); }
.warn { display: flex; align-items: center; gap: 4px; color: var(--crit); font-weight: 600; }
.warnline { display: flex; gap: 6px; align-items: flex-start; color: #f0883e; font-size: 11px; padding: 4px 0 10px; }
.warnline svg { margin-top: 1px; }
.note { margin: 6px 0 10px; line-height: 1.45; }
.note code { color: var(--text); background: rgba(255, 255, 255, 0.07); padding: 0 4px; border-radius: 3px; }

.grid { display: grid; grid-template-rows: repeat(7, 18px); gap: 4px; padding: 6px 0 2px; }
.grid b { font-size: 9px; color: var(--dim); font-weight: 400; line-height: 18px; }
.grid i { display: block; border-radius: 3px; animation: pop 0.35s both; }
.grid i.empty { visibility: hidden; }
.grid i:not(.empty):hover { outline: 1px solid rgba(255, 255, 255, 0.5); }
.lv0 { background: #1c2128; }
.lv1 { background: #0d2a52; }
.lv2 { background: #0f4a9e; }
.lv3 { background: #1f6feb; }
.lv4 { background: #58a6ff; }
@keyframes pop { from { opacity: 0; transform: scale(0.6); } }
.hmeta { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); padding: 6px 0 10px; }
.foot { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); padding-top: 2px; }
.foot a { color: var(--muted); text-decoration: none; }
.foot a:hover { color: var(--accent-2); }

/* configurações */
.sh { display: flex; align-items: center; gap: 8px; padding: 10px 0 4px; font-weight: 600; }
.sh svg { color: var(--accent-2); }
.row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 9px 0; border-top: 1px solid var(--line); }
.sh + .row { border-top: 0; }
.lbl b { display: block; font-weight: 500; }
.lbl span { font-size: 11px; color: var(--muted); }
.grow { flex: 1; }
.tg { flex: none; width: 34px; height: 18px; border-radius: 99px; border: 1px solid var(--dim); position: relative; transition: background 0.2s, border-color 0.2s; }
.tg::after {
  content: ""; position: absolute; left: 3px; top: 3px; width: 10px; height: 10px; border-radius: 50%; background: #c9d1d9;
  transition: transform 0.32s cubic-bezier(0.34, 1.56, 0.64, 1), width 0.15s;
}
.tg:active::after { width: 13px; }
.tg.on { background: var(--accent); border-color: var(--accent); }
.tg.on::after { transform: translateX(16px); background: #fff; }
.segc { position: relative; display: flex; background: rgba(255, 255, 255, 0.05); border: 1px solid var(--line); border-radius: 7px; padding: 2px; }
.segc button { position: relative; z-index: 1; padding: 3px 10px; color: var(--muted); font-weight: 600; font-size: 11px; transition: color 0.2s; }
.segc button.on { color: #fff; }
.pillbg { position: absolute; top: 2px; bottom: 2px; border-radius: 5px; background: var(--accent); transition: left 0.32s cubic-bezier(0.34, 1.56, 0.64, 1), width 0.32s; }
.thr { padding: 18px 0 6px; }
.tbar { position: relative; height: 6px; border-radius: 99px; background: linear-gradient(90deg, #1f6feb 0%, #388bfd 55%, #d29922 78%, #f85149 100%); }
.h {
  position: absolute; top: 50%; width: 14px; height: 14px; margin: -7px 0 0 -7px; border-radius: 50%;
  background: #fff; border: 2px solid #0d1117; cursor: grab; box-shadow: 0 1px 4px rgba(0, 0, 0, 0.6); touch-action: none;
}
.h:active { cursor: grabbing; }
.h em { position: absolute; bottom: 16px; left: 50%; transform: translateX(-50%); font-style: normal; font-size: 10px; font-weight: 600; white-space: nowrap; }
.tcap { display: flex; justify-content: space-between; font-size: 10px; color: var(--dim); margin-top: 8px; }
.toast { display: flex; gap: 8px; align-items: flex-start; margin-top: 10px; background: #1c2128; border: 1px solid #30363d; border-radius: 6px; padding: 7px 9px; font-size: 11px; }
.toast svg { color: var(--warn); margin-top: 1px; }
.prov { display: flex; align-items: center; gap: 10px; padding: 9px 0; }
.plogo { width: 28px; height: 28px; border-radius: 7px; display: grid; place-items: center; background: rgba(217, 119, 87, 0.14); }
.pulse { width: 7px; height: 7px; border-radius: 50%; background: var(--ok); display: inline-block; margin-right: 5px; vertical-align: 1px; animation: pl 2s infinite; }
@keyframes pl {
  0% { box-shadow: 0 0 0 0 rgba(63, 185, 80, 0.55); }
  70% { box-shadow: 0 0 0 6px rgba(63, 185, 80, 0); }
  100% { box-shadow: 0 0 0 0 rgba(63, 185, 80, 0); }
}
.ghost { display: flex; align-items: center; justify-content: center; gap: 6px; border: 1px dashed #30363d; border-radius: 6px; padding: 7px; color: var(--dim); margin: 2px 0 10px; }

@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after { animation-duration: 0.001ms !important; animation-delay: 0s !important; transition-duration: 0.001ms !important; }
}
```

- [ ] **Step 7: Verificar**

Run: `bun run build && bun run test`
Expected: build sem erros de TypeScript; testes passam.

Run: `bun run tauri dev` e comparar com o mockup v3 (`.superpowers/brainstorm/*/content/visual-v3.html`):
- cabeçalho "Pacer" com atualizar/engrenagem (ícones de traço, sem emoji);
- aba com logo do Claude e a %; card com plano "Pro" e barras animando ao abrir, marca branca de projeção;
- grade 7×5/6 em azul, passando o mouse mostra "12 set · 4,2M tokens · 87 msgs";
- rodapé "Atualizado há X · Próxima em Y";
- arrastar pelo cabeçalho move a janela; ela não entra sob a taskbar; Esc esconde.

- [ ] **Step 8: Commit**

```bash
git add src
git commit -m "feat: tela principal do Pacer no visual v3

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 14: Tela de configurações

**Files:**
- Create: `src/components/settings/Row.tsx`, `Toggle.tsx`, `Segmented.tsx`, `ThresholdBar.tsx`, `SettingsView.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `save_config(config) -> Config`, `Icon`, `ClaudeLogo`, `Header`, tipos.
- Produces: `SettingsView({ state, onBack })`.

- [ ] **Step 1: Controles**

`src/components/settings/Row.tsx`:

```tsx
import type { ReactNode } from "react";

export function Row({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="row">
      <div className="lbl">
        <b>{label}</b>
        {hint && <span>{hint}</span>}
      </div>
      {children}
    </div>
  );
}
```

`src/components/settings/Toggle.tsx`:

```tsx
export function Toggle({ on, onChange, label }: { on: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button type="button" role="switch" aria-checked={on} aria-label={label} className={`tg${on ? " on" : ""}`} onClick={() => onChange(!on)} />
  );
}
```

`src/components/settings/Segmented.tsx`:

```tsx
import { useLayoutEffect, useRef, useState } from "react";

interface Props {
  value: number;
  options: readonly number[];
  format: (v: number) => string;
  onChange: (v: number) => void;
}

export function Segmented({ value, options, format, onChange }: Props) {
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const [pill, setPill] = useState({ left: 0, width: 0 });

  useLayoutEffect(() => {
    const el = refs.current[options.indexOf(value)];
    if (el) setPill({ left: el.offsetLeft, width: el.offsetWidth });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value]);

  return (
    <div className="segc">
      <i className="pillbg" style={pill} />
      {options.map((o, i) => (
        <button
          key={o}
          type="button"
          ref={(el) => {
            refs.current[i] = el;
          }}
          className={o === value ? "on" : ""}
          onClick={() => onChange(o)}
        >
          {format(o)}
        </button>
      ))}
    </div>
  );
}
```

`src/components/settings/ThresholdBar.tsx`:

```tsx
import { type PointerEvent as ReactPointerEvent, useEffect, useRef, useState } from "react";
import { Icon } from "../../icons/icons";

export function ThresholdBar({ values, onChange }: { values: number[]; onChange: (v: number[]) => void }) {
  const [local, setLocal] = useState(values);
  const bar = useRef<HTMLDivElement>(null);
  useEffect(() => setLocal(values), [values]);

  const startDrag = (i: number) => (e: ReactPointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    const el = e.currentTarget;
    el.setPointerCapture(e.pointerId);
    let current = local;
    const move = (ev: PointerEvent) => {
      const r = bar.current!.getBoundingClientRect();
      const v = Math.round(Math.min(100, Math.max(1, ((ev.clientX - r.left) / r.width) * 100)));
      current = current.map((x, j) => (j === i ? v : x));
      setLocal(current);
    };
    const up = () => {
      el.removeEventListener("pointermove", move);
      el.removeEventListener("pointerup", up);
      onChange([...current].sort((a, b) => a - b));
    };
    el.addEventListener("pointermove", move);
    el.addEventListener("pointerup", up);
  };

  const first = local.length ? Math.min(...local) : 80;
  return (
    <div className="thr">
      <div className="tbar" ref={bar}>
        {local.map((v, i) => (
          <div key={i} className="h" style={{ left: `${v}%` }} onPointerDown={startDrag(i)}>
            <em>{v}%</em>
          </div>
        ))}
      </div>
      <div className="tcap">
        <span>0%</span>
        <span>aviso · perto do limite</span>
        <span>100%</span>
      </div>
      <div className="toast">
        <Icon name="alert" />
        <span>
          Sua sessão passou de <b>{first}</b>% — agora em {Math.min(100, first + 1)}%
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: `src/components/settings/SettingsView.tsx`**:

```tsx
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ClaudeLogo, Icon } from "../../icons/icons";
import type { AppState, Config } from "../../lib/types";
import { Header } from "../Header";
import { Row } from "./Row";
import { Segmented } from "./Segmented";
import { ThresholdBar } from "./ThresholdBar";
import { Toggle } from "./Toggle";

const REPO_URL = "https://github.com/sthevan027/Claude-Glass";
const REFRESH_OPTIONS = [1, 5, 10] as const;

export function SettingsView({ state, onBack }: { state: AppState; onBack: () => void }) {
  const cfg = state.config;
  const save = (patch: Partial<Config>) => invoke<Config>("save_config", { config: { ...cfg, ...patch } });
  const claude = state.snapshots.find((s) => s.provider === "claude");
  const connected = !!claude && claude.notice !== "noCredentials" && claude.notice !== "tokenExpired";

  return (
    <>
      <Header
        title="Configurações"
        draggable={!cfg.lockPosition}
        leading={
          <button className="ib" title="Voltar" onClick={onBack}>
            <Icon name="back" />
          </button>
        }
        actions={<span className="ver">v{state.version}</span>}
      />

      <section className="sec">
        <div className="sh">
          <Icon name="sliders" />
          Geral
        </div>
        <Row label="Iniciar com o Windows" hint="Abre sozinho ao ligar o PC">
          <Toggle label="Iniciar com o Windows" on={cfg.startWithWindows} onChange={(v) => save({ startWithWindows: v })} />
        </Row>
        <Row label="Travar acima da barra" hint="Fixa no canto, sem arrastar">
          <Toggle label="Travar acima da barra" on={cfg.lockPosition} onChange={(v) => save({ lockPosition: v })} />
        </Row>
        <Row label="Atualizar a cada">
          <Segmented value={cfg.refreshMinutes} options={REFRESH_OPTIONS} format={(v) => `${v}m`} onChange={(v) => save({ refreshMinutes: v })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="bell" />
          Alertas
        </div>
        <ThresholdBar values={cfg.alerts.thresholds} onChange={(t) => save({ alerts: { ...cfg.alerts, thresholds: t } })} />
        <Row label="Notificações" hint="Avisos do Windows ao passar dos limites">
          <Toggle label="Notificações" on={cfg.alerts.enabled} onChange={(v) => save({ alerts: { ...cfg.alerts, enabled: v } })} />
        </Row>
        <Row label="Avisar se a previsão estourar" hint="Antes de chegar no limite">
          <Toggle label="Avisar se a previsão estourar" on={cfg.alerts.pace} onChange={(v) => save({ alerts: { ...cfg.alerts, pace: v } })} />
        </Row>
      </section>

      <section className="sec">
        <div className="sh">
          <Icon name="plug" />
          Provedores
        </div>
        <div className="prov">
          <div className="plogo">
            <ClaudeLogo />
          </div>
          <div className="lbl grow">
            <b>
              Claude {claude?.plan && <span className="muted">· {claude.plan}</span>}
            </b>
            <span>
              {connected ? (
                <>
                  <i className="pulse" />
                  Conectado via Claude Code
                </>
              ) : (
                "Sem login do Claude Code"
              )}
            </span>
          </div>
          <Toggle label="Claude" on={cfg.providers.claude.enabled} onChange={(v) => save({ providers: { claude: { enabled: v } } })} />
        </div>
        <div className="ghost">
          <Icon name="plus" size={13} />
          Adicionar provedor <span className="dim">(em breve)</span>
        </div>
      </section>

      <div className="foot">
        <span>Pacer v{state.version}</span>
        <a
          href={REPO_URL}
          onClick={(e) => {
            e.preventDefault();
            openUrl(REPO_URL);
          }}
        >
          GitHub
        </a>
      </div>
    </>
  );
}
```

- [ ] **Step 3: Ligar no `App.tsx`** — adicionar o import e trocar o placeholder:

```tsx
import { SettingsView } from "./components/settings/SettingsView";
```

```tsx
      ) : (
        <SettingsView state={state} onBack={() => setView("main")} />
      )}
```

- [ ] **Step 4: Verificar**

Run: `bun run build && bun run test`
Expected: sem erros.

Run: `bun run tauri dev` e conferir:
- engrenagem abre as configurações; "‹" e Esc voltam;
- interruptores com efeito de mola; seletor 1m/5m/10m com pílula deslizando;
- arrastar os pontos da barra azul→vermelho muda os números e a prévia; ao soltar, `%APPDATA%\dev.sthevan.pacer\config.json` reflete os novos limiares;
- "Travar acima da barra" ligado: janela volta ao canto e o cabeçalho não arrasta mais;
- bolinha verde pulsando no card do Claude;
- "GitHub" abre o navegador; menu da bandeja → "Configurações" abre esta tela.

- [ ] **Step 5: Commit**

```bash
git add src
git commit -m "feat: tela de configuracoes com cartoes, termometro de alertas e provedores

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 15: Ícone, README, build assinado e release

**Files:**
- Create: `app-icon.svg`, `README.md`, `README.en.md`, `docs/screenshots/pacer-main.png`, `docs/screenshots/pacer-settings.png`
- Modify: `src-tauri/icons/*` (gerados), `src-tauri/tauri.conf.json` (assinatura)

- [ ] **Step 1: Ícone do app** — `app-icon.svg`:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <rect width="1024" height="1024" rx="224" fill="#181c22"/>
  <circle cx="512" cy="512" r="300" fill="none" stroke="#2d333b" stroke-width="88"/>
  <path d="M512 212 A300 300 0 1 1 252 662" fill="none" stroke="#1f6feb" stroke-width="88" stroke-linecap="round"/>
  <rect x="496" y="150" width="32" height="150" rx="16" fill="#e6edf3"/>
</svg>
```

Run: `bunx tauri icon app-icon.svg`
Expected: `src-tauri/icons/` regenerado (32x32.png, 128x128.png, 128x128@2x.png, icon.ico, …).

- [ ] **Step 2: Prints reais** — com `bun run tauri dev` aberto, capturar com Win+Shift+S a tela principal e as configurações e salvar como `docs/screenshots/pacer-main.png` e `docs/screenshots/pacer-settings.png`.

- [ ] **Step 3: `README.md`**:

```markdown
# Pacer

**Português** · [English](README.en.md)

Widget para Windows que mostra quanto do seu plano de IA você já usou — e
**para onde o ritmo atual te leva** antes da redefinição.

![Pacer](docs/screenshots/pacer-main.png)

## O que mostra

- **Sessão (5h) e semanal** do Claude, com a % real vinda da Anthropic
- **Previsão do ritmo**: a marca branca na barra mostra onde você vai estar
  na redefinição; se for estourar, aparece "Limite em 1d 7h"
- **Atividade dos últimos 30 dias** em grade estilo GitHub, a partir dos logs
  locais do Claude Code
- **Alertas** do Windows ao passar de cada limiar (padrão 80% e 95%) e quando
  a previsão indicar que o limite acaba antes da redefinição
- Ícone na bandeja que muda de cor conforme o uso

![Configurações](docs/screenshots/pacer-settings.png)

## Requisitos

- Windows 10/11
- [Claude Code](https://claude.com/claude-code) instalado e logado — é de onde
  vêm o login e os logs

## Instalação

Baixe o instalador em [Releases](https://github.com/sthevan027/Claude-Glass/releases)
e execute. O executável é assinado com um certificado interno (Virex); o
Windows pode mostrar "Windows protegeu o computador" → **Mais informações →
Executar assim mesmo**.

### Rodar do código

Requer Rust, Bun e o Visual Studio Build Tools (C++).

```bash
bun install
bun run tauri dev     # desenvolvimento
bun run tauri build   # instalador em src-tauri/target/release/bundle/nsis/
```

## Privacidade

- O Pacer **lê** `~/.claude/.credentials.json` (o login do Claude Code) e
  **nunca escreve nele nem renova o token** — quem renova é o próprio Claude Code.
- O token nunca chega na interface; fica só no processo Rust.
- Só conversa com `api.anthropic.com`. Sem telemetria.
- Configuração em `%APPDATA%\dev.sthevan.pacer\config.json`.

## Créditos

Estilo inspirado no [ai-usagebar](https://github.com/akitaonrails/ai-usagebar)
de Fabio Akita (MIT), de onde vem também o símbolo do Claude usado no app.

## Licença

MIT
```

`README.en.md`: mesma estrutura em inglês (traduzir cada seção; manter links e caminhos).

- [ ] **Step 4: Assinatura** — conferir se o certificado Virex existe neste PC (pode ter sumido na formatação de 02/10/2026):

Run (PowerShell): `Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -like "*Virex*" | Select-Object Subject, Thumbprint, NotAfter, HasPrivateKey`

- Se aparecer `9318068D26A501B00CF870EBF8A129DEFF3C0873` com `HasPrivateKey = True`: adicionar em `tauri.conf.json` → `bundle`:

```json
    "windows": {
      "certificateThumbprint": "9318068D26A501B00CF870EBF8A129DEFF3C0873",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
```

- Se **não** aparecer: parar e avisar o Sthevan — o `.pfx` precisa ser reimportado do pen drive (ver `D:\meu2cerebro\3-Recursos\Certificados internos Virex.md`). Não gerar certificado novo sem pedir.

- [ ] **Step 5: Build e verificação**

Run: `cargo test --manifest-path src-tauri/Cargo.toml && bun run test && bun run tauri build`
Expected: testes passam; instalador em `src-tauri/target/release/bundle/nsis/Pacer_2.0.0_x64-setup.exe`.

Run (PowerShell, se assinado): `Get-AuthenticodeSignature src-tauri\target\release\bundle\nsis\Pacer_2.0.0_x64-setup.exe, src-tauri\target\release\pacer.exe | Select-Object Path, Status, SignerCertificate`
Expected: `SignerCertificate` com `CN=Virex` nos dois. Se a assinatura automática falhar (histórico do Claude Glass), assinar manualmente com `signtool sign /sha1 9318068D26A501B00CF870EBF8A129DEFF3C0873 /fd sha256 /tr http://timestamp.digicert.com /td sha256 <arquivo>`.

Instalar o setup, abrir o Pacer instalado, conferir tela, bandeja, "Iniciar com o Windows" (Gerenciador de Tarefas → Inicializar) e consumo de RAM no Gerenciador de Tarefas (anotar o valor).

- [ ] **Step 6: Commit**

```bash
git add app-icon.svg src-tauri/icons src-tauri/tauri.conf.json README.md README.en.md docs/screenshots
git commit -m "docs: README pt/en, icone do app e build assinado do Pacer 2.0.0

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 7: Publicação (só com confirmação explícita do Sthevan)**

Perguntar antes de cada ação externa:
1. `git push -u origin pacer` e abrir PR `pacer → main`.
2. Depois do merge: `gh release create v2.0.0 <setup.exe> --title "Pacer 2.0.0" --notes "<resumo>"`.
3. Renomear o repositório: `gh repo rename pacer` (o GitHub redireciona o link antigo); depois atualizar `REPO_URL` em `SettingsView.tsx` e os links dos READMEs.
