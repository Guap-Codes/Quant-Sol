# Quant-Sol: Algorithmic Solana Trading Strategy Backtester

## Overview

Quant-Sol is a sophisticated Rust-based algorithmic trading strategy backtester specifically designed for Solana (SOL) cryptocurrency trading. The project implements and compares multiple technical analysis strategies to provide robust trading insights.

## 🚀 Key Features

### Trading Strategies
- **RSI (Relative Strength Index)**: Momentum-based oscillator strategy
- **Bollinger Bands**: Volatility and mean reversion strategy
- **Combined Strategy**: Confirmation-based trading approach

### Advanced Analytics
- Comprehensive backtesting framework
- Detailed performance metrics:
  - Win Rate
  - Total Profit/Loss (PnL)
  - Sharpe Ratio
  - Maximum Drawdown
  - Trade-level statistics

### Risk Management
- Position sizing
- Commission calculation
- Flexible strategy mode selection

## 🛠 Technical Stack
- Language: Rust
- Data Source: Alpha Vantage API
- Cryptocurrency: Solana (SOL)

## 📊 Performance Metrics Explained

### Sharpe Ratio
- Measures risk-adjusted return
- Higher is better
- Accounts for both returns and volatility

### Maximum Drawdown
- Represents the largest peak-to-trough decline
- Indicates potential risk in the strategy

## 🔧 Configuration

### Environment Setup
1. Clone the repository
  ```bash
  git clone https://github.com/Guap-Codes/Quant-Sol.git
  ```
2. Create a `.env` file with:
   ```
   ALPHA_VANTAGE_API_KEY=your_api_key_here
   ```
3. Install Rust: https://rustup.rs/

### Running Backtests
```bash
cargo run
```

## 🔬 Strategy Modes
- **RSI Mode**: Pure RSI strategy
- **Bollinger Bands Mode**: Pure Bollinger Bands strategy
- **Combined Mode**: Strategies must confirm each other

## 🚧 Roadmap
- [ ] Add more trading strategies
- [ ] Implement live trading capabilities
- [ ] Enhance machine learning strategy selection
- [ ] Add more cryptocurrency support

## 📈 Example Output
```
RSI Strategy Results:
Total Trades: 13
Win Rate: 46.15%
Total PnL: $141.12
Sharpe Ratio: -0.36
Max Drawdown: 2.28%
Average Win: $63.79
Average Loss: $-34.52
Largest Win: $147.57
Largest Loss: $-103.61

Bollinger Bands Strategy Results:
Total Trades: 4
Win Rate: 75.00%
Total PnL: $191.85
Sharpe Ratio: -0.28
Max Drawdown: 4.17%
Average Win: $115.63
Average Loss: $-155.04
Largest Win: $134.20
Largest Loss: $-155.04
```

## 🤝 Contributing
1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## ⚠️ Disclaimer
Trading cryptocurrencies involves significant risk. This tool is for educational purposes only. Always do your own research and consult financial advisors.

## 📄 License
MIT License
