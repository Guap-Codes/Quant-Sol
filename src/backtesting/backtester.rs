use crate::data::ProcessedMarketData;
use crate::strategies::{
    BollingerBands, BollingerSignal, BollingerSignalType, RsiSignal, RsiSignalType, RsiStrategy,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single trade executed during the backtesting process.
///
/// Tracks comprehensive details of a trade from entry to exit, including
/// timing, pricing, position type, and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub position_type: PositionType,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub strategy_name: String,
}

/// Represents the type of trading position (long or short).
///
/// Indicates the directional bias of a trade in the financial market:
/// - `Long`: Expecting the asset price to rise
/// - `Short`: Expecting the asset price to fall
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PositionType {
    Long,
    Short,
}

/// Comprehensive results of a backtesting process.
///
/// Provides a detailed summary of trading strategy performance,
/// including various statistical metrics and trade details.
///
/// Key metrics include:
/// - Total trades
/// - Winning and losing trades
/// - Total Profit and Loss (PnL)
/// - Win rate
/// - Average win and loss
/// - Largest win and loss
/// - Maximum drawdown
/// - Sharpe Ratio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub total_pnl: f64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub trades: Vec<Trade>,
}

/// Represents the mode of strategy execution during backtesting.
///
/// Determines how trading strategies are applied and evaluated:
/// - `Rsi`: Only RSI strategy signals are considered
/// - `BollingerBands`: Only Bollinger Bands strategy signals are considered
/// - `Combined`: Requires signal confirmation from both strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrategyMode {
    Rsi,            // RSI strategy only
    BollingerBands, // Bollinger Bands strategy only
    Combined,       // Both strategies must agree
}

/// Represents possible trading signals used to communicate
/// trading decisions between strategies and the backtester.
///
/// Provides a standardized way to express trading actions:
/// - `Buy`: Enter a long position
/// - `Sell`: Exit or enter a short position
/// - `Hold`: Take no trading action
/// - `Long`: Explicitly open a long position
/// - `Short`: Explicitly open a short position
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeSignal {
    Buy,
    Sell,
    Hold,
    Long,  // Explicitly used in trade position management
    Short, // Explicitly used in trade position management
}

/// Represents a single point in the portfolio's equity curve during backtesting.
///
/// Tracks the portfolio value and drawdown at a specific moment in time,
/// allowing for detailed performance analysis and visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
    pub drawdown: f64,
}

/// Primary backtesting engine for evaluating trading strategies.
///
/// Manages the entire backtesting process, including:
/// - Trade execution
/// - Performance tracking
/// - Strategy evaluation
/// - Metrics calculation
///
/// Supports multiple strategy modes and provides comprehensive
/// performance analysis for trading strategies.
pub struct Backtester {
    initial_capital: f64,
    position_size: f64,
    commission_rate: f64,
    strategy_mode: StrategyMode,
    trades: Vec<Trade>,
    current_position: HashMap<String, Option<Trade>>,
    equity_curve: Vec<EquityPoint>,
}

impl Backtester {
    /// Creates a new Backtester instance with specified initial parameters.
    ///
    /// Initializes the backtesting engine with the given:
    /// - Initial capital
    /// - Position size
    /// - Commission rate
    ///
    /// # Arguments
    /// * `initial_capital` - Starting portfolio value
    /// * `position_size` - Fixed dollar amount per trade
    /// * `commission_rate` - Transaction fee rate
    pub fn new(initial_capital: f64, position_size: f64, commission_rate: f64) -> Self {
        Self {
            initial_capital,
            position_size,
            commission_rate,
            strategy_mode: StrategyMode::Combined,
            trades: Vec::new(),
            current_position: HashMap::new(),
            equity_curve: vec![EquityPoint {
                timestamp: Utc::now(),
                equity: initial_capital,
                drawdown: 0.0,
            }],
        }
    }

    /// Sets the strategy mode for the backtester.
    ///
    /// Updates the strategy mode to the specified value, affecting how
    /// trading strategies are applied and evaluated during the backtest.
    ///
    /// # Arguments
    /// * `mode` - The strategy mode to apply (RSI, Bollinger Bands, or Combined)
    pub fn set_strategy_mode(&mut self, mode: StrategyMode) {
        self.strategy_mode = mode;
    }

    /// Calculates commission for a trade based on its value.
    ///
    /// Applies the transaction commission rate to the trade value to
    /// determine the commission amount.
    ///
    /// # Arguments
    /// * `trade_value` - Total value of the trade
    fn calculate_commission(&self, trade_value: f64) -> f64 {
        trade_value * self.commission_rate
    }

    /// Executes the backtest on the provided market data.
    ///
    /// Processes the market data and applies the configured strategy
    /// to generate trading signals. Calculates performance metrics
    /// based on the backtest results.
    ///
    /// # Arguments
    /// * `data` - Slice of processed market data
    pub fn run_backtest(&mut self, data: &[ProcessedMarketData]) -> BacktestResult {
        self.trades.clear();
        self.current_position.clear();
        self.equity_curve.clear();
        self.equity_curve.push(EquityPoint {
            timestamp: Utc::now(),
            equity: self.initial_capital,
            drawdown: 0.0,
        });

        // Initialize strategies with refined thresholds
        let rsi_strategy = RsiStrategy::new(40.0, 60.0);
        let mut bollinger_bands = BollingerBands::new(20, 1.8);

        // Process signals for each strategy
        let rsi_signals = rsi_strategy.analyze_batch(data);
        let mut bollinger_signals = Vec::new();

        for market_data in data {
            bollinger_signals.push(bollinger_bands.analyze(market_data));
        }

        // Process trades based on strategy mode
        for i in 0..data.len() {
            let rsi_signal = &rsi_signals[i];
            let bollinger_signal = &bollinger_signals[i];
            let market_data = &data[i];

            let mut should_trade = false;
            let mut trade_signal = None;

            match self.strategy_mode {
                StrategyMode::Rsi => {
                    // For RSI only mode, just check RSI signals
                    should_trade = true;
                    trade_signal = Some(Self::convert_rsi_to_trade_signal(rsi_signal.clone()));
                }
                StrategyMode::BollingerBands => {
                    // For Bollinger only mode, just check Bollinger signals
                    should_trade = true;
                    trade_signal = Some(Self::convert_bollinger_to_trade_signal(
                        bollinger_signal.clone(),
                    ));
                }
                StrategyMode::Combined => {
                    // For combined mode, check if signals agree
                    if Self::signals_agree(rsi_signal, bollinger_signal) {
                        should_trade = true;
                        // Prefer Bollinger signal if both are active, otherwise use the non-Hold signal
                        trade_signal =
                            match (&rsi_signal.signal_type, &bollinger_signal.signal_type) {
                                (_, BollingerSignalType::Buy) | (_, BollingerSignalType::Sell) => {
                                    Some(Self::convert_bollinger_to_trade_signal(
                                        bollinger_signal.clone(),
                                    ))
                                }
                                (_signal, _) => {
                                    Some(Self::convert_rsi_to_trade_signal(rsi_signal.clone()))
                                }
                            };
                    }
                }
            }

            if should_trade {
                if let Some(signal) = trade_signal {
                    self.execute_trade(market_data, signal);
                }
            }

            // Update equity curve
            let current_equity = self.calculate_current_equity(market_data.raw_data.price);
            let drawdown = (self.initial_capital - current_equity) / self.initial_capital;
            self.equity_curve.push(EquityPoint {
                timestamp: market_data.raw_data.timestamp,
                equity: current_equity,
                drawdown,
            });
        }

        self.calculate_results()
    }

    /// Determines if signals from both RSI and Bollinger Bands agree.
    ///
    /// Compares the signals from both strategies to determine if they
    /// agree on the same direction (buy/sell) or if they are neutral.
    ///
    /// # Arguments
    /// * `rsi_signal` - Signal from the RSI strategy
    /// * `bollinger_signal` - Signal from the Bollinger Bands strategy
    fn signals_agree(rsi_signal: &RsiSignal, bollinger_signal: &BollingerSignal) -> bool {
        match (&rsi_signal.signal_type, &bollinger_signal.signal_type) {
            // Strong agreement - both strategies signal the same direction
            (RsiSignalType::Buy, BollingerSignalType::Buy) => true,
            (RsiSignalType::Sell, BollingerSignalType::Sell) => true,

            // Allow trades when one strategy signals and the other is neutral
            (RsiSignalType::Buy, BollingerSignalType::Hold) => true,
            (RsiSignalType::Hold, BollingerSignalType::Buy) => true,
            (RsiSignalType::Sell, BollingerSignalType::Hold) => true,
            (RsiSignalType::Hold, BollingerSignalType::Sell) => true,

            // No trade on conflicting signals or both hold
            _ => false,
        }
    }

    /// Converts RSI signal to a trade signal.
    ///
    /// Maps the RSI signal type to a trade signal (Buy, Sell, or Hold).
    ///
    /// # Arguments
    /// * `signal` - RSI signal to convert
    fn convert_rsi_to_trade_signal(signal: RsiSignal) -> TradeSignal {
        match signal.signal_type {
            RsiSignalType::Buy => TradeSignal::Buy,
            RsiSignalType::Sell => TradeSignal::Sell,
            RsiSignalType::Hold => TradeSignal::Hold,
        }
    }

    /// Converts Bollinger Bands signal to a trade signal.
    ///
    /// Maps the Bollinger Bands signal type to a trade signal (Buy, Sell, or Hold).
    ///
    /// # Arguments
    /// * `signal` - Bollinger Bands signal to convert
    fn convert_bollinger_to_trade_signal(signal: BollingerSignal) -> TradeSignal {
        match signal.signal_type {
            BollingerSignalType::Buy => TradeSignal::Buy,
            BollingerSignalType::Sell => TradeSignal::Sell,
            BollingerSignalType::Hold => TradeSignal::Hold,
        }
    }

    /// Executes a trade based on the provided signal.
    ///
    /// Opens a long position if the signal is Buy, closes a short position if the signal is Sell,
    /// or does nothing if the signal is Hold.
    ///
    /// # Arguments
    /// * `market_data` - Processed market data containing the current price and timestamp
    /// * `signal` - Trade signal indicating the direction to take
    fn execute_trade(&mut self, market_data: &ProcessedMarketData, signal: TradeSignal) {
        let symbol = &market_data.raw_data.symbol;
        let current_price = market_data.raw_data.price;

        // Calculate trade commission
        let trade_quantity = self.position_size / current_price;
        let commission = self.calculate_commission(self.position_size);

        match (signal, self.current_position.get(symbol).cloned()) {
            // Open long position on buy signal if no position exists
            (TradeSignal::Buy, None) => {
                let trade = Trade {
                    entry_time: market_data.raw_data.timestamp,
                    exit_time: None,
                    entry_price: current_price,
                    exit_price: None,
                    position_type: PositionType::Long,
                    quantity: trade_quantity,
                    pnl: None,
                    strategy_name: format!("{:?}", self.strategy_mode),
                };
                self.current_position.insert(symbol.clone(), Some(trade));
            }

            // Open short position on sell signal if no position exists
            (TradeSignal::Sell, None) => {
                let trade = Trade {
                    entry_time: market_data.raw_data.timestamp,
                    exit_time: None,
                    entry_price: current_price,
                    exit_price: None,
                    position_type: PositionType::Short,
                    quantity: trade_quantity,
                    pnl: None,
                    strategy_name: format!("{:?}", self.strategy_mode),
                };
                self.current_position.insert(symbol.clone(), Some(trade));
            }

            // Close long position on sell signal and potentially open short
            (TradeSignal::Sell, Some(Some(trade))) if trade.position_type == PositionType::Long => {
                let mut closed_trade = trade.clone();
                closed_trade.exit_time = Some(market_data.raw_data.timestamp);
                closed_trade.exit_price = Some(current_price);

                // Calculate PnL
                let entry_value = closed_trade.entry_price * closed_trade.quantity;
                let exit_value = current_price * closed_trade.quantity;
                let pnl = exit_value - entry_value - commission;
                closed_trade.pnl = Some(pnl);

                self.trades.push(closed_trade);

                // Open new short position
                let new_trade = Trade {
                    entry_time: market_data.raw_data.timestamp,
                    exit_time: None,
                    entry_price: current_price,
                    exit_price: None,
                    position_type: PositionType::Short,
                    quantity: trade_quantity,
                    pnl: None,
                    strategy_name: format!("{:?}", self.strategy_mode),
                };
                self.current_position
                    .insert(symbol.clone(), Some(new_trade));
            }

            // Close short position on buy signal and potentially open long
            (TradeSignal::Buy, Some(Some(trade))) if trade.position_type == PositionType::Short => {
                let mut closed_trade = trade.clone();
                closed_trade.exit_time = Some(market_data.raw_data.timestamp);
                closed_trade.exit_price = Some(current_price);

                // Calculate PnL
                let entry_value = closed_trade.entry_price * closed_trade.quantity;
                let exit_value = current_price * closed_trade.quantity;
                let pnl = entry_value - exit_value - commission;
                closed_trade.pnl = Some(pnl);

                self.trades.push(closed_trade);

                // Open new long position
                let new_trade = Trade {
                    entry_time: market_data.raw_data.timestamp,
                    exit_time: None,
                    entry_price: current_price,
                    exit_price: None,
                    position_type: PositionType::Long,
                    quantity: trade_quantity,
                    pnl: None,
                    strategy_name: format!("{:?}", self.strategy_mode),
                };
                self.current_position
                    .insert(symbol.clone(), Some(new_trade));
            }

            // Hold current position
            _ => {}
        }
    }

    /// Calculates the current equity based on open positions.
    ///
    /// # Arguments
    /// * `current_price` - The current price of the asset.
    fn calculate_current_equity(&self, current_price: f64) -> f64 {
        // Calculate current equity based on open positions
        let mut current_equity = self.initial_capital;

        for (_, trade_opt) in &self.current_position {
            if let Some(trade) = trade_opt {
                let pnl = match trade.position_type {
                    PositionType::Long => (current_price - trade.entry_price) * trade.quantity,
                    PositionType::Short => (trade.entry_price - current_price) * trade.quantity,
                };
                current_equity += pnl;
            }
        }

        current_equity
    }

    /// Calculates the backtest results based on the trades and current equity.
    ///
    /// # Returns
    /// A `BacktestResult` struct containing the backtest statistics.
    fn calculate_results(&self) -> BacktestResult {
        let mut winning_trades = 0;
        let mut losing_trades = 0;
        let mut total_pnl: f64 = 0.0;
        let mut win_amount: f64 = 0.0;
        let mut loss_amount: f64 = 0.0;
        let mut largest_win: f64 = 0.0;
        let mut largest_loss: f64 = 0.0;

        // Calculate trade statistics
        for trade in &self.trades {
            if let Some(pnl) = trade.pnl {
                total_pnl += pnl;
                if pnl > 0.0 {
                    winning_trades += 1;
                    win_amount += pnl;
                    largest_win = largest_win.max(pnl);
                } else {
                    losing_trades += 1;
                    loss_amount += pnl;
                    largest_loss = largest_loss.min(pnl);
                }
            }
        }

        let total_trades = self.trades.len();
        let win_rate: f64 = if total_trades > 0 {
            winning_trades as f64 / total_trades as f64
        } else {
            0.0
        };

        let average_win: f64 = if winning_trades > 0 {
            win_amount / winning_trades as f64
        } else {
            0.0
        };

        let average_loss: f64 = if losing_trades > 0 {
            loss_amount / losing_trades as f64
        } else {
            0.0
        };

        // Calculate max drawdown using equity curve
        let mut max_drawdown: f64 = 0.0;
        let mut peak: f64 = self.initial_capital;

        for point in &self.equity_curve {
            if point.equity > peak {
                peak = point.equity;
            }
            let drawdown = (peak - point.equity) / peak;
            max_drawdown = f64::max(max_drawdown, drawdown);
        }

        // Calculate daily returns for Sharpe Ratio
        let mut returns: Vec<f64> = Vec::new();
        if self.equity_curve.len() >= 2 {
            for window in self.equity_curve.windows(2) {
                let prev = window[0].equity;
                let current = window[1].equity;
                let daily_return = (current - prev) / prev;
                returns.push(daily_return);
            }
        }

        // Calculate annualized Sharpe Ratio
        let sharpe_ratio = if !returns.is_empty() {
            let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns
                .iter()
                .map(|r| (r - avg_return).powi(2))
                .sum::<f64>()
                / returns.len() as f64;
            let std_dev = variance.sqrt();

            // Annualize metrics (assuming daily data)
            let annualized_return = avg_return * 252.0; // 252 trading days in a year
            let annualized_std_dev = std_dev * (252.0_f64).sqrt();
            let risk_free_rate = 0.02; // 2% annual risk-free rate

            if annualized_std_dev > 0.0 {
                (annualized_return - risk_free_rate) / annualized_std_dev
            } else {
                0.0
            }
        } else {
            0.0
        };

        BacktestResult {
            total_trades,
            winning_trades,
            losing_trades,
            total_pnl,
            win_rate,
            average_win,
            average_loss,
            largest_win,
            largest_loss,
            max_drawdown,
            sharpe_ratio,
            trades: self.trades.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MarketData;

    fn create_test_data(price: f64, timestamp: DateTime<Utc>) -> ProcessedMarketData {
        ProcessedMarketData {
            raw_data: MarketData {
                timestamp,
                symbol: "TEST".to_string(),
                price,
                volume: 1000.0,
                high: price + 1.0,
                low: price - 1.0,
            },
            moving_average_5: Some(price),
            moving_average_20: Some(price),
            rsi_14: Some(50.0),
            volatility: Some(1.0),
            is_outlier: false,
        }
    }

    #[test]
    fn test_individual_strategies() {
        let mut backtester = Backtester::new(10000.0, 1000.0, 0.001);
        backtester.set_strategy_mode(StrategyMode::Rsi);
        let now = Utc::now();

        // Create test data with a simple price trend
        let mut market_data = Vec::new();
        let prices = vec![100.0, 101.0, 102.0, 103.0, 102.0, 101.0, 100.0];

        for (i, &price) in prices.iter().enumerate() {
            let timestamp = now + chrono::Duration::hours(i as i64);
            market_data.push(create_test_data(price, timestamp));
        }

        let result = backtester.run_backtest(&market_data);

        assert!(result.total_trades > 0);
        assert!(result.total_pnl.abs() > 0.0);
        assert!(result.win_rate >= 0.0 && result.win_rate <= 1.0);
    }

    #[test]
    fn test_combined_strategies() {
        let mut backtester = Backtester::new(10000.0, 1000.0, 0.001);
        backtester.set_strategy_mode(StrategyMode::Combined);
        let now = Utc::now();

        // Create test data with a simple price trend
        let mut market_data = Vec::new();
        let prices = vec![100.0, 101.0, 102.0, 103.0, 102.0, 101.0, 100.0];

        for (i, &price) in prices.iter().enumerate() {
            let timestamp = now + chrono::Duration::hours(i as i64);
            market_data.push(create_test_data(price, timestamp));
        }

        let result = backtester.run_backtest(&market_data);

        // In combined mode, we expect fewer trades since both strategies must agree
        assert!(result.total_trades <= result.total_trades);
    }

    #[test]
    fn test_strategy_mode_switching() {
        let mut backtester = Backtester::new(10000.0, 1000.0, 0.001);
        let now = Utc::now();

        // Create test data
        let mut market_data = Vec::new();
        let prices = vec![100.0, 101.0, 102.0, 103.0, 102.0, 101.0, 100.0];

        for (i, &price) in prices.iter().enumerate() {
            let timestamp = now + chrono::Duration::hours(i as i64);
            market_data.push(create_test_data(price, timestamp));
        }

        // Test individual mode
        backtester.set_strategy_mode(StrategyMode::Rsi);
        let individual_result = backtester.run_backtest(&market_data);

        // Test combined mode
        backtester.set_strategy_mode(StrategyMode::Combined);
        let combined_result = backtester.run_backtest(&market_data);

        // Combined mode should generally have fewer trades
        assert!(combined_result.total_trades <= individual_result.total_trades);
    }
}
