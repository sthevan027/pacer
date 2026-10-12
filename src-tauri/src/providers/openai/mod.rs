pub mod rollout;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rollout::{LimitWindow, RateLimits};
use std::path::PathBuf;

pub const PROVIDER_ID: &str = "openai";

/// Rótulo e id pela duração da janela, nunca pela posição (primary/secondary mudam conforme o plano).
fn describe(minutes: i64) -> (String, String) {
    match minutes {
        300 => ("five_hour".into(), "Sessão (5h)".into()),
        10080 => ("seven_day".into(), "Semanal".into()),
        1440 => ("window_1440m".into(), "1 dia".into()),
        m if m > 0 && m % 1440 == 0 => (format!("window_{m}m"), format!("{} dias", m / 1440)),
        m if m > 0 && m % 60 == 0 => (format!("window_{m}m"), format!("{}h", m / 60)),
        m if m > 0 => (format!("window_{m}m"), format!("{m} min")),
        _ => ("limit".into(), "Limite".into()),
    }
}

/// `plus` → `Plus`.
fn plan_name(raw: &str) -> String {
    let mut c = raw.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

/// Janela cuja redefinição já passou recomeçou do zero; o dado do rollout é anterior a isso.
fn to_window(w: &LimitWindow, now: DateTime<Utc>) -> UsageWindow {
    let (id, label) = describe(w.minutes);
    let expired = w.resets_at.is_some_and(|r| r <= now);
    let resets_at = if expired { None } else { w.resets_at };
    let used_pct = if expired { 0.0 } else { w.used_pct };
    let pace = resets_at.and_then(|r| pacing::project(used_pct, Duration::minutes(w.minutes), r, now));
    UsageWindow { id, label, used_pct, resets_at, pace }
}

pub struct OpenAiProvider {
    codex_dir: PathBuf,
    limits: Option<RateLimits>,
    windows: Vec<UsageWindow>,
    notice: Option<Notice>,
}

impl OpenAiProvider {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir, limits: None, windows: Vec::new(), notice: None }
    }

    fn recompute(&mut self, now: DateTime<Utc>) {
        self.windows = self.limits.as_ref().map_or_else(Vec::new, |l| l.windows.iter().map(|w| to_window(w, now)).collect());
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn id(&self) -> &'static str {
        PROVIDER_ID
    }

    fn tick_local(&mut self, now: DateTime<Utc>) {
        self.recompute(now);
    }

    /// Não há API: "remoto" aqui é reler os rollouts do Codex, no ritmo do `refreshMinutes`.
    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        match rollout::latest(&self.codex_dir) {
            Some(l) => {
                self.limits = Some(l);
                self.notice = None;
                self.recompute(now);
                FetchResult::Ok
            }
            None => {
                self.limits = None;
                self.notice = Some(Notice::NoCredentials);
                self.recompute(now);
                FetchResult::Skipped
            }
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            provider: PROVIDER_ID.into(),
            name: "OpenAI".into(),
            plan: self.limits.as_ref().and_then(|l| l.plan.as_deref()).map(plan_name),
            windows: self.windows.clone(),
            // Os rollouts não entram no painel nesta leva: sem atividade de 30 dias para mostrar.
            activity: Vec::new(),
            today_tokens: 0,
            fetched_at: self.limits.as_ref().map(|l| l.at),
            next_refresh_at: None,
            stale: false,
            notice: self.notice,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::fs;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/codex_rollout.jsonl");

    fn with_rollout(body: &str) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let day = d.path().join("sessions").join("2026").join("10").join("12");
        fs::create_dir_all(&day).unwrap();
        fs::write(day.join("rollout-x.jsonl"), body).unwrap();
        d
    }

    /// Antes do `resets_at` da fixture (1_790_010_000 ≈ 2026-09-21), com as duas janelas ainda abertas.
    fn before_reset() -> DateTime<Utc> {
        Utc.timestamp_opt(1_790_009_000, 0).unwrap()
    }

    #[tokio::test]
    async fn without_codex_data_shows_notice_and_skips() {
        let d = tempfile::tempdir().unwrap();
        let mut p = OpenAiProvider::new(d.path().into());
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert_eq!(s.notice, Some(Notice::NoCredentials));
        assert!(s.windows.is_empty());
        assert_eq!(s.fetched_at, None);
        assert_eq!(s.provider, "openai");
    }

    #[tokio::test]
    async fn reads_windows_plan_and_event_time() {
        let d = with_rollout(FIXTURE);
        let mut p = OpenAiProvider::new(d.path().into());
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!(s.plan.as_deref(), Some("Plus"));
        assert_eq!(s.notice, None);
        assert_eq!(s.windows.len(), 2);
        assert_eq!((s.windows[0].id.as_str(), s.windows[0].label.as_str()), ("five_hour", "Sessão (5h)"));
        assert_eq!(s.windows[0].used_pct, 42.5);
        assert_eq!(s.windows[1].label, "Semanal");
        assert_eq!(s.fetched_at, Some(Utc.with_ymd_and_hms(2026, 10, 12, 14, 5, 0).unwrap()));
    }

    #[tokio::test]
    async fn a_window_past_its_reset_counts_as_zero() {
        let d = with_rollout(FIXTURE);
        let mut p = OpenAiProvider::new(d.path().into());
        // depois do reset da janela de 5h, antes do reset da semanal
        p.fetch_remote(Utc.timestamp_opt(1_790_020_000, 0).unwrap()).await;
        let s = p.snapshot();
        assert_eq!(s.windows[0].used_pct, 0.0);
        assert_eq!(s.windows[0].resets_at, None);
        assert_eq!(s.windows[1].used_pct, 18.0);
    }

    #[tokio::test]
    async fn tick_local_expires_windows_without_rereading_files() {
        let d = with_rollout(FIXTURE);
        let mut p = OpenAiProvider::new(d.path().into());
        p.fetch_remote(before_reset()).await;
        assert_eq!(p.snapshot().windows[0].used_pct, 42.5);
        fs::remove_dir_all(d.path().join("sessions")).unwrap();
        p.tick_local(Utc.timestamp_opt(1_790_020_000, 0).unwrap());
        assert_eq!(p.snapshot().windows[0].used_pct, 0.0, "tick_local só recalcula; não relê o disco");
    }

    #[tokio::test]
    async fn data_disappearing_clears_the_card() {
        let d = with_rollout(FIXTURE);
        let mut p = OpenAiProvider::new(d.path().into());
        p.fetch_remote(before_reset()).await;
        fs::remove_dir_all(d.path().join("sessions")).unwrap();
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert!(s.windows.is_empty());
        assert_eq!(s.notice, Some(Notice::NoCredentials));
    }

    #[test]
    fn labels_follow_window_length_not_position() {
        assert_eq!(describe(300), ("five_hour".into(), "Sessão (5h)".into()));
        assert_eq!(describe(10080), ("seven_day".into(), "Semanal".into()));
        assert_eq!(describe(1440).1, "1 dia");
        assert_eq!(describe(2880).1, "2 dias");
        assert_eq!(describe(120).1, "2h");
        assert_eq!(describe(45).1, "45 min");
        assert_eq!(describe(0).1, "Limite");
    }
}
