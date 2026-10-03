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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub start_with_windows: bool,
    pub lock_position: bool,
    pub refresh_minutes: u32,
    pub alerts: AlertsConfig,
    pub providers: ProvidersConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            lock_position: false,
            refresh_minutes: 5,
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
        assert!(c.start_with_windows && !c.lock_position && c.providers.claude.enabled);
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
        c.lock_position = true;
        c.refresh_minutes = 1;
        save(&p, &c).unwrap();
        assert_eq!(load(&p), c);
        assert!(!dir.path().join("sub").join("config.json.tmp").exists());
        let raw = fs::read_to_string(&p).unwrap();
        assert!(raw.contains("\"lockPosition\": true"));
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
