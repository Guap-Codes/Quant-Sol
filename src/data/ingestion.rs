use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;

/// Represents a single market data point for a financial instrument.
///
/// This struct captures key information about a financial asset at a specific point in time,
/// including timestamp, symbol, price, volume, and price extremes.
///
/// # Fields
/// * `timestamp`: The exact time of the market data point
/// * `symbol`: The trading symbol of the financial instrument
/// * `price`: The current trading price
/// * `volume`: The total trading volume
/// * `high`: The highest price during the period
/// * `low`: The lowest price during the period
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketData {
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub high: f64,
    pub low: f64,
}

/// Manages data ingestion from external financial data APIs.
///
/// This struct provides methods to fetch market data, primarily focusing on cryptocurrency
/// data retrieval using the Alpha Vantage API. It handles API authentication,
/// request generation, and response parsing.
///
/// # Key Features
/// * Fetches daily cryptocurrency market data
/// * Supports historical data retrieval
/// * Robust error handling for API interactions
/// * Automatic environment-based API key management
pub struct DataIngestion {
    api_key: String,
    client: reqwest::Client,
}

impl DataIngestion {
    /// Creates a new `DataIngestion` instance with API credentials.
    ///
    /// Retrieves the Alpha Vantage API key from environment variables.
    ///
    /// # Errors
    /// Returns an error if the `ALPHA_VANTAGE_API_KEY` environment variable is not set
    ///
    /// # Returns
    /// A new `DataIngestion` instance with configured HTTP client
    pub fn new() -> Result<Self> {
        let api_key = env::var("ALPHA_VANTAGE_API_KEY")
            .expect("ALPHA_VANTAGE_API_KEY must be set in environment");

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }

    /// Fetches daily cryptocurrency market data for a given symbol.
    ///
    /// Retrieves the most recent daily market data from the Alpha Vantage API
    /// for the specified cryptocurrency symbol.
    ///
    /// # Arguments
    /// * `symbol`: The cryptocurrency symbol to fetch data for (e.g., "BTC", "ETH")
    ///
    /// # Errors
    /// Returns an error if:
    /// - API request fails
    /// - Response parsing encounters issues
    /// - No data is available for the symbol
    ///
    /// # Returns
    /// A vector of `MarketData` sorted from most recent to oldest
    pub async fn fetch_crypto_data(&self, symbol: &str) -> Result<Vec<MarketData>> {
        let url = format!(
            "https://www.alphavantage.co/query?function=DIGITAL_CURRENCY_DAILY&symbol={}&market=USD&apikey={}",
            symbol, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Check for error messages
        if let Some(error_message) = response.get("Error Message") {
            return Err(anyhow::anyhow!(
                "Alpha Vantage API error: {}",
                error_message.as_str().unwrap_or("Unknown error")
            ));
        }

        // Check for information messages (like rate limiting)
        if let Some(info) = response.get("Note") {
            eprintln!("Alpha Vantage API note: {}", info.as_str().unwrap_or(""));
            // Continue processing if it's just a warning
        }

        // Parse the response into our MarketData structure
        let time_series = match response.get("Time Series (Digital Currency Daily)") {
            Some(ts) => ts.as_object().ok_or_else(|| {
                eprintln!("Unexpected API response format: {:?}", response);
                anyhow::anyhow!("Invalid response format: Time Series data not found")
            })?,
            None => {
                // Print the full response for debugging
                eprintln!("API Response Debug: {:#?}", response);

                // Check for common error conditions
                if let Some(note) = response.get("Note") {
                    return Err(anyhow::anyhow!(
                        "API Rate limit: {}",
                        note.as_str().unwrap_or("Unknown rate limit message")
                    ));
                }

                if let Some(info) = response.get("Information") {
                    return Err(anyhow::anyhow!(
                        "API Information: {}",
                        info.as_str().unwrap_or("Unknown information message")
                    ));
                }

                return Err(anyhow::anyhow!("Time Series data not found in response. This could be due to an invalid API key, rate limiting, or invalid symbol."));
            }
        };

        let mut market_data = Vec::new();

        for (timestamp_str, data) in time_series {
            let data = data.as_object().ok_or_else(|| {
                anyhow::anyhow!("Invalid data format for timestamp {}", timestamp_str)
            })?;

            // Use more robust error handling for data extraction
            let market_entry = MarketData {
                timestamp: DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", timestamp_str))
                    .map_err(|e| anyhow::anyhow!("Invalid timestamp format: {}", e))?
                    .with_timezone(&Utc),
                symbol: symbol.to_string(),
                price: {
                    // Debug print available keys
                    eprintln!(
                        "Available data keys: {:#?}",
                        data.keys().collect::<Vec<_>>()
                    );

                    data.get("4a. close (USD)")
                        .or_else(|| data.get("4. close"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find close price in data: {:#?}", data);
                            anyhow::anyhow!("Close price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Close price is not a string"))?
                        .parse()?
                },
                volume: {
                    data.get("5. volume")
                        .ok_or_else(|| {
                            eprintln!("Failed to find volume in data: {:#?}", data);
                            anyhow::anyhow!("Volume not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Volume is not a string"))?
                        .parse()?
                },
                high: {
                    data.get("2a. high (USD)")
                        .or_else(|| data.get("2. high"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find high price in data: {:#?}", data);
                            anyhow::anyhow!("High price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("High price is not a string"))?
                        .parse()?
                },
                low: {
                    data.get("3a. low (USD)")
                        .or_else(|| data.get("3. low"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find low price in data: {:#?}", data);
                            anyhow::anyhow!("Low price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Low price is not a string"))?
                        .parse()?
                },
            };

            market_data.push(market_entry);
        }

        // Sort by timestamp in descending order to get most recent first
        market_data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if market_data.is_empty() {
            return Err(anyhow::anyhow!("No market data returned from API"));
        }

        Ok(market_data)
    }

    /// Fetches historical cryptocurrency market data within a specified date range.
    ///
    /// Retrieves daily market data for a cryptocurrency between the given start and end dates.
    ///
    /// # Arguments
    /// * `symbol`: The cryptocurrency symbol to fetch data for (e.g., "BTC", "ETH")
    /// * `start_date`: The beginning of the date range (inclusive)
    /// * `end_date`: The end of the date range (inclusive)
    ///
    /// # Errors
    /// Returns an error if:
    /// - API request fails
    /// - Response parsing encounters issues
    /// - No data is available for the symbol or date range
    ///
    /// # Returns
    /// A vector of `MarketData` within the specified date range, sorted from most recent to oldest
    pub async fn fetch_historical_crypto_data(
        &self,
        symbol: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<MarketData>> {
        let url = format!(
            "https://www.alphavantage.co/query?function=DIGITAL_CURRENCY_DAILY&symbol={}&market=USD&apikey={}",
            symbol, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Check for error messages
        if let Some(error_message) = response.get("Error Message") {
            return Err(anyhow::anyhow!(
                "Alpha Vantage API error: {}",
                error_message.as_str().unwrap_or("Unknown error")
            ));
        }

        // Check for information messages (like rate limiting)
        if let Some(info) = response.get("Note") {
            eprintln!("Alpha Vantage API note: {}", info.as_str().unwrap_or(""));
            // Continue processing if it's just a warning
        }

        let time_series = response["Time Series (Digital Currency Daily)"]
            .as_object()
            .ok_or_else(|| {
                // Print the actual response for debugging
                eprintln!("Unexpected API response format: {:?}", response);
                anyhow::anyhow!("Invalid response format: Time Series data not found")
            })?;

        let mut market_data = Vec::new();

        for (timestamp_str, data) in time_series {
            let timestamp = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", timestamp_str))
                .map_err(|e| anyhow::anyhow!("Invalid timestamp format: {}", e))?
                .with_timezone(&Utc);

            if timestamp < start_date || timestamp > end_date {
                continue;
            }

            let data = data.as_object().ok_or_else(|| {
                anyhow::anyhow!("Invalid data format for timestamp {}", timestamp_str)
            })?;

            // Use more robust error handling for data extraction
            let market_entry = MarketData {
                timestamp,
                symbol: symbol.to_string(),
                price: {
                    // Debug print available keys
                    eprintln!(
                        "Available data keys: {:#?}",
                        data.keys().collect::<Vec<_>>()
                    );

                    data.get("4a. close (USD)")
                        .or_else(|| data.get("4. close"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find close price in data: {:#?}", data);
                            anyhow::anyhow!("Close price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Close price is not a string"))?
                        .parse()?
                },
                volume: {
                    data.get("5. volume")
                        .ok_or_else(|| {
                            eprintln!("Failed to find volume in data: {:#?}", data);
                            anyhow::anyhow!("Volume not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Volume is not a string"))?
                        .parse()?
                },
                high: {
                    data.get("2a. high (USD)")
                        .or_else(|| data.get("2. high"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find high price in data: {:#?}", data);
                            anyhow::anyhow!("High price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("High price is not a string"))?
                        .parse()?
                },
                low: {
                    data.get("3a. low (USD)")
                        .or_else(|| data.get("3. low"))
                        .ok_or_else(|| {
                            eprintln!("Failed to find low price in data: {:#?}", data);
                            anyhow::anyhow!("Low price not found in response")
                        })?
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("Low price is not a string"))?
                        .parse()?
                },
            };

            market_data.push(market_entry);
        }

        if market_data.is_empty() {
            return Err(anyhow::anyhow!(
                "No market data found in the specified date range"
            ));
        }

        Ok(market_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_crypto_data() {
        let ingestion = DataIngestion::new().unwrap();
        let data = ingestion.fetch_crypto_data("SOL").await;
        assert!(data.is_ok());
    }
}
