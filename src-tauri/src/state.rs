use crate::config::Config;
use crate::snapshot::Snapshot;
use crate::usage::UsageStore;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub struct Shared {
    pub snapshots: Mutex<Vec<Snapshot>>,
    pub config: Mutex<Config>,
    pub config_path: PathBuf,
    /// Eventos de consumo. O scheduler alimenta (via provedor) e o painel lê — por isso mora
    /// aqui e não dentro do provedor, que é inalcançável de um comando.
    pub usage: Arc<Mutex<UsageStore>>,
    pub force: AtomicBool,
    pub wake: Notify,
}

impl Shared {
    pub fn new(config_path: PathBuf, config: Config) -> Self {
        Self {
            snapshots: Mutex::new(Vec::new()),
            config: Mutex::new(config),
            config_path,
            usage: Arc::new(Mutex::new(UsageStore::new())),
            force: AtomicBool::new(false),
            wake: Notify::new(),
        }
    }

    /// Pede uma busca remota imediata (respeitando o mínimo de 60 s).
    pub fn request_refresh(&self) {
        self.force.store(true, Ordering::SeqCst);
        self.wake.notify_one();
    }

    pub fn take_force(&self) -> bool {
        self.force.swap(false, Ordering::SeqCst)
    }

    /// Acorda o laço sem forçar busca remota (ex.: config mudou).
    pub fn wake_up(&self) {
        self.wake.notify_one();
    }
}
