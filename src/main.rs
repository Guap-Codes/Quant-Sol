mod backtesting;
mod data;
//mod execution;
mod strategies;

use backtesting::{BacktestResult, Backtester, StrategyMode};
use chrono::{Duration, Utc};
use data::{DataIngestion, DataProcessor};
use dotenv::dotenv;
//use execution::binance::BinanceExecutor;

/// Monitors current market conditions for a specified cryptocurrency.
///
/// This function fetches the latest market data, processes it, and prints
/// key market indicators to provide a real-time market overview.
///
/// # Arguments
/// * `ingestion`: A reference to the `DataIngestion` instance for fetching market data
/// * `processor`: A mutable reference to the `DataProcessor` for processing market data
///
/// # Behavior
/// - Fetches current market data for the SOL cryptocurrency
/// - Processes the data and prints market status information
/// - Displays timestamp, symbol, price, volume, RSI, moving averages, and volatility
///
/// # Errors
/// Returns an error if data fetching or processing fails
async fn monitor_current_market(
    ingestion: &DataIngestion,
    processor: &mut DataProcessor,
) -> anyhow::Result<()> {
    println!("\nMonitoring current market conditions...");
    let current_data = ingestion.fetch_crypto_data("SOL").await?;

    if current_data.is_empty() {
        println!("No current market data available");
        return Ok(());
    }

    let processed_current = processor.process_batch(current_data)?;

    if let Some(latest) = processed_current.last() {
        println!("\nCurrent Market Status:");
        println!("Time: {}", latest.raw_data.timestamp);
        println!("Symbol: {}", latest.raw_data.symbol);
        println!("Price: ${:.4}", latest.raw_data.price);
        println!("Volume: {:.2} SOL", latest.raw_data.volume);

        if let Some(rsi) = latest.rsi_14 {
            println!("RSI (14): {:.2}", rsi);
        }

        if let Some(ma20) = latest.moving_average_20 {
            println!("20-day MA: ${:.4}", ma20);
        }

        if let Some(vol) = latest.volatility {
            println!("Volatility: {:.4}", vol);
        }
    }

    Ok(())
}

/// Prints detailed backtest results for a specific trading strategy.
///
/// Displays comprehensive performance metrics to help evaluate 
/// the effectiveness of a trading strategy.
///
/// # Arguments
/// * `results`: A reference to the `BacktestResult` containing performance metrics
/// * `strategy_name`: Name of the strategy being evaluated
///
/// # Metrics Displayed
/// - Total number of trades
/// - Win rate
/// - Total Profit and Loss (PnL)
/// - Sharpe Ratio
/// - Maximum drawdown
/// - Average win and loss
/// - Largest win and loss trades
fn print_backtest_results(results: &BacktestResult, strategy_name: &str) {
    println!("\n{} Results:", strategy_name);
    println!("Total Trades: {}", results.total_trades);
    println!("Win Rate: {:.2}%", results.win_rate * 100.0);
    println!("Total PnL: ${:.2}", results.total_pnl);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    println!("Average Win: ${:.2}", results.average_win);
    println!("Average Loss: ${:.2}", results.average_loss);
    println!("Largest Win: ${:.2}", results.largest_win);
    println!("Largest Loss: ${:.2}", results.largest_loss);
}

/// Compares backtest results between individual and combined trading strategies.
///
/// Provides a side-by-side comparison of key performance metrics to help
/// understand the relative performance of different strategies.
///
/// # Arguments
/// * `individual`: A reference to the `BacktestResult` of an individual strategy
/// * `combined`: A reference to the `BacktestResult` of the combined strategy
///
/// # Metrics Compared
/// - Total number of trades
/// - Win rate
/// - Total Profit and Loss (PnL)
/// - Sharpe Ratio
/// - Maximum drawdown
fn compare_results(individual: &BacktestResult, combined: &BacktestResult) {
    println!("\nStrategy Comparison (Individual vs Combined):");
    println!(
        "Trade Count: {} vs {}",
        individual.total_trades, combined.total_trades
    );
    println!(
        "Win Rate: {:.2}% vs {:.2}%",
        individual.win_rate * 100.0,
        combined.win_rate * 100.0
    );
    println!(
        "Total PnL: ${:.2} vs ${:.2}",
        individual.total_pnl, combined.total_pnl
    );
    println!(
        "Sharpe Ratio: {:.2} vs {:.2}",
        individual.sharpe_ratio, combined.sharpe_ratio
    );
    println!(
        "Max Drawdown: {:.2}% vs {:.2}%",
        individual.max_drawdown * 100.0,
        combined.max_drawdown * 100.0
    );
}

/// Main application entry point for the quantitative trading solution.
///
/// This function orchestrates the entire trading analysis workflow:
/// 1. Initialize logging and environment variables
/// 2. Create data ingestion and processing instances
/// 3. Monitor current market conditions
/// 4. Fetch historical market data
/// 5. Run backtests for individual and combined trading strategies
/// 6. Print and compare backtest results
///
/// # Workflow Steps
/// - Load environment variables from .env file
/// - Fetch and process current market data
/// - Retrieve historical market data for the past 180 days
/// - Run backtests for:
///   * RSI Strategy
///   * Bollinger Bands Strategy
///   * Combined Strategy
/// - Compare and display performance metrics
///
/// # Returns
/// Returns `Ok(())` if all operations complete successfully, 
/// otherwise returns an error
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv().ok();

    // Create data ingestion instance
    let ingestion = DataIngestion::new()?;

    // Create data processor with increased history capacity for better indicator calculations
    let mut processor = DataProcessor::new(500);

    // Monitor current market conditions
    monitor_current_market(&ingestion, &mut processor).await?;

    // Fetch historical data for backtesting
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(180);

    let historical_data = ingestion
        .fetch_historical_crypto_data("SOL", start_date, end_date)
        .await?;

    let processed_data = processor.process_batch(historical_data)?;

    // Create backtester with initial settings
    let mut backtester = Backtester::new(
        10000.0, 
        500.0,   
        0.001,   
    );

    // Run backtest with individual strategies
    backtester.set_strategy_mode(StrategyMode::Rsi);
    let rsi_results = backtester.run_backtest(&processed_data);
    print_backtest_results(&rsi_results, "RSI Strategy");

    backtester.set_strategy_mode(StrategyMode::BollingerBands);
    let bb_results = backtester.run_backtest(&processed_data);
    print_backtest_results(&bb_results, "Bollinger Bands Strategy");

    backtester.set_strategy_mode(StrategyMode::Combined);
    let combined_results = backtester.run_backtest(&processed_data);
    print_backtest_results(&combined_results, "Combined Strategy");

    compare_results(&rsi_results, &combined_results);
    compare_results(&bb_results, &combined_results);

    Ok(())
}
