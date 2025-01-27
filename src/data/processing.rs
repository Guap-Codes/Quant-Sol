use super::ingestion::MarketData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Represents processed market data with additional technical indicators and analysis.
///
/// This struct extends the raw market data with computed metrics such as moving averages,
/// Relative Strength Index (RSI), volatility, and outlier detection.
///
/// # Fields
/// * `raw_data`: The original market data
/// * `moving_average_5`: 5-period simple moving average
/// * `moving_average_20`: 20-period simple moving average
/// * `rsi_14`: 14-period Relative Strength Index
/// * `volatility`: Price volatility measure
/// * `is_outlier`: Indicates if the data point is considered an statistical outlier
pub struct ProcessedMarketData {
    pub raw_data: MarketData,
    pub moving_average_5: Option<f64>,
    pub moving_average_20: Option<f64>,
    pub rsi_14: Option<f64>,
    pub volatility: Option<f64>,
    pub is_outlier: bool,
}

/// A processor for computing technical indicators and performing data analysis on market data.
///
/// `DataProcessor` maintains a rolling window of price history and provides methods
/// to calculate various technical indicators and perform statistical analysis.
///
/// # Key Features
/// * Calculates moving averages
/// * Computes Relative Strength Index (RSI)
/// * Estimates price volatility
/// * Detects statistical outliers
pub struct DataProcessor {
    price_history: VecDeque<f64>,
    max_history_size: usize,
}

impl DataProcessor {
    /// Creates a new `DataProcessor` with a specified maximum history size.
    ///
    /// # Arguments
    /// * `max_history_size`: Maximum number of price points to retain in history
    ///
    /// # Returns
    /// A new `DataProcessor` instance
    pub fn new(max_history_size: usize) -> Self {
        Self {
            price_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
        }
    }

    /// Processes a single market data point and computes technical indicators.
    ///
    /// Updates the price history and calculates various metrics including:
    /// - Moving averages (5 and 20 periods)
    /// - Relative Strength Index (14 periods)
    /// - Price volatility
    /// - Outlier detection
    ///
    /// # Arguments
    /// * `market_data`: Raw market data to process
    ///
    /// # Returns
    /// A `Result` containing the processed market data with computed indicators
    pub fn process_data(&mut self, market_data: MarketData) -> Result<ProcessedMarketData> {
        // Update price history
        self.price_history.push_back(market_data.price);
        if self.price_history.len() > self.max_history_size {
            self.price_history.pop_front();
        }

        Ok(ProcessedMarketData {
            moving_average_5: self.calculate_moving_average(5),
            moving_average_20: self.calculate_moving_average(20),
            rsi_14: self.calculate_rsi(14),
            volatility: self.calculate_volatility(),
            is_outlier: self.detect_outlier(market_data.price),
            raw_data: market_data,
        })
    }

    /// Processes a batch of market data points.
    ///
    /// Applies `process_data` to each market data point in the input vector.
    ///
    /// # Arguments
    /// * `market_data`: Vector of market data points to process
    ///
    /// # Returns
    /// A `Result` containing a vector of processed market data
    pub fn process_batch(
        &mut self,
        market_data: Vec<MarketData>,
    ) -> Result<Vec<ProcessedMarketData>> {
        let mut processed_data = Vec::with_capacity(market_data.len());

        for data in market_data {
            processed_data.push(self.process_data(data)?);
        }

        Ok(processed_data)
    }

    /// Calculates the simple moving average for a given period.
    ///
    /// # Arguments
    /// * `period`: Number of periods to calculate the moving average
    ///
    /// # Returns
    /// An `Option<f64>` containing the moving average, or `None` if insufficient history
    fn calculate_moving_average(&self, period: usize) -> Option<f64> {
        if self.price_history.len() < period {
            return None;
        }

        let sum: f64 = self.price_history.iter().rev().take(period).sum();

        Some(sum / period as f64)
    }

    /// Calculates the Relative Strength Index (RSI) for a given period.
    ///
    /// Uses the Exponential Moving Average (EMA) method for RSI calculation.
    ///
    /// # Arguments
    /// * `period`: Number of periods to calculate RSI
    ///
    /// # Returns
    /// An `Option<f64>` containing the RSI value, or `None` if insufficient history
    fn calculate_rsi(&self, period: usize) -> Option<f64> {
        if self.price_history.len() < period + 1 {
            return None;
        }

        let mut gains = Vec::new();
        let mut losses = Vec::new();

        // Calculate price changes and separate into gains and losses
        for i in 1..=period {
            let current = self.price_history[self.price_history.len() - i];
            let previous = self.price_history[self.price_history.len() - i - 1];
            let change = current - previous;

            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // Calculate EMA of gains and losses
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut avg_gain = gains[0];
        let mut avg_loss = losses[0];

        for i in 1..gains.len() {
            avg_gain = (gains[i] * alpha) + (avg_gain * (1.0 - alpha));
            avg_loss = (losses[i] * alpha) + (avg_loss * (1.0 - alpha));
        }

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }

    /// Calculates the price volatility using standard deviation.
    ///
    /// Uses the most recent 20 price points to compute volatility.
    ///
    /// # Returns
    /// An `Option<f64>` containing the volatility, or `None` if insufficient history
    fn calculate_volatility(&self) -> Option<f64> {
        if self.price_history.len() < 20 {
            return None;
        }

        // Only use last 20 periods for more responsive volatility
        let prices: Vec<f64> = self.price_history.iter().rev().take(20).copied().collect();
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;

        let variance = prices
            .iter()
            .map(|&price| {
                let diff = price - mean;
                diff * diff
            })
            .sum::<f64>()
            / (prices.len() - 1) as f64;

        Some(variance.sqrt())
    }

    /// Detects statistical outliers in price data.
    ///
    /// Uses a z-score method to identify price points that deviate significantly
    /// from the recent price history.
    ///
    /// # Arguments
    /// * `price`: Current price to check for outlier status
    ///
    /// # Returns
    /// A boolean indicating whether the price is an outlier
    fn detect_outlier(&self, price: f64) -> bool {
        if self.price_history.len() < 4 {
            return false;
        }

        // Only use recent prices for outlier detection
        let recent_prices: Vec<f64> = self.price_history.iter().rev().take(20).copied().collect();
        let mean = recent_prices.iter().sum::<f64>() / recent_prices.len() as f64;

        if let Some(volatility) = self.calculate_volatility() {
            let z_score = (price - mean).abs() / volatility;
            return z_score > 4.0; // More permissive outlier detection
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_market_data(price: f64) -> MarketData {
        MarketData {
            timestamp: Utc::now(),
            symbol: "TEST".to_string(),
            price,
            volume: 1000.0,
            high: price + 1.0,
            low: price - 1.0,
        }
    }

    #[test]
    fn test_moving_average_calculation() {
        let mut processor = DataProcessor::new(100);
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        for price in prices {
            let market_data = create_test_market_data(price);
            let _ = processor.process_data(market_data);
        }

        let last_processed = processor
            .process_data(create_test_market_data(15.0))
            .unwrap();
        assert!(last_processed.moving_average_5.is_some());
        assert_eq!(last_processed.moving_average_5.unwrap(), 13.0);
    }

    #[test]
    fn test_outlier_detection() {
        let mut processor = DataProcessor::new(100);
        let normal_prices = vec![100.0, 101.0, 99.0, 100.5, 101.5];

        for price in normal_prices {
            let market_data = create_test_market_data(price);
            let processed = processor.process_data(market_data).unwrap();
            assert!(!processed.is_outlier);
        }

        // Test outlier
        let outlier = create_test_market_data(150.0);
        let processed = processor.process_data(outlier).unwrap();
        assert!(processed.is_outlier);
    }
}
