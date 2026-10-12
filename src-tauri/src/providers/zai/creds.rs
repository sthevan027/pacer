//! De onde sai o token do z.ai. O z.ai é usado pelo Claude Code trocando `ANTHROPIC_BASE_URL`
//! e `ANTHROPIC_AUTH_TOKEN` (em `~/.claude/settings.json`, bloco `env`, ou no ambiente), e é o que
//! o plugin oficial do z.ai lê. Também aceita `ZAI_API_KEY`/`ZHIPU_API_KEY`. Só leitura.

use serde_json::Value;

#[derive(Clone, PartialEq)]
pub struct Auth {
    pub token: String,
    /// Domínio da API, sem barra no fim (`https://api.z.ai`).
    pub domain: String,
}

impl std::fmt::Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Auth").field("token", &"<oculto>").field("domain", &self.domain).finish()
    }
}

pub const ZAI: &str = "https://api.z.ai";
pub const ZHIPU: &str = "https://open.bigmodel.cn";

/// Domínio do z.ai a partir do `ANTHROPIC_BASE_URL`; qualquer outro endereço (a Anthropic mesmo)
/// devolve `None`, para nunca mandar o token de um serviço a outro.
pub fn domain_for(base_url: &str) -> Option<&'static str> {
    if base_url.contains("api.z.ai") {
        Some(ZAI)
    } else if base_url.contains("open.bigmodel.cn") || base_url.contains("dev.bigmodel.cn") {
        Some(ZHIPU)
    } else {
        None
    }
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// `env` lê uma variável de ambiente; `settings` é o JSON do `settings.json` do Claude Code.
pub fn find(env: &dyn Fn(&str) -> Option<String>, settings: Option<&Value>) -> Option<Auth> {
    if let Some(token) = non_empty(env("ZAI_API_KEY")) {
        return Some(Auth { token, domain: ZAI.into() });
    }
    if let Some(token) = non_empty(env("ZHIPU_API_KEY")) {
        return Some(Auth { token, domain: ZHIPU.into() });
    }
    let from_settings = |key: &str| non_empty(settings.and_then(|s| s.get("env")?.get(key)?.as_str().map(str::to_string)));
    for (token, base) in [
        (from_settings("ANTHROPIC_AUTH_TOKEN"), from_settings("ANTHROPIC_BASE_URL")),
        (non_empty(env("ANTHROPIC_AUTH_TOKEN")), non_empty(env("ANTHROPIC_BASE_URL"))),
    ] {
        if let (Some(token), Some(domain)) = (token, base.as_deref().and_then(domain_for)) {
            return Some(Auth { token, domain: domain.into() });
        }
    }
    None
}

pub fn discover() -> Option<Auth> {
    let settings = std::fs::read_to_string(crate::providers::claude::creds::claude_dir().join("settings.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
    find(&|k| std::env::var(k).ok(), settings.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn env_of(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| v.to_string())
    }

    #[test]
    fn only_zai_and_bigmodel_addresses_count() {
        assert_eq!(domain_for("https://api.z.ai/api/anthropic"), Some(ZAI));
        assert_eq!(domain_for("https://open.bigmodel.cn/api/anthropic"), Some(ZHIPU));
        assert_eq!(domain_for("https://dev.bigmodel.cn/x"), Some(ZHIPU));
        assert_eq!(domain_for("https://api.anthropic.com"), None);
        assert_eq!(domain_for(""), None);
    }

    #[test]
    fn dedicated_variables_win() {
        let a = find(&env_of(&[("ZAI_API_KEY", " k1 ")]), None).unwrap();
        assert_eq!((a.token.as_str(), a.domain.as_str()), ("k1", ZAI));
        let b = find(&env_of(&[("ZHIPU_API_KEY", "k2")]), None).unwrap();
        assert_eq!((b.token.as_str(), b.domain.as_str()), ("k2", ZHIPU));
    }

    #[test]
    fn claude_code_settings_with_a_zai_base_url_are_used() {
        let s = json!({"env":{"ANTHROPIC_AUTH_TOKEN":"abc","ANTHROPIC_BASE_URL":"https://api.z.ai/api/anthropic"}});
        let a = find(&env_of(&[]), Some(&s)).unwrap();
        assert_eq!((a.token.as_str(), a.domain.as_str()), ("abc", ZAI));
    }

    #[test]
    fn a_token_pointed_at_anthropic_or_without_base_url_is_never_taken() {
        let anth = json!({"env":{"ANTHROPIC_AUTH_TOKEN":"abc","ANTHROPIC_BASE_URL":"https://api.anthropic.com"}});
        assert_eq!(find(&env_of(&[]), Some(&anth)), None);
        let no_base = json!({"env":{"ANTHROPIC_AUTH_TOKEN":"abc"}});
        assert_eq!(find(&env_of(&[]), Some(&no_base)), None);
        assert_eq!(find(&env_of(&[("ANTHROPIC_AUTH_TOKEN", "abc")]), None), None);
    }

    #[test]
    fn process_environment_is_the_last_resort() {
        let a = find(&env_of(&[("ANTHROPIC_AUTH_TOKEN", "e"), ("ANTHROPIC_BASE_URL", "https://open.bigmodel.cn/api/anthropic")]), None).unwrap();
        assert_eq!((a.token.as_str(), a.domain.as_str()), ("e", ZHIPU));
    }

    #[test]
    fn debug_never_prints_the_token() {
        let a = Auth { token: "segredo".into(), domain: ZAI.into() };
        assert!(!format!("{a:?}").contains("segredo"));
    }
}
