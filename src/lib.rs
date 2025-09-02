mod exchange_outpost;
use crate::exchange_outpost::FinData;
use extism_pdk::{FnResult, Json, ToBytes, encoding, plugin_fn};
use serde::Serialize;

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    ticker: String,
    period: usize,
}

#[plugin_fn]
pub fn run(fin_data: FinData) -> FnResult<Output> {
    let ticker = fin_data.get_ticker("pegged_data")?;
    let period = fin_data.get_call_argument::<usize>("period")?;

    Ok(Output {
        ticker: ticker.symbol.clone(),
        period,
    })
}
