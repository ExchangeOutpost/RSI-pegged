use exchange_outpost_abi::{FunctionArgs, schedule_email};
use extism_pdk::{FnResult, Json, ToBytes, encoding, plugin_fn};
use serde::Serialize;
use ta::{Next, indicators::RelativeStrengthIndex};

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {
    ticker: String,
    rsi: f64,
    direction: f64,
    email_sent: bool,
}

#[plugin_fn]
pub fn run(call_args: FunctionArgs) -> FnResult<Output> {
    let ticker = call_args.get_ticker("pegged_data")?;
    let period = call_args.get_call_argument::<usize>("period").unwrap_or(14);
    let rsi_low = call_args
        .get_call_argument::<f64>("rsi_low")
        .unwrap_or(20.0);
    let rsi_high = call_args
        .get_call_argument::<f64>("rsi_high")
        .unwrap_or(70.0);
    let email = call_args
        .get_call_argument::<String>("email")
        .unwrap_or("".to_string());
    let mut rsi = RelativeStrengthIndex::new(period).unwrap();
    let mut last = 0.0;
    let mut email_sent = false;

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

    if direction != 0.0 && !email.is_empty() {
        let message = format!(
            "RSI alert for {}: RSI value is {:.2}, direction is {}.",
            ticker.symbol, last, direction
        );
        schedule_email(&email, message.as_str())?;
        email_sent = true;
    }

    Ok(Output {
        ticker: ticker.symbol.clone(),
        rsi: last,
        direction,
        email_sent,
    })
}
