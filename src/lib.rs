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
}

#[plugin_fn]
pub fn run(fin_data: FinData) -> FnResult<Output> {
    let ticker = fin_data.get_ticker("pegged_data")?;
    let period = fin_data.get_call_argument::<usize>("period")?;
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut last = 0.0;

    for price in ticker.candles.iter() {
        last = rsi.next(price.close);
    }

    Ok(Output {
        ticker: ticker.symbol.clone(),
        rsi: last,
    })
}
