//! Leitor dos logs do Claude Code (`~/.claude/projects/**/*.jsonl`).
//!
//! Cada linha de resposta do assistente traz modelo, sessão, projeto (`cwd`) e a divisão
//! entrada/saída/cache — este módulo deixa de descartar isso e entrega [`UsageEvent`] prontos.
//! A agregação (widget e painel) mora em `crate::usage`.

use crate::usage::{UsageEvent, DAYS};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const PROVIDER_ID: &str = "claude";

struct CachedFile {
    modified: SystemTime,
    len: u64,
    events: Vec<UsageEvent>,
}

#[derive(Default)]
pub struct LogScanner {
    cache: HashMap<PathBuf, CachedFile>,
    last_parsed: usize,
}

/// Última pasta do `cwd` é o nome que a pessoa reconhece (`D:\Projetos\pacer` → `pacer`).
fn project_of(cwd: Option<&str>, fallback: &str) -> String {
    cwd.filter(|c| !c.is_empty())
        .and_then(|c| Path::new(c).file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| fallback.to_string())
}

/// `projects/D--Projetos-pacer/<sessão>.jsonl` → `D--Projetos-pacer`, usado quando a linha não
/// tem `cwd` (não deveria acontecer, mas o log é arquivo de terceiro).
fn fallback_project(path: &Path, projects_dir: &Path) -> String {
    path.strip_prefix(projects_dir)
        .ok()
        .and_then(|rest| rest.components().next())
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_else(|| "sem projeto".into())
}

fn parse_line(line: &str, fallback_project: &str) -> Option<UsageEvent> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("type")?.as_str()? != "assistant" {
        return None;
    }
    let msg = v.get("message")?;
    let usage = msg.get("usage")?;
    let model = msg.get("model").and_then(Value::as_str)?;
    // Respostas sintéticas (erro local, corte) não são consumo.
    if model == "<synthetic>" {
        return None;
    }
    let ts = v.get("timestamp")?.as_str()?.parse::<DateTime<Utc>>().ok()?;
    let n = |k: &str| usage.get(k).and_then(Value::as_u64).unwrap_or(0);

    // O log separa a escrita de cache em 5 min e 1 h, e os preços diferem. Log antigo (ou
    // provedor que não detalha) cai tudo na faixa de 5 min.
    let (cache_write_5m, cache_write_1h) = match usage.get("cache_creation") {
        Some(c) => (
            c.get("ephemeral_5m_input_tokens").and_then(Value::as_u64).unwrap_or(0),
            c.get("ephemeral_1h_input_tokens").and_then(Value::as_u64).unwrap_or(0),
        ),
        None => (n("cache_creation_input_tokens"), 0),
    };

    Some(UsageEvent {
        ts,
        provider: PROVIDER_ID,
        model: model.to_string(),
        session: v.get("sessionId").and_then(Value::as_str).unwrap_or_default().to_string(),
        project: project_of(v.get("cwd").and_then(Value::as_str), fallback_project),
        input: n("input_tokens"),
        output: n("output_tokens"),
        cache_read: n("cache_read_input_tokens"),
        cache_write_5m,
        cache_write_1h,
        sidechain: v.get("isSidechain").and_then(Value::as_bool).unwrap_or(false),
        // Id da mensagem + id do pedido identifica a resposta. NÃO usar (sessão, timestamp,
        // modelo): duas respostas diferentes no mesmo segundo e no mesmo modelo virariam uma só.
        key: msg.get("id").and_then(Value::as_str).map(|id| {
            let req = v.get("requestId").and_then(Value::as_str).unwrap_or("");
            format!("{id}:{req}")
        }),
    })
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

    /// Eventos dos últimos 30 dias, no fuso de `now`, já deduplicados.
    /// Só relê arquivos cujo mtime/tamanho mudou desde a última varredura.
    pub fn scan<Tz: TimeZone>(&mut self, projects_dir: &Path, now: DateTime<Tz>) -> Vec<UsageEvent> {
        self.last_parsed = 0;
        let tz = now.timezone();
        let today: NaiveDate = now.date_naive();
        let first = today - Duration::days(DAYS - 1);
        let cutoff: SystemTime = (now.with_timezone(&Utc) - Duration::days(DAYS + 1)).into();

        let mut files = Vec::new();
        walk(projects_dir, cutoff, &mut files);
        let live: HashSet<&PathBuf> = files.iter().map(|(p, _)| p).collect();
        self.cache.retain(|p, _| live.contains(p));

        let mut seen: HashSet<String> = HashSet::new();
        let mut out = Vec::new();

        for (path, md) in &files {
            let modified = md.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let len = md.len();
            let changed = self.cache.get(path).is_none_or(|c| c.modified != modified || c.len != len);
            if changed {
                let fallback = fallback_project(path, projects_dir);
                let events = fs::read_to_string(path)
                    .map(|raw| raw.lines().filter_map(|l| parse_line(l, &fallback)).collect())
                    .unwrap_or_default();
                self.cache.insert(path.clone(), CachedFile { modified, len, events });
                self.last_parsed += 1;
            }
            for e in &self.cache[path].events {
                let date = e.ts.with_timezone(&tz).date_naive();
                if date < first || date > today {
                    continue;
                }
                // A mesma resposta pode aparecer em mais de um arquivo (retomada de sessão).
                if let Some(k) = &e.key {
                    if !seen.insert(k.clone()) {
                        continue;
                    }
                }
                out.push(e.clone());
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use std::io::Write;

    fn brt() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    // "agora" = 10/10/2026 15:00 em UTC−3
    fn now() -> DateTime<FixedOffset> {
        brt().with_ymd_and_hms(2026, 10, 10, 15, 0, 0).unwrap()
    }

    /// Linha completa, no formato que o Claude Code grava de verdade.
    fn line(ts: &str, id: &str, req: &str, model: &str, cwd: &str, input: u64, output: u64) -> String {
        format!(
            r#"{{"type":"assistant","timestamp":"{ts}","sessionId":"sess-1","cwd":"{cwd}","isSidechain":false,"requestId":"{req}","message":{{"id":"{id}","model":"{model}","usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
        )
    }

    /// `cwd` como o Claude Code grava no Windows: barras invertidas escapadas dentro do JSON.
    /// Num literal normal (`"...\\..."`) isso viraria uma barra só e o JSON ficaria inválido.
    const CWD_JSON: &str = r"D:\\Projetos\\pacer";

    fn simple(ts: &str, id: &str, tokens: u64) -> String {
        line(ts, id, "r", "claude-opus-5-5", CWD_JSON, tokens, 0)
    }

    #[test]
    fn missing_dir_gives_no_events() {
        let d = tempfile::tempdir().unwrap();
        let evs = LogScanner::new().scan(&d.path().join("nao-existe"), now());
        assert!(evs.is_empty());
    }

    #[test]
    fn extracts_model_session_project_and_usage_parts() {
        let d = tempfile::tempdir().unwrap();
        let proj = d.path().join("D--Projetos-pacer");
        fs::create_dir_all(&proj).unwrap();
        let body = r#"{"type":"assistant","timestamp":"2026-10-10T12:00:00Z","sessionId":"0a13f081-5d59","cwd":"D:\\Projetos\\pacer","isSidechain":true,"requestId":"req_1","message":{"id":"msg_1","model":"claude-sonnet-5-5","usage":{"input_tokens":2,"output_tokens":26,"cache_read_input_tokens":171107,"cache_creation_input_tokens":3186,"cache_creation":{"ephemeral_5m_input_tokens":3186,"ephemeral_1h_input_tokens":0}}}}"#;
        fs::write(proj.join("s.jsonl"), body).unwrap();

        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 1);
        let e = &evs[0];
        assert_eq!(e.provider, "claude");
        assert_eq!(e.model, "claude-sonnet-5-5");
        assert_eq!(e.session, "0a13f081-5d59");
        assert_eq!(e.project, "pacer");
        assert_eq!(e.input, 2);
        assert_eq!(e.output, 26);
        assert_eq!(e.cache_read, 171107);
        assert_eq!(e.cache_write_5m, 3186);
        assert_eq!(e.cache_write_1h, 0);
        assert!(e.sidechain);
        // e.tokens() é o total que o widget sempre usou
        assert_eq!(e.tokens(), 2 + 26 + 171107 + 3186);
    }

    #[test]
    fn splits_cache_write_into_5m_and_1h() {
        let d = tempfile::tempdir().unwrap();
        let body = r#"{"type":"assistant","timestamp":"2026-10-10T12:00:00Z","sessionId":"s","cwd":"C:\\x","message":{"id":"m","model":"claude-opus-5-5","usage":{"input_tokens":0,"output_tokens":0,"cache_creation_input_tokens":1000,"cache_creation":{"ephemeral_5m_input_tokens":400,"ephemeral_1h_input_tokens":600}}}}"#;
        fs::write(d.path().join("s.jsonl"), body).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs[0].cache_write_5m, 400);
        assert_eq!(evs[0].cache_write_1h, 600);
        assert_eq!(evs[0].cache_write(), 1000);
    }

    #[test]
    fn without_the_cache_breakdown_everything_goes_to_the_5m_bucket() {
        let d = tempfile::tempdir().unwrap();
        let body = r#"{"type":"assistant","timestamp":"2026-10-10T12:00:00Z","sessionId":"s","cwd":"C:\\x","message":{"id":"m","model":"claude-opus-5-5","usage":{"input_tokens":1,"cache_creation_input_tokens":700}}}"#;
        fs::write(d.path().join("s.jsonl"), body).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs[0].cache_write_5m, 700);
        assert_eq!(evs[0].cache_write_1h, 0);
    }

    #[test]
    fn skips_noise() {
        let d = tempfile::tempdir().unwrap();
        let body = [
            simple("2026-10-10T12:00:00Z", "m1", 100),
            simple("2026-10-10T13:00:00Z", "m2", 50),
            r#"{"type":"user","timestamp":"2026-10-10T13:00:00Z","message":{"content":"oi"}}"#.to_string(),
            r#"{"type":"assistant","timestamp":"2026-10-10T13:00:00Z","message":{"id":"s","model":"<synthetic>","usage":{"input_tokens":999}}}"#.to_string(),
            "{ linha quebrada".to_string(),
            String::new(),
        ]
        .join("\n");
        fs::write(d.path().join("s.jsonl"), body).unwrap();
        fs::write(d.path().join("ignorado.txt"), simple("2026-10-10T12:00:00Z", "x", 7)).unwrap();
        fs::write(d.path().join("sem-modelo.jsonl"), r#"{"type":"assistant","timestamp":"2026-10-10T12:00:00Z","message":{"usage":{"input_tokens":5}}}"#).unwrap();

        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 2);
        assert_eq!(evs.iter().map(|e| e.input).sum::<u64>(), 150);
    }

    #[test]
    fn falls_back_to_the_project_folder_when_cwd_is_missing() {
        let d = tempfile::tempdir().unwrap();
        let proj = d.path().join("D--Projetos-pacer");
        fs::create_dir_all(&proj).unwrap();
        let body = r#"{"type":"assistant","timestamp":"2026-10-10T12:00:00Z","sessionId":"s","message":{"id":"m","model":"claude-opus-5-5","usage":{"input_tokens":1}}}"#;
        fs::write(proj.join("s.jsonl"), body).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs[0].project, "D--Projetos-pacer");
    }

    #[test]
    fn deduplicates_the_same_response_across_files() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.jsonl"), simple("2026-10-09T12:00:00Z", "m1", 100)).unwrap();
        fs::write(d.path().join("b.jsonl"), simple("2026-10-09T12:00:00Z", "m1", 100)).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].input, 100);
    }

    #[test]
    fn keeps_two_distinct_responses_from_the_same_second_and_model() {
        // Regressão: deduplicar por (sessão, timestamp, modelo) fundia duas requisições
        // diferentes disparadas no mesmo segundo pelo mesmo modelo. A identidade é o id da
        // mensagem, não o instante.
        let d = tempfile::tempdir().unwrap();
        let body = [
            simple("2026-10-10T12:00:00Z", "msg_a", 10),
            simple("2026-10-10T12:00:00Z", "msg_b", 20),
        ]
        .join("\n");
        fs::write(d.path().join("a.jsonl"), body).unwrap();

        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 2);
        assert_eq!(evs.iter().map(|e| e.input).sum::<u64>(), 30);
    }

    #[test]
    fn ignores_entries_outside_30_days() {
        let d = tempfile::tempdir().unwrap();
        let body = [
            simple("2026-09-03T15:00:00Z", "old", 500),  // 37 dias atrás
            simple("2026-09-11T15:00:00Z", "first", 5),  // no limite
        ]
        .join("\n");
        fs::write(d.path().join("a.jsonl"), body).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].input, 5);
    }

    #[test]
    fn groups_by_local_day_not_utc_day() {
        // 02:30 UTC do dia 10 = 23:30 do dia 9 em UTC−3: entra, mas não como "hoje"
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.jsonl"), simple("2026-10-10T02:30:00Z", "m1", 40)).unwrap();
        let evs = LogScanner::new().scan(d.path(), now());
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].ts.with_timezone(&brt()).date_naive(), NaiveDate::from_ymd_opt(2026, 10, 9).unwrap());
    }

    #[test]
    fn rescans_only_changed_files_and_forgets_deleted() {
        let d = tempfile::tempdir().unwrap();
        let a_path = d.path().join("a.jsonl");
        let b_path = d.path().join("b.jsonl");
        fs::write(&a_path, simple("2026-10-10T12:00:00Z", "m1", 10)).unwrap();
        fs::write(&b_path, simple("2026-10-10T12:00:00Z", "m2", 20)).unwrap();
        let mut s = LogScanner::new();
        assert_eq!(s.scan(d.path(), now()).len(), 2);
        assert_eq!(s.last_parsed(), 2);

        assert_eq!(s.scan(d.path(), now()).len(), 2);
        assert_eq!(s.last_parsed(), 0);

        let mut f = fs::OpenOptions::new().append(true).open(&a_path).unwrap();
        writeln!(f).unwrap();
        writeln!(f, "{}", simple("2026-10-10T13:00:00Z", "m3", 5)).unwrap();
        drop(f);
        assert_eq!(s.scan(d.path(), now()).len(), 3);
        assert_eq!(s.last_parsed(), 1);

        fs::remove_file(&b_path).unwrap();
        let evs = s.scan(d.path(), now());
        assert_eq!(evs.len(), 2);
        assert_eq!(evs.iter().map(|e| e.input).sum::<u64>(), 15);
    }

    #[test]
    fn sees_appends_while_the_file_handle_is_still_open() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.jsonl");
        // o Claude Code mantém o arquivo da sessão aberto enquanto escreve
        let mut f = fs::OpenOptions::new().create(true).append(true).open(&p).unwrap();
        writeln!(f, "{}", simple("2026-10-10T12:00:00Z", "m1", 10)).unwrap();
        f.flush().unwrap();
        let mut s = LogScanner::new();
        assert_eq!(s.scan(d.path(), now()).len(), 1);

        writeln!(f, "{}", simple("2026-10-10T13:00:00Z", "m2", 5)).unwrap();
        f.flush().unwrap();
        assert_eq!(s.scan(d.path(), now()).len(), 2);
    }
}
