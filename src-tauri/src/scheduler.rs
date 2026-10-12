use crate::alerts::{self, AlertState};
use crate::config::Config;
use crate::providers::claude::{creds, ClaudeProvider};
use crate::providers::openai::{rollout, OpenAiProvider};
use crate::providers::{FetchResult, Provider};
use crate::snapshot::Snapshot;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchPlan {
    /// Ainda não é hora e ninguém pediu.
    Wait,
    /// Pode buscar agora.
    Now,
    /// Pediram/venceu, mas a última busca foi há menos de 60 s: busca quando o intervalo mínimo acabar.
    Defer(DateTime<Utc>),
}

/// Decide se o provedor busca agora, depois ou nada. Um pedido forçado dentro do
/// intervalo mínimo é adiado (não descartado).
pub fn plan_fetch(forced: bool, now: DateTime<Utc>, next_remote: DateTime<Utc>, last_remote: Option<DateTime<Utc>>) -> FetchPlan {
    if !(forced || now >= next_remote) {
        return FetchPlan::Wait;
    }
    match last_remote {
        Some(last) if now - last < MIN_GAP => FetchPlan::Defer(last + MIN_GAP),
        _ => FetchPlan::Now,
    }
}

struct Slot {
    provider: Box<dyn Provider>,
    backoff: Backoff,
    next_remote: DateTime<Utc>,
    last_remote: Option<DateTime<Utc>>,
}

/// Uma volta do laço para um provedor. Desligado não é tocado (nem leitura local, nem API) e
/// devolve `None`. Ligado: sempre atualiza a parte local e busca na API só quando o plano manda.
async fn poll_slot(s: &mut Slot, enabled: bool, forced: bool, now: DateTime<Utc>) -> Option<Snapshot> {
    if !enabled {
        return None;
    }
    s.provider.tick_local(now);
    match plan_fetch(forced, now, s.next_remote, s.last_remote) {
        FetchPlan::Now => {
            let r = s.provider.fetch_remote(now).await;
            s.backoff.record(r);
            s.last_remote = Some(now);
            s.next_remote = now + chrono::Duration::from_std(s.backoff.delay()).unwrap();
        }
        FetchPlan::Defer(until) => s.next_remote = s.next_remote.min(until),
        FetchPlan::Wait => {}
    }
    let mut snap = s.provider.snapshot();
    snap.next_refresh_at = Some(s.next_remote);
    Some(snap)
}

/// Uma volta completa: consulta a lista inteira e devolve só os snapshots dos provedores ligados.
async fn poll_all(slots: &mut [Slot], cfg: &Config, forced: bool, now: DateTime<Utc>) -> Vec<Snapshot> {
    let mut snaps = Vec::new();
    for s in slots.iter_mut() {
        let enabled = cfg.provider_enabled(s.provider.id());
        if let Some(snap) = poll_slot(s, enabled, forced, now).await {
            snaps.push(snap);
        }
    }
    snaps
}

pub fn spawn(app: AppHandle, shared: Arc<Shared>) {
    tauri::async_runtime::spawn(async move {
        let mut refresh = shared.config.lock().unwrap().refresh_minutes;
        let new_slot = |provider: Box<dyn Provider>| Slot {
            provider,
            backoff: Backoff::new(refresh),
            next_remote: Utc::now(),
            last_remote: None,
        };
        let mut slots = vec![
            new_slot(Box::new(ClaudeProvider::new(creds::claude_dir(), shared.usage.clone()))),
            new_slot(Box::new(OpenAiProvider::new(rollout::codex_dir()))),
        ];
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

            let snaps = poll_all(&mut slots, &cfg, forced, now).await;
            for snap in &snaps {
                for a in alert_state.evaluate(snap, &cfg.alerts, now) {
                    alerts::notify(&app, &a);
                }
            }

            crate::tray::update(&app, &snaps, &cfg.accent_color);
            *shared.snapshots.lock().unwrap() = snaps.clone();
            let _ = app.emit("snapshot", &snaps);

            tokio::select! {
                _ = tokio::time::sleep(LOCAL_TICK) => {}
                _ = shared.wake.notified() => {}
            }
        }
    });
}

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

    fn t(secs: i64) -> DateTime<Utc> {
        chrono::TimeZone::timestamp_opt(&Utc, 1_790_000_000 + secs, 0).unwrap()
    }

    #[test]
    fn waits_when_not_due_and_not_forced() {
        assert_eq!(plan_fetch(false, t(100), t(300), Some(t(0))), FetchPlan::Wait);
    }

    #[test]
    fn fetches_when_due() {
        assert_eq!(plan_fetch(false, t(300), t(300), Some(t(0))), FetchPlan::Now);
        assert_eq!(plan_fetch(false, t(0), t(0), None), FetchPlan::Now);
    }

    #[test]
    fn forced_after_the_minimum_gap_fetches_now() {
        assert_eq!(plan_fetch(true, t(90), t(300), Some(t(0))), FetchPlan::Now);
    }

    #[test]
    fn forced_inside_the_minimum_gap_is_deferred_not_dropped() {
        // buscou em t=0, usuário clica "Atualizar agora" em t=30: tem que buscar em t=60, não em t=300
        assert_eq!(plan_fetch(true, t(30), t(300), Some(t(0))), FetchPlan::Defer(t(60)));
    }

    #[derive(Default)]
    struct Calls {
        ticks: u32,
        fetches: u32,
    }

    struct Fake {
        id: &'static str,
        calls: Arc<std::sync::Mutex<Calls>>,
    }

    #[async_trait::async_trait]
    impl Provider for Fake {
        fn id(&self) -> &'static str {
            self.id
        }
        fn tick_local(&mut self, _now: DateTime<Utc>) {
            self.calls.lock().unwrap().ticks += 1;
        }
        async fn fetch_remote(&mut self, _now: DateTime<Utc>) -> FetchResult {
            self.calls.lock().unwrap().fetches += 1;
            FetchResult::Ok
        }
        fn snapshot(&self) -> crate::snapshot::Snapshot {
            crate::snapshot::Snapshot {
                provider: "fake".into(),
                name: "Fake".into(),
                plan: None,
                windows: vec![],
                activity: vec![],
                today_tokens: 0,
                fetched_at: None,
                next_refresh_at: None,
                stale: false,
                notice: None,
            }
        }
    }

    fn fake_slot(id: &'static str, calls: &Arc<std::sync::Mutex<Calls>>, next_remote: DateTime<Utc>) -> Slot {
        Slot {
            provider: Box::new(Fake { id, calls: calls.clone() }),
            backoff: Backoff::new(5),
            next_remote,
            last_remote: None,
        }
    }

    #[tokio::test]
    async fn disabled_provider_is_never_touched() {
        let calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        let mut s = fake_slot("claude", &calls, t(0));
        assert!(poll_slot(&mut s, false, true, t(0)).await.is_none());
        let c = calls.lock().unwrap();
        assert_eq!((c.ticks, c.fetches), (0, 0));
    }

    #[tokio::test]
    async fn enabled_provider_fetches_when_due() {
        let calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        let mut s = fake_slot("claude", &calls, t(0));
        assert!(poll_slot(&mut s, true, false, t(0)).await.is_some());
        let c = calls.lock().unwrap();
        assert_eq!((c.ticks, c.fetches), (1, 1));
    }

    #[tokio::test]
    async fn enabled_provider_only_ticks_locally_when_not_due() {
        let calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        let mut s = fake_slot("claude", &calls, t(300));
        assert!(poll_slot(&mut s, true, false, t(100)).await.is_some());
        let c = calls.lock().unwrap();
        assert_eq!((c.ticks, c.fetches), (1, 0));
    }

    #[tokio::test]
    async fn poll_all_only_returns_and_touches_enabled_providers() {
        // Config padrão: claude ligado, openai desligado. Os dois estão na lista do scheduler.
        let cfg = Config::default();
        let claude_calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        let openai_calls = Arc::new(std::sync::Mutex::new(Calls::default()));
        let mut slots = vec![
            fake_slot("claude", &claude_calls, t(0)),
            fake_slot("openai", &openai_calls, t(0)),
        ];
        let snaps = poll_all(&mut slots, &cfg, false, t(0)).await;
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].provider, "fake");
        assert_eq!(claude_calls.lock().unwrap().fetches, 1);
        let o = openai_calls.lock().unwrap();
        assert_eq!((o.ticks, o.fetches), (0, 0));
    }
}
