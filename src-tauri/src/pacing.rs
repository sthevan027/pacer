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
