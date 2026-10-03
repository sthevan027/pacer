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
