use crate::config::AlertsConfig;
use crate::snapshot::Snapshot;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Projeção (% na redefinição) abaixo da qual o alerta de previsão volta a poder disparar.
const PACE_REARM_BELOW: f64 = 95.0;

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
                    // Histerese: entre buscas o uso fica parado e o relógio corre, então o limite
                    // projetado "foge" para depois da redefinição sem o ritmo ter melhorado. Só
                    // rearma quando a projeção cai bem abaixo de 100% (ou some, ex.: janela nova).
                    None => {
                        let calm = w.pace.as_ref().is_none_or(|p| p.projected_pct < PACE_REARM_BELOW);
                        if calm {
                            self.armed.remove(&key);
                        }
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

    #[test]
    fn pace_alert_does_not_refire_while_projection_stays_high() {
        let mut st = AlertState::default();
        let mut c = cfg();
        c.pace = true;
        let soon = Pace { projected_pct: 130.0, limit_at: Some(now() + Duration::minutes(90)) };
        assert_eq!(st.evaluate(&snap(50.0, Some(soon.clone())), &c, now()).len(), 1);
        // entre buscas o uso fica parado e o tempo corre: o limite projetado passa da redefinição
        let drifted = Pace { projected_pct: 104.0, limit_at: Some(now() + Duration::hours(4)) };
        assert!(st.evaluate(&snap(50.0, Some(drifted)), &c, now()).is_empty());
        // nova busca volta a estourar antes da redefinição — mesma situação, não avisa de novo
        assert!(st.evaluate(&snap(52.0, Some(soon)), &c, now()).is_empty());
    }

    #[test]
    fn pace_alert_rearms_after_projection_drops_well_below_limit() {
        let mut st = AlertState::default();
        let mut c = cfg();
        c.pace = true;
        let soon = Pace { projected_pct: 130.0, limit_at: Some(now() + Duration::minutes(90)) };
        assert_eq!(st.evaluate(&snap(50.0, Some(soon.clone())), &c, now()).len(), 1);
        let calm = Pace { projected_pct: 80.0, limit_at: None };
        assert!(st.evaluate(&snap(40.0, Some(calm)), &c, now()).is_empty());
        assert_eq!(st.evaluate(&snap(50.0, Some(soon)), &c, now()).len(), 1);
    }
}
