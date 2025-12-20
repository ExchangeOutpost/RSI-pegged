# Pivot Point SuperTrend Strategy for ExchangeOutpost

A Rust-based implementation of the Pivot Point SuperTrend trading strategy that can be deployed and executed on [ExchangeOutpost](https://www.exchangeoutpost.com/).

## Overview

This project implements a hybrid trading strategy that combines pivot points for identifying support/resistance levels with the SuperTrend indicator for trend direction. It includes EMA confirmation (50/200) and volume validation for breakout and trend-following trading signals.

## Strategy Components

### Pivot Points Calculation
Calculates pivot highs and lows over a specified lookback period. The center line is derived through a weighted average of these pivot points, which serves as the baseline for the SuperTrend calculation.

### SuperTrend Indicator
Uses Average True Range (ATR) and a configurable multiplier to create dynamic bands:
- **Upper Band**: Dynamic Center + (Multiplier × ATR)
- **Lower Band**: Dynamic Center - (Multiplier × ATR)
- **ATR Calculation**: Wilder's smoothing method [(Prior ATR × (period-1)) + Current TR] / period

### Trend Confirmation
Includes 50 EMA and 200 EMA as an additional trend confirmation layer:
- Bullish alignment: Price > EMA 50 > EMA 200
- Bearish alignment: Price < EMA 50 < EMA 200

### Signal Generation

**Long Entry Rules:**
- Price closes above both moving averages
- 50 EMA positioned above 200 EMA
- Pivot Point SuperTrend generates a bullish signal
- Volume confirmation: volume exceeds threshold above average

**Short Entry Rules:**
- Price closes below both moving averages
- 50 EMA positioned below 200 EMA
- Pivot Point SuperTrend generates a bearish signal
- Volume confirmation at signal issuance

## Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `atr_period` | integer | 14 | ATR period for volatility calculation (typically 10-14) |
| `multiplier` | number | 2.0 | ATR multiplier for SuperTrend bands (typically 2-3) |
| `pivot_lookback` | integer | 5 | Lookback period for pivot point calculation |
| `volume_period` | integer | 20 | Period for average volume calculation |
| `volume_threshold` | number | 1.5 | Volume threshold multiplier for confirmation |
| `swing_lookback` | integer | 20 | Lookback period for swing high/low stop loss calculation |
| `email` | string | "" | Email address for signal notifications (optional) |

## Output Structure

The strategy outputs the following data:

```json
{
    "ticker": "SYMBOL",
    "signal": "Long|Short|Hold",
    "trend": "Bullish|Bearish|Neutral",
    "supertrend_value": 100.50,
    "upper_band": 102.30,
    "lower_band": 98.70,
    "atr": 1.80,
    "ema_50": 101.20,
    "ema_200": 99.80,
    "current_price": 101.50,
    "stop_loss": 98.50,
    "volume_confirmed": true,
    "email_sent": false
}
```

## Features

- **WebAssembly Compilation**: Functions compile to WASM for cross-platform execution
- **Financial Data Structures**: Built-in support for candlestick data, ticker information, and market data
- **Dynamic Risk Management**: Automatic stop loss calculation based on recent swing highs/lows
- **Email Notifications**: Optional email alerts when trading signals are generated
- **Volume Confirmation**: Reduces false signals by requiring elevated volume

## Project Structure

```
src/
├── lib.rs                      # Main strategy implementation
└── exchange_outpost/           # Financial data structures and utility functions
```

## Getting Started

### Prerequisites

- Rust 1.70+ with 2024 edition support
- `wasm32-unknown-unknown` target installed

### Installation

1. Clone this repository:
```bash
git clone https://github.com/ExchangeOutpost/RSI-pegged.git
cd RSI-pegged
```

2. Install the WebAssembly target:
```bash
rustup target add wasm32-unknown-unknown
```

3. Build the project:
```bash
cargo build --target wasm32-unknown-unknown --release
```

## Building and Deployment

### Local Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM file will be located at:
`target/wasm32-unknown-unknown/release/rust_function_template.wasm`

### Automated Releases

This project includes GitHub Actions for automated releases. When you push a tag, it will:
1. Build the WASM binary
2. Create a GitHub release
3. Upload the binary as `finfunc.wasm`

To create a release:
```bash
git tag 1.0.0
git push origin 1.0.0
```
Tags must follow [semantic versioning](https://semver.org/).

### Testing Your Function
When pushing to the `master` branch, the CI will automatically build your function and create a preview release named `master`.
You can use this release to test your function on the ExchangeOutpost platform.

## Dependencies

- **extism-pdk** (1.4.1): Plugin development kit for WebAssembly functions
- **rust_decimal** (1.37.2): High-precision decimal arithmetic for financial calculations
- **serde** (1.0.219): Serialization/deserialization framework
- **serde_json** (1.0.143): JSON support for serde

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for more details.

## Related Links

- [ExchangeOutpost Platform](https://www.exchangeoutpost.com/)
- [Extism Documentation](https://extism.org/)
- [Rust WebAssembly Book](https://rustwasm.github.io/docs/book/)
