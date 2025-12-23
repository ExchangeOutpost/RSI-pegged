use exchange_outpost_abi::FunctionArgs;
use extism_pdk::{FnResult, Json, ToBytes, encoding, plugin_fn};
use serde::Serialize;

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
pub struct Output {}

#[plugin_fn]
pub fn run(call_args: FunctionArgs) -> FnResult<Output> {
    Ok(Output {})
}
