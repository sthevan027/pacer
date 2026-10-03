pub mod claude;

use crate::snapshot::Snapshot;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchResult {
    Ok,
    RateLimited,
    Failed,
    /// Não houve chamada (sem login, token vencido).
    Skipped,
}

#[async_trait]
pub trait Provider: Send {
    fn id(&self) -> &'static str;
    /// Atualização local barata (logs, recálculo da projeção). Chamado a cada ~10 s.
    fn tick_local(&mut self, now: DateTime<Utc>);
    /// Busca remota (API). O scheduler decide quando chamar.
    async fn fetch_remote(&mut self, now: DateTime<Utc>) -> FetchResult;
    fn snapshot(&self) -> Snapshot;
}
