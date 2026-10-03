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
