//! Consumo dos planos de IA, independente de provedor.
//!
//! O Claude é o primeiro produtor de [`UsageEvent`]; a Etapa B (outros provedores) acrescenta
//! produtores novos sem mexer no painel. Este módulo não sabe ler log nenhum — quem lê é
//! `providers::claude::logs`, que entrega eventos prontos pro [`UsageStore`].

pub mod pricing;

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use std::collections::HashMap;

pub const DAYS: i64 = 30;

/// Um consumo pontual: uma resposta do assistente.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageEvent {
    pub ts: DateTime<Utc>,
    pub provider: &'static str,
    /// Id exato do modelo, como vem no log (`claude-sonnet-5-5`, `claude-haiku-4-5-20251001`).
    pub model: String,
    pub session: String,
    /// Nome curto do projeto (última pasta do `cwd`), pro ranking.
    pub project: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    /// O log separa a escrita de cache em 5 min e 1 h, e os preços diferem (1,6×).
    pub cache_write_5m: u64,
    pub cache_write_1h: u64,
    /// Subagente (`isSidechain`). Entra nos totais, como já entrava no widget.
    pub sidechain: bool,
    /// Identidade da resposta (`message.id:requestId`) só para deduplicar: a mesma resposta pode
    /// aparecer em mais de um arquivo quando a sessão é retomada. `None` = sem como identificar.
    pub key: Option<String>,
}

impl UsageEvent {
    pub fn cache_write(&self) -> u64 {
        self.cache_write_5m + self.cache_write_1h
    }

    /// Mesma soma que o widget já usa (entrada + saída + cache lido + cache criado).
    /// **Não mexer:** é o número que a grade de 30 dias e o README mostram hoje. O painel é quem
    /// separa as partes (o total infla com cache).
    pub fn tokens(&self) -> u64 {
        self.input + self.output + self.cache_read + self.cache_write()
    }
}

/// Tier do modelo: é por ele que o painel filtra e colore, como a referência do console.
pub fn family(model: &str) -> &'static str {
    let m = model.to_ascii_lowercase();
    if m.contains("opus") {
        "opus"
    } else if m.contains("sonnet") {
        "sonnet"
    } else if m.contains("haiku") {
        "haiku"
    } else if m.contains("fable") {
        "fable"
    } else if m.contains("mythos") {
        "mythos"
    } else {
        "outro"
    }
}

/// Ordem canônica das famílias nas abas, nos chips e na pilha do gráfico.
const FAMILY_ORDER: [&str; 6] = ["opus", "sonnet", "haiku", "fable", "mythos", "outro"];

fn family_rank(f: &str) -> usize {
    FAMILY_ORDER.iter().position(|x| *x == f).unwrap_or(FAMILY_ORDER.len())
}

/// Soma das quatro partes. O `messages` conta respostas, não tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write_5m: u64,
    pub cache_write_1h: u64,
    pub messages: u32,
}

impl Totals {
    fn add(&mut self, e: &UsageEvent) {
        self.input += e.input;
        self.output += e.output;
        self.cache_read += e.cache_read;
        self.cache_write_5m += e.cache_write_5m;
        self.cache_write_1h += e.cache_write_1h;
        self.messages += 1;
    }

    pub fn cache_write(&self) -> u64 {
        self.cache_write_5m + self.cache_write_1h
    }

    /// Entrada + saída, sem cache: é o que a referência plota como consumo principal.
    pub fn billable_tokens(&self) -> u64 {
        self.input + self.output
    }
}

/// Agregado diário de 30 dias que o widget consome (contrato inalterado desde a v1).
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub days: Vec<crate::snapshot::DayActivity>,
    pub today_tokens: u64,
}

// ---------------------------------------------------------------- relatório

#[derive(Debug, Clone)]
pub struct ReportFilter {
    /// 7, 14 ou 30.
    pub range_days: i64,
    /// Vazio = todas as famílias.
    pub families: Vec<String>,
}

impl Default for ReportFilter {
    fn default() -> Self {
        Self { range_days: DAYS, families: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTotal {
    pub model: String,
    pub family: String,
    pub totals: Totals,
    /// `None` = modelo fora da tabela de preços (não inventamos valor).
    pub cost_brl: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilySlice {
    pub family: String,
    pub totals: Totals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySlice {
    pub date: NaiveDate,
    pub totals: Totals,
    /// Uma fatia por família, já na ordem canônica — é a pilha do gráfico.
    pub by_family: Vec<FamilySlice>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestRow {
    pub ts: DateTime<Utc>,
    pub model: String,
    pub family: String,
    pub session: String,
    pub project: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

/// Linha de ranking (top sessões e top projetos).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupRow {
    pub name: String,
    pub sub: String,
    pub totals: Totals,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageReport {
    pub range_days: i64,
    pub generated_at: DateTime<Utc>,
    /// Famílias presentes no período, **sem** aplicar o filtro — é com isso que a UI monta os chips.
    pub available_families: Vec<String>,
    pub filtered_families: Vec<String>,
    /// Um item por dia do período, do mais antigo pro mais recente (dias vazios entram zerados).
    pub days: Vec<DaySlice>,
    pub by_model: Vec<ModelTotal>,
    pub totals: Totals,
    pub total_cost_brl: f64,
    /// Modelos do período sem preço na tabela: o total em R$ fica subestimado, e a UI avisa.
    pub unpriced_models: Vec<String>,
    pub usd_brl: f64,
    pub top_sessions: Vec<GroupRow>,
    pub top_projects: Vec<GroupRow>,
    /// Mais recentes primeiro.
    pub recent: Vec<RequestRow>,
    /// Quantas requisições passaram pelo filtro (o `recent` é só a cauda).
    pub request_count: usize,
}

/// Quantas requisições a tabela "Registros" recebe de uma vez.
pub const RECENT_LIMIT: usize = 200;
/// Quantas linhas os rankings trazem.
pub const RANK_LIMIT: usize = 10;

// ---------------------------------------------------------------- store

/// Guarda os eventos de todos os provedores.
///
/// Vive no [`crate::state::Shared`] porque precisa ser alcançável dos dois lados: o scheduler
/// alimenta, e o comando do painel lê. Antes disso o scanner morava dentro do `ClaudeProvider`,
/// que vive dentro do laço do scheduler — inalcançável por um comando.
#[derive(Default)]
pub struct UsageStore {
    by_provider: HashMap<&'static str, Vec<UsageEvent>>,
}

impl UsageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Substitui os eventos de um provedor (o scanner devolve o conjunto inteiro a cada varredura).
    pub fn set_provider(&mut self, provider: &'static str, events: Vec<UsageEvent>) {
        self.by_provider.insert(provider, events);
    }

    pub fn events(&self) -> impl Iterator<Item = &UsageEvent> {
        self.by_provider.values().flatten()
    }

    pub fn is_empty(&self) -> bool {
        self.by_provider.values().all(|v| v.is_empty())
    }

    /// Agregado diário dos últimos 30 dias, no fuso de `now`. Alimenta o widget.
    pub fn activity<Tz: TimeZone>(&self, now: DateTime<Tz>) -> Activity {
        let today: NaiveDate = now.date_naive();
        let first = today - Duration::days(DAYS - 1);
        let mut days: Vec<crate::snapshot::DayActivity> = (0..DAYS)
            .map(|i| crate::snapshot::DayActivity { date: first + Duration::days(i), tokens: 0, messages: 0 })
            .collect();

        for e in self.events() {
            let date = e.ts.with_timezone(&now.timezone()).date_naive();
            if date < first || date > today {
                continue;
            }
            let d = &mut days[(date - first).num_days() as usize];
            d.tokens += e.tokens();
            d.messages += 1;
        }

        let today_tokens = days.last().map_or(0, |d| d.tokens);
        Activity { days, today_tokens }
    }

    /// Calcula o painel sob demanda (nada disso vai no `Snapshot`, que é empurrado a cada refresh).
    pub fn report<Tz: TimeZone>(&self, filter: &ReportFilter, now: DateTime<Tz>, usd_brl: f64) -> UsageReport {
        let range = filter.range_days.clamp(1, DAYS);
        let tz = now.timezone();
        let today: NaiveDate = now.date_naive();
        let first = today - Duration::days(range - 1);

        let in_window = |e: &UsageEvent| {
            let date = e.ts.with_timezone(&tz).date_naive();
            date >= first && date <= today
        };

        // As famílias disponíveis saem do período inteiro, antes do filtro: senão, ao filtrar
        // por uma família, os chips das outras sumiriam e não daria pra voltar.
        let mut available: Vec<String> = self
            .events()
            .filter(|e| in_window(e))
            .map(|e| family(&e.model).to_string())
            .collect();
        available.sort_by_key(|f| family_rank(f));
        available.dedup();

        let selected = |e: &UsageEvent| {
            in_window(e)
                && (filter.families.is_empty() || filter.families.iter().any(|f| f == family(&e.model)))
        };

        let mut totals = Totals::default();
        let mut by_model: HashMap<String, Totals> = HashMap::new();
        let mut by_day: HashMap<NaiveDate, HashMap<&'static str, Totals>> = HashMap::new();
        let mut by_session: HashMap<String, (Totals, String, DateTime<Utc>, DateTime<Utc>)> = HashMap::new();
        let mut by_project: HashMap<String, (Totals, DateTime<Utc>, DateTime<Utc>)> = HashMap::new();
        let mut requests: Vec<&UsageEvent> = Vec::new();

        for e in self.events().filter(|e| selected(e)) {
            let fam = family(&e.model);
            let date = e.ts.with_timezone(&tz).date_naive();

            totals.add(e);
            by_model.entry(e.model.clone()).or_default().add(e);
            by_day.entry(date).or_default().entry(fam).or_default().add(e);

            let s = by_session
                .entry(e.session.clone())
                .or_insert_with(|| (Totals::default(), e.project.clone(), e.ts, e.ts));
            s.0.add(e);
            s.2 = s.2.min(e.ts);
            s.3 = s.3.max(e.ts);

            let p = by_project.entry(e.project.clone()).or_insert_with(|| (Totals::default(), e.ts, e.ts));
            p.0.add(e);
            p.1 = p.1.min(e.ts);
            p.2 = p.2.max(e.ts);

            requests.push(e);
        }

        // Custo por modelo; o que não tem preço entra em `unpriced_models` e não soma.
        let mut unpriced: Vec<String> = Vec::new();
        let mut models: Vec<ModelTotal> = by_model
            .into_iter()
            .map(|(model, t)| {
                let cost = pricing::price_for(&model).map(|p| pricing::cost_usd(p, &t) * usd_brl);
                if cost.is_none() {
                    unpriced.push(model.clone());
                }
                ModelTotal { family: family(&model).into(), model, totals: t, cost_brl: cost }
            })
            .collect();
        models.sort_by(|a, b| {
            b.totals
                .billable_tokens()
                .cmp(&a.totals.billable_tokens())
                .then_with(|| a.model.cmp(&b.model))
        });
        let total_cost_brl: f64 = models.iter().filter_map(|m| m.cost_brl).sum();
        unpriced.sort();

        // Dias: o período inteiro, do mais antigo pro mais recente, com os vazios zerados.
        let days: Vec<DaySlice> = (0..range)
            .map(|i| {
                let date = first + Duration::days(i);
                let fams = by_day.remove(&date).unwrap_or_default();
                let mut by_family: Vec<FamilySlice> = fams
                    .into_iter()
                    .map(|(f, t)| FamilySlice { family: f.into(), totals: t })
                    .collect();
                by_family.sort_by_key(|s| family_rank(&s.family));
                let totals = by_family.iter().fold(Totals::default(), |mut acc, s| {
                    acc.input += s.totals.input;
                    acc.output += s.totals.output;
                    acc.cache_read += s.totals.cache_read;
                    acc.cache_write_5m += s.totals.cache_write_5m;
                    acc.cache_write_1h += s.totals.cache_write_1h;
                    acc.messages += s.totals.messages;
                    acc
                });
                DaySlice { date, totals, by_family }
            })
            .collect();

        let mut sessions: Vec<GroupRow> = by_session
            .into_iter()
            .map(|(id, (totals, project, first_at, last_at))| GroupRow {
                // id completo é feio e não diz nada; o começo já distingue
                name: id.chars().take(8).collect(),
                sub: project,
                totals,
                first_at,
                last_at,
            })
            .collect();
        sessions.sort_by(|a, b| b.totals.billable_tokens().cmp(&a.totals.billable_tokens()));

        let mut projects: Vec<GroupRow> = by_project
            .into_iter()
            .map(|(name, (totals, first_at, last_at))| GroupRow { name, sub: String::new(), totals, first_at, last_at })
            .collect();
        projects.sort_by(|a, b| b.totals.billable_tokens().cmp(&a.totals.billable_tokens()));

        requests.sort_by(|a, b| b.ts.cmp(&a.ts));
        let request_count = requests.len();
        let recent: Vec<RequestRow> = requests
            .into_iter()
            .take(RECENT_LIMIT)
            .map(|e| RequestRow {
                ts: e.ts,
                family: family(&e.model).into(),
                model: e.model.clone(),
                session: e.session.chars().take(8).collect(),
                project: e.project.clone(),
                input: e.input,
                output: e.output,
                cache_read: e.cache_read,
                cache_write: e.cache_write(),
            })
            .collect();

        UsageReport {
            range_days: range,
            generated_at: Utc::now(),
            available_families: available,
            filtered_families: filter.families.clone(),
            days,
            by_model: models,
            totals,
            total_cost_brl,
            unpriced_models: unpriced,
            usd_brl,
            top_sessions: sessions.into_iter().take(RANK_LIMIT).collect(),
            top_projects: projects.into_iter().take(RANK_LIMIT).collect(),
            recent,
            request_count,
        }
    }
}

/// Linhas do CSV de exportação (mesmas colunas da tabela "Registros").
pub fn report_csv(report: &UsageReport) -> String {
    let mut out = String::from("data,modelo,tier,sessao,projeto,entrada,saida,cache_lido,cache_escrito\n");
    for r in &report.recent {
        // projetos e sessões vêm do disco do usuário e podem ter vírgula/aspas
        let esc = |s: &str| {
            if s.contains([',', '"', '\n']) {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            r.ts.to_rfc3339(),
            esc(&r.model),
            esc(&r.family),
            esc(&r.session),
            esc(&r.project),
            r.input,
            r.output,
            r.cache_read,
            r.cache_write,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn brt() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    fn now() -> DateTime<FixedOffset> {
        brt().with_ymd_and_hms(2026, 10, 10, 12, 0, 0).unwrap()
    }

    /// Horário **em UTC**, como vem no log — é o deslocamento pro fuso local que está sendo testado.
    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn ev(ts: DateTime<Utc>, model: &str, session: &str, project: &str, input: u64, output: u64) -> UsageEvent {
        UsageEvent {
            ts,
            provider: "claude",
            model: model.into(),
            session: session.into(),
            project: project.into(),
            input,
            output,
            cache_read: 0,
            cache_write_5m: 0,
            cache_write_1h: 0,
            sidechain: false,
            key: None,
        }
    }

    fn store(events: Vec<UsageEvent>) -> UsageStore {
        let mut s = UsageStore::new();
        s.set_provider("claude", events);
        s
    }

    #[test]
    fn family_groups_models_into_tiers() {
        assert_eq!(family("claude-opus-5-5"), "opus");
        assert_eq!(family("claude-sonnet-5"), "sonnet");
        assert_eq!(family("claude-haiku-4-5-20251001"), "haiku");
        assert_eq!(family("claude-fable-5-1"), "fable");
        assert_eq!(family("gpt-4"), "outro");
    }

    #[test]
    fn activity_keeps_the_widget_contract() {
        // tokens = entrada + saída + cache lido + cache criado, como sempre foi
        let mut e = ev(at(2026, 10, 10, 9), "claude-sonnet-5", "s1", "pacer", 10, 5);
        e.cache_read = 100;
        e.cache_write_5m = 50;
        let a = store(vec![e]).activity(now());
        assert_eq!(a.days.len(), 30);
        assert_eq!(a.days[29].date, NaiveDate::from_ymd_opt(2026, 10, 10).unwrap());
        assert_eq!(a.today_tokens, 165);
        assert_eq!(a.days[29].messages, 1);
    }

    #[test]
    fn activity_uses_the_local_day_not_the_utc_day() {
        // 02:30 UTC do dia 10 = 23:30 do dia 9 em UTC−3
        let a = store(vec![ev(at(2026, 10, 10, 2), "claude-sonnet-5", "s", "p", 40, 0)]).activity(now());
        assert_eq!(a.days[28].tokens, 40);
        assert_eq!(a.today_tokens, 0);
    }

    #[test]
    fn report_covers_the_whole_range_with_empty_days() {
        let r = store(vec![ev(at(2026, 10, 10, 9), "claude-sonnet-5", "s", "p", 10, 5)])
            .report(&ReportFilter { range_days: 7, families: vec![] }, now(), 5.0);
        assert_eq!(r.days.len(), 7);
        assert_eq!(r.days[0].date, NaiveDate::from_ymd_opt(2026, 10, 4).unwrap());
        assert_eq!(r.days[6].date, NaiveDate::from_ymd_opt(2026, 10, 10).unwrap());
        assert_eq!(r.days[0].totals.messages, 0);
        assert_eq!(r.days[6].totals.input, 10);
        assert_eq!(r.totals.messages, 1);
    }

    #[test]
    fn report_leaves_events_outside_the_range_behind() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-sonnet-5", "s", "p", 10, 0),
            ev(at(2026, 10, 1, 9), "claude-sonnet-5", "s", "p", 999, 0),
        ]);
        let r = s.report(&ReportFilter { range_days: 7, families: vec![] }, now(), 5.0);
        assert_eq!(r.totals.input, 10);
        let r30 = s.report(&ReportFilter { range_days: 30, families: vec![] }, now(), 5.0);
        assert_eq!(r30.totals.input, 1009);
    }

    #[test]
    fn report_stacks_by_family_per_day() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-opus-5-5", "s", "p", 100, 0),
            ev(at(2026, 10, 10, 10), "claude-sonnet-5", "s", "p", 50, 0),
        ]);
        let r = s.report(&ReportFilter { range_days: 7, families: vec![] }, now(), 5.0);
        let day = &r.days[6];
        assert_eq!(day.by_family.len(), 2);
        // ordem canônica: opus antes de sonnet
        assert_eq!(day.by_family[0].family, "opus");
        assert_eq!(day.by_family[1].family, "sonnet");
        assert_eq!(day.totals.input, 150);
    }

    #[test]
    fn filtering_by_family_keeps_the_other_chips_available() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-opus-5-5", "s", "p", 100, 0),
            ev(at(2026, 10, 10, 10), "claude-sonnet-5", "s", "p", 50, 0),
        ]);
        let r = s.report(&ReportFilter { range_days: 7, families: vec!["opus".into()] }, now(), 5.0);
        assert_eq!(r.totals.input, 100);
        // os chips continuam mostrando as duas famílias, senão não daria pra voltar
        assert_eq!(r.available_families, vec!["opus", "sonnet"]);
        assert_eq!(r.filtered_families, vec!["opus"]);
        assert_eq!(r.by_model.len(), 1);
    }

    #[test]
    fn ranks_sessions_and_projects_by_billable_tokens() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-opus-5-5", "sessao-a", "pacer", 100, 0),
            ev(at(2026, 10, 10, 10), "claude-opus-5-5", "sessao-a", "pacer", 50, 0),
            ev(at(2026, 10, 10, 11), "claude-opus-5-5", "sessao-b", "outro", 10, 0),
        ]);
        let r = s.report(&ReportFilter::default(), now(), 5.0);
        assert_eq!(r.top_sessions[0].name, "sessao-a");
        assert_eq!(r.top_sessions[0].totals.input, 150);
        assert_eq!(r.top_sessions[0].sub, "pacer");
        assert_eq!(r.top_projects[0].name, "pacer");
        assert_eq!(r.top_projects[0].totals.input, 150);
        // primeira e última atividade da sessão
        assert!(r.top_sessions[0].first_at < r.top_sessions[0].last_at);
    }

    #[test]
    fn recent_requests_come_newest_first_and_counted() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-opus-5-5", "s", "p", 1, 0),
            ev(at(2026, 10, 10, 11), "claude-opus-5-5", "s", "p", 3, 0),
            ev(at(2026, 10, 10, 10), "claude-opus-5-5", "s", "p", 2, 0),
        ]);
        let r = s.report(&ReportFilter::default(), now(), 5.0);
        let entradas: Vec<u64> = r.recent.iter().map(|x| x.input).collect();
        assert_eq!(entradas, vec![3, 2, 1]);
        assert_eq!(r.request_count, 3);
        assert_eq!(r.recent[0].family, "opus");
    }

    #[test]
    fn cost_converts_with_the_configured_rate_and_flags_unpriced_models() {
        let s = store(vec![
            ev(at(2026, 10, 10, 9), "claude-sonnet-5", "s", "p", 1_000_000, 0),
            ev(at(2026, 10, 10, 9), "modelo-desconhecido", "s", "p", 1_000_000, 0),
        ]);
        let r = s.report(&ReportFilter::default(), now(), 5.0);
        // US$ 2 de entrada do Sonnet 5 × 5,00 = R$ 10
        assert!((r.total_cost_brl - 10.0).abs() < 1e-9);
        assert_eq!(r.usd_brl, 5.0);
        assert_eq!(r.unpriced_models, vec!["modelo-desconhecido"]);
        // o modelo sem preço não ganha um valor inventado
        let sem_preco = r.by_model.iter().find(|m| m.model == "modelo-desconhecido").unwrap();
        assert_eq!(sem_preco.cost_brl, None);
    }

    #[test]
    fn csv_quotes_names_that_would_break_the_columns() {
        let s = store(vec![ev(at(2026, 10, 10, 9), "claude-sonnet-5", "s", "obra, fase 2", 1, 2)]);
        let r = s.report(&ReportFilter::default(), now(), 5.0);
        let csv = report_csv(&r);
        assert!(csv.starts_with("data,modelo,tier,sessao,projeto,entrada,saida,cache_lido,cache_escrito\n"));
        assert!(csv.contains("\"obra, fase 2\""));
    }

    #[test]
    fn empty_store_reports_zeroed_days_without_panicking() {
        let r = store(vec![]).report(&ReportFilter::default(), now(), 5.0);
        assert_eq!(r.days.len(), 30);
        assert_eq!(r.totals.messages, 0);
        assert_eq!(r.total_cost_brl, 0.0);
        assert!(r.top_projects.is_empty());
        assert!(r.recent.is_empty());
    }
}
