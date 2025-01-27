pub mod ingestion;
pub mod processing;

pub use ingestion::DataIngestion;
pub use processing::{DataProcessor, ProcessedMarketData};

// Re-export for tests
#[cfg(test)]
pub use ingestion::MarketData;
