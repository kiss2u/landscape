pub(crate) mod connect;
pub(crate) mod dns;
mod global_stats;
mod ingest;
mod runtime;
mod store;

pub use store::DuckMetricStore;
