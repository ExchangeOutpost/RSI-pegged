mod exchange_outpost;
use crate::exchange_outpost::FinData;
use extism_pdk::{FnResult, Json, ToBytes, encoding, plugin_fn};
use serde::Serialize;
use ta::{Next, indicators::RelativeStrengthIndex};

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    ticker: String,
    rsi: f64,
    direction: f64,
}

#[plugin_fn]
pub fn run(fin_data: FinData) -> FnResult<Output> {
    let ticker = fin_data.get_ticker("pegged_data")?;
    let period = fin_data.get_call_argument::<usize>("period")?;
    let rsi_low = fin_data.get_call_argument::<f64>("rsi_low").unwrap_or(14.0);
    let rsi_high = fin_data
        .get_call_argument::<f64>("rsi_high")
        .unwrap_or(70.0);
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut last = 0.0;

    for price in ticker.candles.iter() {
        last = rsi.next(price.close);
    }

    let direction = if last < rsi_low {
        -1.0
    } else if last > rsi_high {
        1.0
    } else {
        0.0
    };

    Ok(Output {
        ticker: ticker.symbol.clone(),
        rsi: last,
        direction,
    })
}
