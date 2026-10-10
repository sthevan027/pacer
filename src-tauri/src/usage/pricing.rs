//! Preços oficiais, em USD por milhão de tokens, usados só para **estimar** quanto o consumo
//! custaria a preço de API — quem usa assinatura não paga nada disso.
//!
//! Fonte: <https://platform.claude.com/docs/en/about-claude/pricing>, lida em 2026-10-10.
//!
//! Modelo fora da tabela fica **sem valor** (`None`) em vez de virar zero: o total em R$ fica
//! subestimado e o painel avisa quais modelos faltam, em vez de mentir um número baixo.

use crate::usage::Totals;

const MTOK: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price {
    pub input: f64,
    pub write_5m: f64,
    pub write_1h: f64,
    pub read: f64,
    pub output: f64,
}

/// Casamento por **prefixo mais longo**: `claude-opus-5-5` ganha de `claude-opus-5`, e
/// `claude-haiku-4-5-20251001` cai em `claude-haiku-4-5`. Por isso todo modelo com preço
/// diferente precisa de entrada própria — `claude-opus-4` sozinho não pode "pegar" o 4.6.
const TABLE: &[(&str, Price)] = &[
    // US$ 10 / 50
    ("claude-fable-5-1", Price { input: 10.0, write_5m: 12.50, write_1h: 20.0, read: 0.25, output: 50.0 }),
    ("claude-fable-5", Price { input: 10.0, write_5m: 12.50, write_1h: 20.0, read: 1.00, output: 50.0 }),
    ("claude-mythos-5-1", Price { input: 10.0, write_5m: 12.50, write_1h: 20.0, read: 0.25, output: 50.0 }),
    ("claude-mythos-5", Price { input: 10.0, write_5m: 12.50, write_1h: 20.0, read: 1.00, output: 50.0 }),
    // US$ 4 / 20 — o Opus atual
    ("claude-opus-5-5", Price { input: 4.0, write_5m: 5.0, write_1h: 8.0, read: 0.20, output: 20.0 }),
    // US$ 5 / 25 — Opus 5 e toda a linha 4.5–4.8
    ("claude-opus-5", Price { input: 5.0, write_5m: 6.25, write_1h: 10.0, read: 0.50, output: 25.0 }),
    ("claude-opus-4-8", Price { input: 5.0, write_5m: 6.25, write_1h: 10.0, read: 0.50, output: 25.0 }),
    ("claude-opus-4-7", Price { input: 5.0, write_5m: 6.25, write_1h: 10.0, read: 0.50, output: 25.0 }),
    ("claude-opus-4-6", Price { input: 5.0, write_5m: 6.25, write_1h: 10.0, read: 0.50, output: 25.0 }),
    ("claude-opus-4-5", Price { input: 5.0, write_5m: 6.25, write_1h: 10.0, read: 0.50, output: 25.0 }),
    // US$ 15 / 75 — aposentados fora de Bedrock/Google Cloud
    ("claude-opus-4-1", Price { input: 15.0, write_5m: 18.75, write_1h: 30.0, read: 1.50, output: 75.0 }),
    ("claude-opus-4", Price { input: 15.0, write_5m: 18.75, write_1h: 30.0, read: 1.50, output: 75.0 }),
    // US$ 2 / 10
    ("claude-sonnet-5-5", Price { input: 2.0, write_5m: 2.50, write_1h: 4.0, read: 0.10, output: 10.0 }),
    ("claude-sonnet-5", Price { input: 2.0, write_5m: 2.50, write_1h: 4.0, read: 0.20, output: 10.0 }),
    // US$ 3 / 15
    ("claude-sonnet-4-6", Price { input: 3.0, write_5m: 3.75, write_1h: 6.0, read: 0.30, output: 15.0 }),
    ("claude-sonnet-4-5", Price { input: 3.0, write_5m: 3.75, write_1h: 6.0, read: 0.30, output: 15.0 }),
    ("claude-sonnet-4", Price { input: 3.0, write_5m: 3.75, write_1h: 6.0, read: 0.30, output: 15.0 }),
    // Haiku — a faixa de 100k tokens do Haiku 5.5 não dá pra separar pelo log; usamos a de baixo
    ("claude-haiku-5-5", Price { input: 0.10, write_5m: 0.125, write_1h: 0.20, read: 0.01, output: 0.50 }),
    ("claude-haiku-4-5", Price { input: 1.0, write_5m: 1.25, write_1h: 2.0, read: 0.10, output: 5.0 }),
    ("claude-haiku-3-5", Price { input: 0.80, write_5m: 1.0, write_1h: 1.60, read: 0.08, output: 4.0 }),
];

/// Preço do modelo, pelo prefixo mais longo que casa. `None` = fora da tabela.
pub fn price_for(model: &str) -> Option<Price> {
    TABLE
        .iter()
        .filter(|(prefix, _)| model.starts_with(prefix))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(_, p)| *p)
}

/// Custo em USD do consumo, já separando a escrita de cache de 5 min da de 1 h (os preços
/// diferem em 1,6×, e o log traz a divisão).
pub fn cost_usd(p: Price, t: &Totals) -> f64 {
    let part = |n: u64, rate: f64| (n as f64) * rate / MTOK;
    part(t.input, p.input)
        + part(t.output, p.output)
        + part(t.cache_read, p.read)
        + part(t.cache_write_5m, p.write_5m)
        + part(t.cache_write_1h, p.write_1h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn totals(input: u64, output: u64, cache_read: u64, c5: u64, c1h: u64) -> Totals {
        Totals { input, output, cache_read, cache_write_5m: c5, cache_write_1h: c1h, messages: 1 }
    }

    #[test]
    fn matches_the_longest_prefix_not_the_first() {
        // `claude-opus-5-5` também começa com `claude-opus-5`, que é mais caro
        assert_eq!(price_for("claude-opus-5-5").unwrap().input, 4.0);
        assert_eq!(price_for("claude-opus-5").unwrap().input, 5.0);
        // idem: o 4.6 não pode cair no preço do Opus 4 aposentado
        assert_eq!(price_for("claude-opus-4-6").unwrap().input, 5.0);
        assert_eq!(price_for("claude-opus-4").unwrap().input, 15.0);
        assert_eq!(price_for("claude-sonnet-4-6").unwrap().input, 3.0);
    }

    #[test]
    fn ignores_the_date_suffix_on_snapshot_ids() {
        // o log grava `claude-haiku-4-5-20251001`
        assert_eq!(price_for("claude-haiku-4-5-20251001"), price_for("claude-haiku-4-5"));
        assert_eq!(price_for("claude-haiku-4-5-20251001").unwrap().input, 1.0);
    }

    #[test]
    fn unknown_model_has_no_price_instead_of_zero() {
        assert_eq!(price_for("gpt-4"), None);
        assert_eq!(price_for(""), None);
        assert_eq!(price_for("<synthetic>"), None);
    }

    #[test]
    fn prices_every_model_that_shows_up_in_the_logs() {
        for m in [
            "claude-sonnet-5",
            "claude-opus-5-5",
            "claude-sonnet-5-5",
            "claude-sonnet-4-6",
            "claude-haiku-4-5-20251001",
            "claude-haiku-5-5",
        ] {
            assert!(price_for(m).is_some(), "sem preço: {m}");
        }
    }

    #[test]
    fn cost_is_per_million_tokens() {
        let p = Price { input: 4.0, write_5m: 5.0, write_1h: 8.0, read: 0.20, output: 20.0 };
        // 1M de entrada = US$ 4
        assert_eq!(cost_usd(p, &totals(1_000_000, 0, 0, 0, 0)), 4.0);
        // 1M de saída = US$ 20
        assert_eq!(cost_usd(p, &totals(0, 1_000_000, 0, 0, 0)), 20.0);
        // 500k de leitura de cache = US$ 0,10
        assert!((cost_usd(p, &totals(0, 0, 500_000, 0, 0)) - 0.10).abs() < 1e-9);
    }

    #[test]
    fn cache_write_1h_costs_more_than_5m() {
        let p = Price { input: 4.0, write_5m: 5.0, write_1h: 8.0, read: 0.20, output: 20.0 };
        let curto = cost_usd(p, &totals(0, 0, 0, 1_000_000, 0));
        let longo = cost_usd(p, &totals(0, 0, 0, 0, 1_000_000));
        assert_eq!(curto, 5.0);
        assert_eq!(longo, 8.0);
    }

    #[test]
    fn empty_usage_costs_nothing() {
        let p = price_for("claude-sonnet-5").unwrap();
        assert_eq!(cost_usd(p, &Totals::default()), 0.0);
    }
}
