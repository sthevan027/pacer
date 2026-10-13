//! Chaves de API digitadas pelo usuário (OpenRouter, z.ai). Ficam no Gerenciador de Credenciais do
//! Windows, nunca no `config.json`, e o front-end só descobre *se* existe uma — o valor não volta
//! para a interface, não entra em log nem em mensagem de erro.

use std::collections::HashMap;
use std::sync::Mutex;

/// Provedores que aceitam chave digitada (os outros leem o login que a própria ferramenta deixou).
pub const KEY_PROVIDERS: [&str; 2] = ["openrouter", "zai"];

const SERVICE: &str = "Pacer";
const MAX_KEY_LEN: usize = 512;

pub trait SecretStore: Send + Sync {
    fn get(&self, id: &str) -> Option<String>;
    fn set(&self, id: &str, value: &str) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// Gerenciador de Credenciais do Windows (entrada `Pacer` / `<provedor>`).
pub struct KeyringStore;

impl KeyringStore {
    fn entry(id: &str) -> Result<keyring::Entry, String> {
        keyring::Entry::new(SERVICE, id).map_err(|e| e.to_string())
    }
}

impl SecretStore for KeyringStore {
    fn get(&self, id: &str) -> Option<String> {
        Self::entry(id).ok()?.get_password().ok().filter(|v| !v.is_empty())
    }

    fn set(&self, id: &str, value: &str) -> Result<(), String> {
        Self::entry(id)?.set_password(value).map_err(|e| e.to_string())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        match Self::entry(id)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Cofre em memória: os testes nunca tocam no Gerenciador de Credenciais de verdade.
#[derive(Default)]
pub struct MemoryStore(Mutex<HashMap<String, String>>);

impl SecretStore for MemoryStore {
    fn get(&self, id: &str) -> Option<String> {
        self.0.lock().unwrap().get(id).cloned()
    }

    fn set(&self, id: &str, value: &str) -> Result<(), String> {
        self.0.lock().unwrap().insert(id.into(), value.into());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        self.0.lock().unwrap().remove(id);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    UnknownProvider,
    Empty,
    Invalid,
}

impl KeyError {
    /// Mensagem para a interface. Nunca cita o valor digitado.
    pub fn message(self) -> &'static str {
        match self {
            KeyError::UnknownProvider => "Esse provedor não usa chave de API.",
            KeyError::Empty => "Cole a chave antes de salvar.",
            KeyError::Invalid => "Isso não parece uma chave: tem espaço, quebra de linha ou caractere estranho.",
        }
    }
}

/// Limpa o que costuma vir junto ao copiar (espaços, aspas, `Bearer `) e recusa o que não é chave.
pub fn clean_key(raw: &str) -> Result<String, KeyError> {
    let mut k = raw.trim();
    if let Some(rest) = k.strip_prefix("Bearer ").or_else(|| k.strip_prefix("bearer ")) {
        k = rest.trim();
    }
    let k = k.trim_matches(|c| c == '"' || c == '\'' || c == '`').trim();
    if k.is_empty() {
        return Err(KeyError::Empty);
    }
    if k.len() > MAX_KEY_LEN || !k.chars().all(|c| c.is_ascii_graphic()) {
        return Err(KeyError::Invalid);
    }
    Ok(k.to_string())
}

fn check_provider(id: &str) -> Result<(), KeyError> {
    if KEY_PROVIDERS.contains(&id) {
        Ok(())
    } else {
        Err(KeyError::UnknownProvider)
    }
}

/// Guarda a chave de um provedor depois de validar. Erro do cofre vira texto genérico.
pub fn save_key(store: &dyn SecretStore, provider: &str, raw: &str) -> Result<(), String> {
    check_provider(provider).map_err(|e| e.message().to_string())?;
    let key = clean_key(raw).map_err(|e| e.message().to_string())?;
    store.set(provider, &key).map_err(|_| "O Windows recusou guardar a chave no Gerenciador de Credenciais.".to_string())
}

pub fn remove_key(store: &dyn SecretStore, provider: &str) -> Result<(), String> {
    check_provider(provider).map_err(|e| e.message().to_string())?;
    store.delete(provider).map_err(|_| "Não consegui remover a chave do Gerenciador de Credenciais.".to_string())
}

/// Provedores que já têm chave guardada (só os ids, nunca os valores).
pub fn stored_ids(store: &dyn SecretStore) -> Vec<String> {
    KEY_PROVIDERS.iter().filter(|id| store.get(id).is_some()).map(|id| id.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_key_strips_what_copying_adds() {
        assert_eq!(clean_key("  sk-or-v1-abc \n").unwrap(), "sk-or-v1-abc");
        assert_eq!(clean_key("\"sk-or-v1-abc\"").unwrap(), "sk-or-v1-abc");
        assert_eq!(clean_key("Bearer sk-or-v1-abc").unwrap(), "sk-or-v1-abc");
        assert_eq!(clean_key("'abc.def'").unwrap(), "abc.def");
    }

    #[test]
    fn clean_key_refuses_things_that_are_not_keys() {
        assert_eq!(clean_key("   "), Err(KeyError::Empty));
        assert_eq!(clean_key("\"\""), Err(KeyError::Empty));
        assert_eq!(clean_key("duas palavras"), Err(KeyError::Invalid));
        assert_eq!(clean_key("abc\ndef"), Err(KeyError::Invalid));
        assert_eq!(clean_key("chávé"), Err(KeyError::Invalid));
        assert_eq!(clean_key(&"a".repeat(MAX_KEY_LEN + 1)), Err(KeyError::Invalid));
        assert!(clean_key(&"a".repeat(MAX_KEY_LEN)).is_ok());
    }

    #[test]
    fn save_and_remove_round_trip_through_the_store() {
        let s = MemoryStore::default();
        assert!(stored_ids(&s).is_empty());
        save_key(&s, "openrouter", " sk-or-v1-abc ").unwrap();
        assert_eq!(s.get("openrouter").as_deref(), Some("sk-or-v1-abc"));
        assert_eq!(stored_ids(&s), vec!["openrouter".to_string()]);
        save_key(&s, "zai", "zk").unwrap();
        assert_eq!(stored_ids(&s), vec!["openrouter".to_string(), "zai".to_string()]);
        remove_key(&s, "openrouter").unwrap();
        assert_eq!(stored_ids(&s), vec!["zai".to_string()]);
        remove_key(&s, "openrouter").unwrap(); // remover o que não existe não é erro
    }

    #[test]
    fn only_key_providers_are_accepted() {
        let s = MemoryStore::default();
        for id in ["claude", "copilot", "openai", "", "../x"] {
            assert!(save_key(&s, id, "abc").is_err(), "{id}");
            assert!(remove_key(&s, id).is_err(), "{id}");
        }
        assert!(s.0.lock().unwrap().is_empty());
    }

    #[test]
    fn errors_never_echo_the_typed_value() {
        let s = MemoryStore::default();
        let e = save_key(&s, "openrouter", "segredo com espaço").unwrap_err();
        assert!(!e.contains("segredo"));
        for k in [KeyError::UnknownProvider, KeyError::Empty, KeyError::Invalid] {
            assert!(!k.message().is_empty());
        }
    }

    #[test]
    fn a_failing_store_reports_a_generic_error() {
        struct Broken;
        impl SecretStore for Broken {
            fn get(&self, _: &str) -> Option<String> {
                None
            }
            fn set(&self, _: &str, v: &str) -> Result<(), String> {
                Err(format!("boom {v}"))
            }
            fn delete(&self, _: &str) -> Result<(), String> {
                Err("boom".into())
            }
        }
        let e = save_key(&Broken, "zai", "tok123").unwrap_err();
        assert!(!e.contains("tok123") && !e.contains("boom"));
        assert!(remove_key(&Broken, "zai").is_err());
    }

    /// Ida e volta no Gerenciador de Credenciais de verdade, com um nome de teste que o app nunca usa.
    /// `cargo test -- --ignored secrets` (mexe no cofre do usuário, por isso não roda sozinho).
    #[test]
    #[ignore]
    fn keyring_round_trip_on_this_machine() {
        let id = "pacer-selftest";
        let s = KeyringStore;
        s.set(id, "valor-de-teste").unwrap();
        assert_eq!(s.get(id).as_deref(), Some("valor-de-teste"));
        s.delete(id).unwrap();
        assert_eq!(s.get(id), None);
        s.delete(id).unwrap();
    }
}
