use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AlertsConfig {
    pub enabled: bool,
    pub thresholds: Vec<u8>,
    pub pace: bool,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        Self { enabled: true, thresholds: vec![80, 95], pace: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderToggle {
    pub enabled: bool,
}

impl Default for ProviderToggle {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProvidersConfig {
    pub claude: ProviderToggle,
}

/// Cor de destaque: substitui só o estado "ok" da escala de severidade (anel da bandeja, barras,
/// chrome geral da UI) — aviso/crítico continuam laranja/vermelho fixos. Só presets, sem hex livre.
pub const DEFAULT_ACCENT: &str = "#1f6feb";
pub const ACCENT_PRESETS: [&str; 5] = ["#1f6feb", "#8957e5", "#2ea043", "#db61a2", "#39c5cf"];

/// Cotação US$→R$ usada só para estimar o consumo a preço de API no painel. É uma referência
/// (BCB, 06/10/2026), não uma verdade: quem decide o valor é o usuário, nas Configurações.
pub const DEFAULT_USD_BRL: f64 = 4.97;
/// Faixa aceita; fora dela (ou NaN) volta pro padrão.
const USD_BRL_RANGE: std::ops::RangeInclusive<f64> = 0.5..=20.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub start_with_windows: bool,
    pub refresh_minutes: u32,
    pub accent_color: String,
    pub usd_brl: f64,
    pub alerts: AlertsConfig,
    pub providers: ProvidersConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            refresh_minutes: 5,
            accent_color: DEFAULT_ACCENT.to_string(),
            usd_brl: DEFAULT_USD_BRL,
            alerts: AlertsConfig::default(),
            providers: ProvidersConfig::default(),
        }
    }
}

impl Config {
    pub fn normalized(mut self) -> Self {
        if ![1, 5, 10].contains(&self.refresh_minutes) {
            self.refresh_minutes = 5;
        }
        if !ACCENT_PRESETS.contains(&self.accent_color.as_str()) {
            self.accent_color = DEFAULT_ACCENT.to_string();
        }
        if !self.usd_brl.is_finite() || !USD_BRL_RANGE.contains(&self.usd_brl) {
            self.usd_brl = DEFAULT_USD_BRL;
        }
        self.alerts.thresholds.retain(|t| (1..=100).contains(t));
        self.alerts.thresholds.sort_unstable();
        self.alerts.thresholds.dedup();
        self
    }

    pub fn provider_enabled(&self, id: &str) -> bool {
        match id {
            "claude" => self.providers.claude.enabled,
            _ => false,
        }
    }
}

/// Lê a config; arquivo ausente → padrões; inválido → padrões e o original vira `.bak`.
pub fn load(path: &Path) -> Config {
    let Ok(raw) = fs::read_to_string(path) else {
        return Config::default();
    };
    match serde_json::from_str::<Config>(&raw) {
        Ok(c) => c.normalized(),
        Err(_) => {
            let _ = fs::rename(path, path.with_extension("json.bak"));
            Config::default()
        }
    }
}

/// Gravação atômica: escreve num `.tmp` e renomeia por cima.
pub fn save(path: &Path, cfg: &Config) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(cfg)?)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let c = load(&dir.path().join("config.json"));
        assert_eq!(c, Config::default());
        assert_eq!(c.refresh_minutes, 5);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
        assert!(c.start_with_windows && c.providers.claude.enabled);
    }

    #[test]
    fn partial_file_fills_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"refreshMinutes":10,"alerts":{"pace":false}}"#).unwrap();
        let c = load(&p);
        assert_eq!(c.refresh_minutes, 10);
        assert!(!c.alerts.pace);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
    }

    #[test]
    fn invalid_values_are_normalized() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"refreshMinutes":7,"alerts":{"thresholds":[95,0,80,80,120]}}"#).unwrap();
        let c = load(&p);
        assert_eq!(c.refresh_minutes, 5);
        assert_eq!(c.alerts.thresholds, vec![80, 95]);
    }

    #[test]
    fn corrupt_file_becomes_bak_and_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, "{ quebrado").unwrap();
        assert_eq!(load(&p), Config::default());
        assert!(!p.exists());
        assert_eq!(fs::read_to_string(dir.path().join("config.json.bak")).unwrap(), "{ quebrado");
    }

    #[test]
    fn save_roundtrip_without_leftover_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("sub").join("config.json");
        let mut c = Config::default();
        c.refresh_minutes = 1;
        save(&p, &c).unwrap();
        assert_eq!(load(&p), c);
        assert!(!dir.path().join("sub").join("config.json.tmp").exists());
        let raw = fs::read_to_string(&p).unwrap();
        assert!(raw.contains("\"refreshMinutes\": 1"));
        // o widget agora é fixo no canto: não existe mais "travar posição"
        assert!(!raw.contains("lockPosition"));
    }

    #[test]
    fn old_config_with_lock_position_still_loads() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"lockPosition":true,"refreshMinutes":10}"#).unwrap();
        let c = load(&p);
        assert_eq!(c.refresh_minutes, 10);
        assert!(p.exists(), "campo antigo não pode contar como arquivo corrompido");
    }

    #[test]
    fn accent_color_defaults_to_the_first_preset() {
        assert_eq!(Config::default().accent_color, DEFAULT_ACCENT);
        assert_eq!(DEFAULT_ACCENT, ACCENT_PRESETS[0]);
    }

    #[test]
    fn accent_color_outside_the_preset_list_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r##"{"accentColor":"#ff00ff"}"##).unwrap();
        assert_eq!(load(&p).accent_color, DEFAULT_ACCENT);
    }

    #[test]
    fn accent_color_preset_survives_normalization() {
        let mut c = Config { accent_color: ACCENT_PRESETS[2].to_string(), ..Config::default() };
        c = c.normalized();
        assert_eq!(c.accent_color, ACCENT_PRESETS[2]);
    }

    #[test]
    fn usd_brl_defaults_and_survives_a_roundtrip() {
        assert_eq!(Config::default().usd_brl, DEFAULT_USD_BRL);
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        save(&p, &Config { usd_brl: 5.75, ..Config::default() }).unwrap();
        assert_eq!(load(&p).usd_brl, 5.75);
        // camelCase no arquivo, como o resto
        assert!(fs::read_to_string(&p).unwrap().contains("\"usdBrl\": 5.75"));
    }

    #[test]
    fn absurd_exchange_rates_fall_back_to_the_default() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        for bruto in ["0", "-3", "999", "null"] {
            fs::write(&p, format!(r#"{{"usdBrl":{bruto}}}"#)).unwrap();
            assert_eq!(load(&p).usd_brl, DEFAULT_USD_BRL, "aceitou {bruto}");
        }
    }

    #[test]
    fn config_without_usd_brl_gets_the_default() {
        // arquivo escrito antes deste campo existir
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("config.json");
        fs::write(&p, r#"{"refreshMinutes":10}"#).unwrap();
        assert_eq!(load(&p).usd_brl, DEFAULT_USD_BRL);
        assert_eq!(load(&p).refresh_minutes, 10);
    }

    #[test]
    fn provider_enabled_lookup() {
        let mut c = Config::default();
        assert!(c.provider_enabled("claude"));
        c.providers.claude.enabled = false;
        assert!(!c.provider_enabled("claude"));
        assert!(!c.provider_enabled("desconhecido"));
    }
}
