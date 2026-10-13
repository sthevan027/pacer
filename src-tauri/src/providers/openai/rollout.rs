//! Leitor dos rollouts do Codex (`<CODEX_HOME>/sessions/AAAA/MM/DD/rollout-*.jsonl`).
//!
//! Só leitura de arquivo local: o Pacer não toca em credencial nem em rede. Cada evento
//! `token_count` carrega `rate_limits` (janelas `primary`/`secondary`); ficamos com o último.
//! Formato conferido em `openai/codex` (`codex-rs/protocol`). Ver
//! `docs/superpowers/specs/2026-10-12-openai-codex-pesquisa.md`.

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct LimitWindow {
    pub used_pct: f64,
    pub minutes: i64,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RateLimits {
    pub plan: Option<String>,
    pub windows: Vec<LimitWindow>,
    /// Hora do evento no rollout (não a da leitura).
    pub at: DateTime<Utc>,
}

pub fn codex_dir() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".codex"))
}

/// Quantos rollouts (do mais recente para trás) vale olhar até achar um `token_count`.
const MAX_FILES: usize = 5;
/// Só o fim do arquivo interessa: o `token_count` mais novo é um dos últimos eventos.
const TAIL_BYTES: u64 = 512 * 1024;
pub(super) const MAX_WINDOW_MINUTES: i64 = 365 * 24 * 60;

fn collect_rollouts(dir: &Path, out: &mut Vec<(SystemTime, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        let Ok(meta) = e.metadata() else { continue };
        if meta.is_dir() {
            collect_rollouts(&path, out);
        } else if path.extension().is_some_and(|x| x == "jsonl")
            && path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("rollout-"))
        {
            if let Ok(m) = meta.modified() {
                out.push((m, path));
            }
        }
    }
}

fn read_tail(path: &Path) -> Option<String> {
    let mut f = fs::File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let start = len.saturating_sub(TAIL_BYTES);
    f.seek(SeekFrom::Start(start)).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    // começou no meio do arquivo: a primeira linha provavelmente está cortada
    Some(if start > 0 { text.split_once('\n').map_or(String::new(), |(_, rest)| rest.to_string()) } else { text })
}

fn window_of(v: &Value) -> Option<LimitWindow> {
    let used = v.get("used_percent")?.as_f64()?;
    if !used.is_finite() {
        return None;
    }
    // Duração absurda (arquivo corrompido) vira 0 = "Limite" sem projeção; um ano já é folga demais.
    let minutes = v.get("window_minutes").and_then(Value::as_i64).filter(|m| (1..=MAX_WINDOW_MINUTES).contains(m)).unwrap_or(0);
    Some(LimitWindow {
        used_pct: used.clamp(0.0, 100.0),
        minutes,
        resets_at: v.get("resets_at").and_then(Value::as_i64).and_then(|s| Utc.timestamp_opt(s, 0).single()),
    })
}

/// Uma linha do rollout → `RateLimits`, se for um `token_count` com limites.
fn parse_line(line: &str) -> Option<RateLimits> {
    // a maioria das linhas é conversa grande: nem parseia o que não é token_count
    if !line.contains("token_count") {
        return None;
    }
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("type")?.as_str()? != "event_msg" {
        return None;
    }
    let p = v.get("payload")?;
    if p.get("type")?.as_str()? != "token_count" {
        return None;
    }
    let rl = p.get("rate_limits")?;
    if !rl.is_object() {
        return None;
    }
    let at = DateTime::parse_from_rfc3339(v.get("timestamp")?.as_str()?).ok()?.with_timezone(&Utc);
    let windows: Vec<LimitWindow> = ["primary", "secondary"].iter().filter_map(|k| rl.get(k).and_then(window_of)).collect();
    if windows.is_empty() {
        return None;
    }
    Some(RateLimits { plan: rl.get("plan_type").and_then(Value::as_str).map(str::to_string), windows, at })
}

/// Último `rate_limits` conhecido dentro de `<codex_dir>/sessions`, ou `None` se não houver nada legível.
/// Entre os rollouts mais recentes vale o evento mais novo, não o arquivo mexido por último: com
/// duas sessões abertas, uma pode ter gravado outra coisa depois sem ter um `token_count` mais novo.
pub fn latest(codex_dir: &Path) -> Option<RateLimits> {
    let mut files = Vec::new();
    collect_rollouts(&codex_dir.join("sessions"), &mut files);
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files
        .iter()
        .take(MAX_FILES)
        .filter_map(|(_, path)| read_tail(path)?.lines().rev().find_map(parse_line))
        .fold(None, |best: Option<RateLimits>, r| match best {
            Some(b) if b.at >= r.at => Some(b),
            _ => Some(r),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/codex_rollout.jsonl");

    fn day_dir(root: &Path) -> PathBuf {
        let d = root.join("sessions").join("2026").join("10").join("12");
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn takes_the_last_token_count_with_rate_limits() {
        let d = tempfile::tempdir().unwrap();
        fs::write(day_dir(d.path()).join("rollout-2026-10-12T14-00-00-aaa.jsonl"), FIXTURE).unwrap();
        let r = latest(d.path()).unwrap();
        assert_eq!(r.at, Utc.with_ymd_and_hms(2026, 10, 12, 14, 5, 0).unwrap());
        assert_eq!(r.plan.as_deref(), Some("plus"));
        assert_eq!(r.windows.len(), 2);
        assert_eq!(r.windows[0], LimitWindow {
            used_pct: 42.5,
            minutes: 300,
            resets_at: Some(Utc.timestamp_opt(1_790_010_000, 0).unwrap()),
        });
        assert_eq!((r.windows[1].used_pct, r.windows[1].minutes), (18.0, 10080));
    }

    #[test]
    fn no_sessions_folder_gives_none() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(latest(d.path()), None);
    }

    #[test]
    fn rollout_without_rate_limits_gives_none() {
        let d = tempfile::tempdir().unwrap();
        let only = r#"{"timestamp":"2026-10-12T14:00:00.000Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":null}}"#;
        fs::write(day_dir(d.path()).join("rollout-x.jsonl"), only).unwrap();
        assert_eq!(latest(d.path()), None);
    }

    #[test]
    fn newest_rollout_wins_and_older_one_is_the_fallback() {
        let d = tempfile::tempdir().unwrap();
        let dir = day_dir(d.path());
        fs::write(dir.join("rollout-old.jsonl"), FIXTURE).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        // o rollout mais novo ainda não tem nenhum token_count: cai para o anterior
        fs::write(dir.join("rollout-new.jsonl"), r#"{"timestamp":"2026-10-12T15:00:00.000Z","type":"session_meta","payload":{}}"#).unwrap();
        assert_eq!(latest(d.path()).unwrap().windows[0].used_pct, 42.5);

        std::thread::sleep(std::time::Duration::from_millis(30));
        let newer = FIXTURE.replace("42.5", "77.0");
        fs::write(dir.join("rollout-newer.jsonl"), newer).unwrap();
        assert_eq!(latest(d.path()).unwrap().windows[0].used_pct, 77.0);
    }

    #[test]
    fn out_of_range_percentages_are_clamped_and_bad_windows_dropped() {
        let d = tempfile::tempdir().unwrap();
        let line = r#"{"timestamp":"2026-10-12T14:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":140.0,"window_minutes":300},"secondary":{"window_minutes":10080}}}}"#;
        fs::write(day_dir(d.path()).join("rollout-x.jsonl"), line).unwrap();
        let r = latest(d.path()).unwrap();
        assert_eq!(r.windows.len(), 1, "janela sem used_percent não entra");
        assert_eq!(r.windows[0].used_pct, 100.0);
        assert_eq!(r.windows[0].resets_at, None);
    }

    #[test]
    fn absurd_window_length_is_dropped_to_zero_minutes() {
        let d = tempfile::tempdir().unwrap();
        let line = r#"{"timestamp":"2026-10-12T14:00:00.000Z","type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":10.0,"window_minutes":9000000000000000000}}}}"#;
        fs::write(day_dir(d.path()).join("rollout-x.jsonl"), line).unwrap();
        assert_eq!(latest(d.path()).unwrap().windows[0].minutes, 0);
    }

    #[test]
    fn the_most_recent_event_wins_even_if_its_file_was_touched_earlier() {
        let d = tempfile::tempdir().unwrap();
        let dir = day_dir(d.path());
        // sessão A tem o evento mais novo (14:05), mas o arquivo da B foi mexido depois com um evento mais velho
        fs::write(dir.join("rollout-a.jsonl"), FIXTURE).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        let older = FIXTURE.replace("2026-10-12T14:05:00.000Z", "2026-10-12T13:00:00.000Z").replace("42.5", "11.0");
        fs::write(dir.join("rollout-b.jsonl"), older).unwrap();
        assert_eq!(latest(d.path()).unwrap().windows[0].used_pct, 42.5);
    }

    #[test]
    fn big_rollout_is_read_from_the_tail_only() {
        let d = tempfile::tempdir().unwrap();
        let filler = format!(r#"{{"timestamp":"2026-10-12T13:00:00.000Z","type":"response_item","payload":{{"text":"{}"}}}}"#, "x".repeat(200));
        let mut body = String::new();
        for _ in 0..4000 {
            body.push_str(&filler);
            body.push('\n');
        }
        body.push_str(FIXTURE);
        fs::write(day_dir(d.path()).join("rollout-big.jsonl"), body).unwrap();
        assert_eq!(latest(d.path()).unwrap().windows[0].used_pct, 42.5);
    }
}
