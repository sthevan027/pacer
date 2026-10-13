pub mod app_server;
pub mod rollout;

use crate::pacing;
use crate::providers::{FetchResult, Provider};
use crate::snapshot::{Notice, Snapshot, UsageWindow};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use app_server::Outcome;
use rollout::{LimitWindow, RateLimits};
use std::path::PathBuf;
use std::sync::Arc;

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

/// Leitura ao vivo dos limites (o `codex app-server`); injetável para os testes não abrirem processo.
pub type LiveSource = Arc<dyn Fn(DateTime<Utc>) -> Outcome + Send + Sync>;

pub struct OpenAiProvider {
    codex_dir: PathBuf,
    live: LiveSource,
    limits: Option<RateLimits>,
    windows: Vec<UsageWindow>,
    notice: Option<Notice>,
}

impl OpenAiProvider {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self::with_live(codex_dir, Arc::new(app_server::read_limits))
    }

    pub fn with_live(codex_dir: PathBuf, live: LiveSource) -> Self {
        Self { codex_dir, live, limits: None, windows: Vec::new(), notice: None }
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

    /// Pergunta ao próprio Codex (`app-server`); se ele não está instalado, não tem login ou não
    /// responde, cai nos rollouts em disco (o último evento de uso). No ritmo do `refreshMinutes`.
    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult {
        let live = self.live.clone();
        let outcome = tauri::async_runtime::spawn_blocking(move || live(now)).await.unwrap_or(Outcome::Unavailable);
        let limits = match outcome {
            Outcome::Limits(l) => Some(l),
            Outcome::NotSignedIn | Outcome::Unavailable => rollout::latest(&self.codex_dir),
        };
        match limits {
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

    /// Provedor de teste: sem o `codex` ao vivo (nunca abre processo), só os rollouts da pasta.
    fn local(dir: &std::path::Path) -> OpenAiProvider {
        OpenAiProvider::with_live(dir.into(), Arc::new(|_| Outcome::Unavailable))
    }

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
        let mut p = local(d.path());
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
        let mut p = local(d.path());
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
        let mut p = local(d.path());
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
        let mut p = local(d.path());
        p.fetch_remote(before_reset()).await;
        assert_eq!(p.snapshot().windows[0].used_pct, 42.5);
        fs::remove_dir_all(d.path().join("sessions")).unwrap();
        p.tick_local(Utc.timestamp_opt(1_790_020_000, 0).unwrap());
        assert_eq!(p.snapshot().windows[0].used_pct, 0.0, "tick_local só recalcula; não relê o disco");
    }

    #[tokio::test]
    async fn data_disappearing_clears_the_card() {
        let d = with_rollout(FIXTURE);
        let mut p = local(d.path());
        p.fetch_remote(before_reset()).await;
        fs::remove_dir_all(d.path().join("sessions")).unwrap();
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        let s = p.snapshot();
        assert!(s.windows.is_empty());
        assert_eq!(s.notice, Some(Notice::NoCredentials));
    }

    fn live_limits(used: f64) -> Outcome {
        Outcome::Limits(RateLimits {
            plan: Some("pro".into()),
            windows: vec![LimitWindow { used_pct: used, minutes: 300, resets_at: Some(Utc.timestamp_opt(1_790_030_000, 0).unwrap()) }],
            at: Utc.timestamp_opt(1_790_009_000, 0).unwrap(),
        })
    }

    #[tokio::test]
    async fn live_codex_answer_wins_over_the_rollouts() {
        let d = with_rollout(FIXTURE);
        let mut p = OpenAiProvider::with_live(d.path().into(), Arc::new(|_| live_limits(7.0)));
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Ok);
        let s = p.snapshot();
        assert_eq!(s.plan.as_deref(), Some("Pro"));
        assert_eq!((s.windows.len(), s.windows[0].used_pct), (1, 7.0));
        assert_eq!(s.fetched_at, Some(before_reset()), "hora da leitura ao vivo, não a de um evento antigo");
    }

    #[tokio::test]
    async fn not_signed_in_or_missing_codex_falls_back_to_the_rollouts() {
        for outcome in [Outcome::NotSignedIn, Outcome::Unavailable] {
            let d = with_rollout(FIXTURE);
            let o = outcome.clone();
            let mut p = OpenAiProvider::with_live(d.path().into(), Arc::new(move |_| o.clone()));
            assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Ok, "{outcome:?}");
            assert_eq!(p.snapshot().windows[0].used_pct, 42.5);
        }
    }

    #[tokio::test]
    async fn no_live_answer_and_no_rollouts_shows_the_notice() {
        let d = tempfile::tempdir().unwrap();
        let mut p = OpenAiProvider::with_live(d.path().into(), Arc::new(|_| Outcome::NotSignedIn));
        assert_eq!(p.fetch_remote(before_reset()).await, FetchResult::Skipped);
        assert_eq!(p.snapshot().notice, Some(Notice::NoCredentials));
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
