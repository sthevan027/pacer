//! Limites do Codex pelo próprio Codex: `codex app-server` + o RPC `account/rateLimits/read`.
//! Quem fala com a OpenAI (e cuida do login) é o `codex`; o Pacer não lê token nenhum.
//! Protocolo conferido no `alethe-agents` (`codex_usage.rs`) e testado contra o `codex.exe`
//! da extensão do VS Code.

use super::rollout::{LimitWindow, RateLimits, MAX_WINDOW_MINUTES};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(12);
const REQUESTS: &str = concat!(
    r#"{"id":1,"method":"initialize","params":{"clientInfo":{"name":"pacer","version":"2"}}}"#,
    "\n",
    r#"{"method":"initialized"}"#,
    "\n",
    r#"{"id":2,"method":"account/rateLimits/read"}"#,
    "\n",
);

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Limits(RateLimits),
    /// O Codex está instalado mas sem conta logada.
    NotSignedIn,
    Unavailable,
}

fn window(v: Option<&Value>) -> Option<LimitWindow> {
    let v = v.filter(|v| v.is_object())?;
    let used = v.get("usedPercent")?.as_f64().filter(|u| u.is_finite())?;
    Some(LimitWindow {
        used_pct: used.clamp(0.0, 100.0),
        minutes: v.get("windowDurationMins").and_then(Value::as_i64).filter(|m| (1..=MAX_WINDOW_MINUTES).contains(m)).unwrap_or(0),
        resets_at: v.get("resetsAt").and_then(Value::as_i64).and_then(|s| Utc.timestamp_opt(s, 0).single()),
    })
}

/// Uma linha da saída do app-server. `None` se não é a resposta do pedido 2 (notificações, eco).
pub fn parse_response(line: &str, now: DateTime<Utc>) -> Option<Outcome> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("id").and_then(Value::as_i64) != Some(2) {
        return None;
    }
    if let Some(err) = v.get("error") {
        let msg = err.get("message").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
        return Some(if msg.contains("authentication required") { Outcome::NotSignedIn } else { Outcome::Unavailable });
    }
    let rl = v.get("result").and_then(|r| r.get("rateLimits")).filter(|r| r.is_object());
    let Some(rl) = rl else { return Some(Outcome::Unavailable) };
    let windows: Vec<LimitWindow> = ["primary", "secondary"].iter().filter_map(|k| window(rl.get(k))).collect();
    if windows.is_empty() {
        return Some(Outcome::Unavailable);
    }
    Some(Outcome::Limits(RateLimits { plan: rl.get("planType").and_then(Value::as_str).map(str::to_string), windows, at: now }))
}

/// `codex.exe` no PATH, ou o embutido na extensão do VS Code (a instalação mais comum no Windows).
pub fn find_codex() -> Option<PathBuf> {
    let on_path = std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).map(|d| d.join(if cfg!(windows) { "codex.exe" } else { "codex" })).find(|p| p.is_file())
    });
    on_path.or_else(|| {
        let ext = dirs::home_dir()?.join(".vscode").join("extensions");
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(ext)
            .ok()?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("openai.chatgpt-")))
            .collect();
        dirs.sort();
        dirs.iter().rev().find_map(|d| {
            std::fs::read_dir(d.join("bin")).ok()?.flatten().map(|e| e.path().join(if cfg!(windows) { "codex.exe" } else { "codex" })).find(|p| p.is_file())
        })
    })
}

#[cfg(windows)]
fn no_window(cmd: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000)
}

#[cfg(not(windows))]
fn no_window(cmd: &mut Command) -> &mut Command {
    cmd
}

/// Bloqueante (processo filho, até 12 s): chamar fora da thread do laço assíncrono.
pub fn read_limits(now: DateTime<Utc>) -> Outcome {
    let Some(exe) = find_codex() else { return Outcome::Unavailable };
    let mut cmd = Command::new(exe);
    cmd.arg("app-server").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    let Ok(mut child) = no_window(&mut cmd).spawn() else { return Outcome::Unavailable };
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        let _ = child.kill();
        return Outcome::Unavailable;
    };
    if stdin.write_all(REQUESTS.as_bytes()).and_then(|_| stdin.flush()).is_err() {
        let _ = child.kill();
        return Outcome::Unavailable;
    }
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(outcome) = parse_response(&line, now) {
                let _ = tx.send(outcome);
                break;
            }
        }
    });
    let outcome = rx.recv_timeout(TIMEOUT).unwrap_or(Outcome::Unavailable);
    let _ = child.kill();
    let _ = child.wait();
    drop(stdin);
    let _ = reader.join();
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 12, 15, 0, 0).unwrap()
    }

    #[test]
    fn parses_the_rate_limits_response() {
        let line = r#"{"id":2,"result":{"rateLimits":{"primary":{"usedPercent":42.5,"windowDurationMins":300,"resetsAt":1790010000},
            "secondary":{"usedPercent":18.0,"windowDurationMins":10080,"resetsAt":1790600000},"planType":"plus","rateLimitReachedType":null}}}"#
            .replace('\n', "");
        let Outcome::Limits(l) = parse_response(&line, now()).unwrap() else { panic!("esperava limites") };
        assert_eq!(l.plan.as_deref(), Some("plus"));
        assert_eq!(l.at, now());
        assert_eq!(l.windows.len(), 2);
        assert_eq!((l.windows[0].used_pct, l.windows[0].minutes), (42.5, 300));
        assert_eq!(l.windows[0].resets_at, Some(Utc.timestamp_opt(1_790_010_000, 0).unwrap()));
        assert_eq!((l.windows[1].used_pct, l.windows[1].minutes), (18.0, 10080));
    }

    #[test]
    fn ignores_lines_that_are_not_the_answer_to_request_2() {
        assert_eq!(parse_response(r#"{"id":1,"result":{"userAgent":"x"}}"#, now()), None);
        assert_eq!(parse_response(r#"{"method":"remoteControl/status/changed","params":{}}"#, now()), None);
        assert_eq!(parse_response("não é json", now()), None);
    }

    #[test]
    fn missing_login_is_told_apart_from_other_failures() {
        let not_signed = r#"{"error":{"code":-32600,"message":"codex account authentication required to read rate limits"},"id":2}"#;
        assert_eq!(parse_response(not_signed, now()), Some(Outcome::NotSignedIn));
        assert_eq!(parse_response(r#"{"id":2,"error":{"code":-1,"message":"boom"}}"#, now()), Some(Outcome::Unavailable));
    }

    #[test]
    fn empty_or_odd_results_are_unavailable_and_numbers_are_sanitized() {
        assert_eq!(parse_response(r#"{"id":2,"result":{"rateLimits":null}}"#, now()), Some(Outcome::Unavailable));
        assert_eq!(parse_response(r#"{"id":2,"result":{}}"#, now()), Some(Outcome::Unavailable));
        let odd = r#"{"id":2,"result":{"rateLimits":{"primary":{"usedPercent":150.0,"windowDurationMins":9000000000000000000}}}}"#;
        let Outcome::Limits(l) = parse_response(odd, now()).unwrap() else { panic!() };
        assert_eq!((l.windows[0].used_pct, l.windows[0].minutes, l.windows[0].resets_at), (100.0, 0, None));
    }
}
