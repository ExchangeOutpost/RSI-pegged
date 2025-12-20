mod exchange_outpost;
use crate::exchange_outpost::{Candle, FinData, schedule_email};
use extism_pdk::{FnResult, Json, ToBytes, plugin_fn};
use serde::Serialize;

#[derive(Serialize, ToBytes, Clone, Copy, PartialEq)]
#[encoding(Json)]
pub enum Signal {
    Long,
    Short,
    Hold,
}

#[derive(Serialize, ToBytes, Clone, Copy, PartialEq)]
#[encoding(Json)]
pub enum TrendDirection {
    Bullish,
    Bearish,
    Neutral,
}

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    ticker: String,
    signal: Signal,
    trend: TrendDirection,
    supertrend_value: f64,
    upper_band: f64,
    lower_band: f64,
    atr: f64,
    ema_50: f64,
    ema_200: f64,
    current_price: f64,
    stop_loss: f64,
    volume_confirmed: bool,
    email_sent: bool,
}

/// Calculate True Range for a candle
fn true_range(candle: &Candle<f64>, prev_close: f64) -> f64 {
    let high_low = candle.high - candle.low;
    let high_prev_close = (candle.high - prev_close).abs();
    let low_prev_close = (candle.low - prev_close).abs();
    high_low.max(high_prev_close).max(low_prev_close)
}

/// Calculate Average True Range (ATR) using Wilder's smoothing method
fn calculate_atr(candles: &[Candle<f64>], period: usize) -> Vec<f64> {
    if candles.len() < period + 1 {
        return vec![];
    }

    let mut atr_values = vec![0.0; candles.len()];

    // Calculate initial TR values
    let mut tr_values = Vec::with_capacity(candles.len());
    tr_values.push(candles[0].high - candles[0].low); // First TR

    for i in 1..candles.len() {
        tr_values.push(true_range(&candles[i], candles[i - 1].close));
    }

    // Calculate initial ATR as simple average of first 'period' TR values
    let initial_atr: f64 = tr_values[1..=period].iter().sum::<f64>() / period as f64;
    atr_values[period] = initial_atr;

    // Calculate subsequent ATR values using Wilder's smoothing
    // ATR = [(Prior ATR × (period-1)) + Current TR] / period
    for i in (period + 1)..candles.len() {
        atr_values[i] = (atr_values[i - 1] * (period - 1) as f64 + tr_values[i]) / period as f64;
    }

    atr_values
}

/// Calculate Exponential Moving Average (EMA)
fn calculate_ema(candles: &[Candle<f64>], period: usize) -> Vec<f64> {
    if candles.len() < period {
        return vec![0.0; candles.len()];
    }

    let mut ema_values = vec![0.0; candles.len()];
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initial EMA is SMA of first 'period' values
    let initial_sma: f64 = candles[..period].iter().map(|c| c.close).sum::<f64>() / period as f64;
    ema_values[period - 1] = initial_sma;

    // Calculate subsequent EMA values
    for i in period..candles.len() {
        ema_values[i] = (candles[i].close - ema_values[i - 1]) * multiplier + ema_values[i - 1];
    }

    ema_values
}

/// Find pivot highs within a lookback window
fn find_pivot_high(candles: &[Candle<f64>], index: usize, lookback: usize) -> Option<f64> {
    if index < lookback || index + lookback >= candles.len() {
        return None;
    }

    let current_high = candles[index].high;

    // Check if current high is highest in the lookback window
    for i in (index - lookback)..index {
        if candles[i].high >= current_high {
            return None;
        }
    }
    for i in (index + 1)..=(index + lookback) {
        if candles[i].high > current_high {
            return None;
        }
    }

    Some(current_high)
}

/// Find pivot lows within a lookback window
fn find_pivot_low(candles: &[Candle<f64>], index: usize, lookback: usize) -> Option<f64> {
    if index < lookback || index + lookback >= candles.len() {
        return None;
    }

    let current_low = candles[index].low;

    // Check if current low is lowest in the lookback window
    for i in (index - lookback)..index {
        if candles[i].low <= current_low {
            return None;
        }
    }
    for i in (index + 1)..=(index + lookback) {
        if candles[i].low < current_low {
            return None;
        }
    }

    Some(current_low)
}

/// Calculate pivot point center line based on recent pivot highs and lows
fn calculate_pivot_center(candles: &[Candle<f64>], lookback: usize, max_pivots: usize) -> f64 {
    let mut pivot_highs = Vec::new();
    let mut pivot_lows = Vec::new();

    // Find pivot points (excluding the last lookback candles as they can't be confirmed)
    let end_index = if candles.len() > lookback {
        candles.len() - lookback
    } else {
        return 0.0;
    };

    for i in lookback..end_index {
        if let Some(ph) = find_pivot_high(candles, i, lookback) {
            pivot_highs.push(ph);
        }
        if let Some(pl) = find_pivot_low(candles, i, lookback) {
            pivot_lows.push(pl);
        }
    }

    // Take the most recent pivots
    let recent_highs: Vec<f64> = pivot_highs.into_iter().rev().take(max_pivots).collect();
    let recent_lows: Vec<f64> = pivot_lows.into_iter().rev().take(max_pivots).collect();

    if recent_highs.is_empty() && recent_lows.is_empty() {
        // Fallback to typical price
        let last = &candles[candles.len() - 1];
        return (last.high + last.low + last.close) / 3.0;
    }

    let avg_high = if recent_highs.is_empty() {
        0.0
    } else {
        recent_highs.iter().sum::<f64>() / recent_highs.len() as f64
    };

    let avg_low = if recent_lows.is_empty() {
        0.0
    } else {
        recent_lows.iter().sum::<f64>() / recent_lows.len() as f64
    };

    if avg_high > 0.0 && avg_low > 0.0 {
        (avg_high + avg_low) / 2.0
    } else if avg_high > 0.0 {
        avg_high
    } else {
        avg_low
    }
}

/// SuperTrend calculation result
struct SuperTrendResult {
    supertrend: f64,
    upper_band: f64,
    lower_band: f64,
    trend: TrendDirection,
}

/// Calculate SuperTrend indicator
fn calculate_supertrend(
    candles: &[Candle<f64>],
    atr_values: &[f64],
    multiplier: f64,
    pivot_lookback: usize,
) -> SuperTrendResult {
    if candles.is_empty() {
        return SuperTrendResult {
            supertrend: 0.0,
            upper_band: 0.0,
            lower_band: 0.0,
            trend: TrendDirection::Neutral,
        };
    }

    let n = candles.len();

    // Use pivot-based center line or HL/2
    let center = calculate_pivot_center(candles, pivot_lookback, 5);
    let hl2 = (candles[n - 1].high + candles[n - 1].low) / 2.0;

    // Blend pivot center with current HL/2 for dynamic centerline
    let dynamic_center = if center > 0.0 {
        (center + hl2) / 2.0
    } else {
        hl2
    };

    let current_atr = atr_values[n - 1];

    // Calculate basic bands
    let basic_upper = dynamic_center + (multiplier * current_atr);
    let basic_lower = dynamic_center - (multiplier * current_atr);

    // For proper SuperTrend, we need to track the bands over time
    // Here we calculate final bands based on recent history
    let mut final_upper = basic_upper;
    let mut final_lower = basic_lower;
    let mut supertrend = basic_lower;
    let mut trend = TrendDirection::Bullish;

    // Look back to determine trend direction
    if n >= 2 {
        let prev_close = candles[n - 2].close;
        let curr_close = candles[n - 1].close;

        // Determine trend based on price position relative to bands
        if curr_close > basic_upper {
            trend = TrendDirection::Bullish;
            supertrend = final_lower;
        } else if curr_close < basic_lower {
            trend = TrendDirection::Bearish;
            supertrend = final_upper;
        } else {
            // Check previous trend
            let prev_hl2 = (candles[n - 2].high + candles[n - 2].low) / 2.0;
            let prev_atr = if n >= 2 {
                atr_values[n - 2]
            } else {
                current_atr
            };
            let prev_upper = prev_hl2 + (multiplier * prev_atr);
            let prev_lower = prev_hl2 - (multiplier * prev_atr);

            if prev_close > prev_lower {
                // Was bullish
                final_lower = final_lower.max(prev_lower);
                if curr_close > final_lower {
                    trend = TrendDirection::Bullish;
                    supertrend = final_lower;
                } else {
                    trend = TrendDirection::Bearish;
                    supertrend = final_upper;
                }
            } else {
                // Was bearish
                final_upper = final_upper.min(prev_upper);
                if curr_close < final_upper {
                    trend = TrendDirection::Bearish;
                    supertrend = final_upper;
                } else {
                    trend = TrendDirection::Bullish;
                    supertrend = final_lower;
                }
            }
        }
    }

    SuperTrendResult {
        supertrend,
        upper_band: final_upper,
        lower_band: final_lower,
        trend,
    }
}

/// Check if volume is elevated compared to average
fn is_volume_elevated(candles: &[Candle<f64>], period: usize, threshold: f64) -> bool {
    if candles.len() < period {
        return false;
    }

    let n = candles.len();
    let current_volume = candles[n - 1].volume;

    let avg_volume: f64 = candles[(n - period)..n]
        .iter()
        .map(|c| c.volume)
        .sum::<f64>()
        / period as f64;

    current_volume > avg_volume * threshold
}

/// Find the last swing high for stop loss calculation
fn find_last_swing_high(candles: &[Candle<f64>], lookback: usize) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }

    let n = candles.len();
    let search_range = lookback.min(n);

    candles[(n - search_range)..n]
        .iter()
        .map(|c| c.high)
        .fold(f64::MIN, f64::max)
}

/// Find the last swing low for stop loss calculation
fn find_last_swing_low(candles: &[Candle<f64>], lookback: usize) -> f64 {
    if candles.is_empty() {
        return 0.0;
    }

    let n = candles.len();
    let search_range = lookback.min(n);

    candles[(n - search_range)..n]
        .iter()
        .map(|c| c.low)
        .fold(f64::MAX, f64::min)
}

#[plugin_fn]
pub fn run(fin_data: FinData) -> FnResult<Output> {
    let ticker = fin_data.get_ticker("symbol_data")?;
    let candles = &ticker.candles;

    // Get configurable parameters with defaults
    let atr_period = fin_data
        .get_call_argument::<usize>("atr_period")
        .unwrap_or(14);
    let multiplier = fin_data
        .get_call_argument::<f64>("multiplier")
        .unwrap_or(2.0);
    let pivot_lookback = fin_data
        .get_call_argument::<usize>("pivot_lookback")
        .unwrap_or(5);
    let volume_period = fin_data
        .get_call_argument::<usize>("volume_period")
        .unwrap_or(20);
    let volume_threshold = fin_data
        .get_call_argument::<f64>("volume_threshold")
        .unwrap_or(1.5);
    let swing_lookback = fin_data
        .get_call_argument::<usize>("swing_lookback")
        .unwrap_or(20);
    let email = fin_data
        .get_call_argument::<String>("email")
        .unwrap_or_default();

    // Ensure we have enough data
    let min_periods = atr_period.max(200).max(pivot_lookback * 2);
    if candles.len() < min_periods {
        return Ok(Output {
            ticker: ticker.symbol.clone(),
            signal: Signal::Hold,
            trend: TrendDirection::Neutral,
            supertrend_value: 0.0,
            upper_band: 0.0,
            lower_band: 0.0,
            atr: 0.0,
            ema_50: 0.0,
            ema_200: 0.0,
            current_price: if candles.is_empty() {
                0.0
            } else {
                candles[candles.len() - 1].close
            },
            stop_loss: 0.0,
            volume_confirmed: false,
            email_sent: false,
        });
    }

    let n = candles.len();
    let current_candle = &candles[n - 1];
    let current_price = current_candle.close;

    // Calculate indicators
    let atr_values = calculate_atr(candles, atr_period);
    let ema_50_values = calculate_ema(candles, 50);
    let ema_200_values = calculate_ema(candles, 200);

    let current_atr = atr_values[n - 1];
    let ema_50 = ema_50_values[n - 1];
    let ema_200 = ema_200_values[n - 1];

    // Calculate SuperTrend with pivot points
    let supertrend_result = calculate_supertrend(candles, &atr_values, multiplier, pivot_lookback);

    // Check volume confirmation
    let volume_confirmed = is_volume_elevated(candles, volume_period, volume_threshold);

    // Determine EMA trend alignment
    let ema_bullish = ema_50 > ema_200 && current_price > ema_50;
    let ema_bearish = ema_50 < ema_200 && current_price < ema_50;

    // Generate trading signal
    let signal = match supertrend_result.trend {
        TrendDirection::Bullish if ema_bullish && volume_confirmed => Signal::Long,
        TrendDirection::Bearish if ema_bearish && volume_confirmed => Signal::Short,
        _ => Signal::Hold,
    };

    // Calculate stop loss based on signal
    let stop_loss = match signal {
        Signal::Long => find_last_swing_low(candles, swing_lookback),
        Signal::Short => find_last_swing_high(candles, swing_lookback),
        Signal::Hold => 0.0,
    };

    // Send email notification if signal is generated
    let mut email_sent = false;
    if signal != Signal::Hold && !email.is_empty() {
        let signal_str = match signal {
            Signal::Long => "LONG",
            Signal::Short => "SHORT",
            Signal::Hold => "HOLD",
        };
        let message = format!(
            "Pivot Point SuperTrend Alert for {}:\n\
             Signal: {}\n\
             Current Price: {:.4}\n\
             SuperTrend: {:.4}\n\
             Stop Loss: {:.4}\n\
             ATR: {:.4}\n\
             EMA 50: {:.4}\n\
             EMA 200: {:.4}\n\
             Volume Confirmed: {}",
            ticker.symbol,
            signal_str,
            current_price,
            supertrend_result.supertrend,
            stop_loss,
            current_atr,
            ema_50,
            ema_200,
            volume_confirmed
        );
        schedule_email(&email, &message)?;
        email_sent = true;
    }

    Ok(Output {
        ticker: ticker.symbol.clone(),
        signal,
        trend: supertrend_result.trend,
        supertrend_value: supertrend_result.supertrend,
        upper_band: supertrend_result.upper_band,
        lower_band: supertrend_result.lower_band,
        atr: current_atr,
        ema_50,
        ema_200,
        current_price,
        stop_loss,
        volume_confirmed,
        email_sent,
    })
}
