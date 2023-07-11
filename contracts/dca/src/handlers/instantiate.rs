#![allow(unused)]
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};

use crate::contract::{AppResult, DCAApp};
use crate::msg::AppInstantiateMsg;
use crate::state::{Config, CONFIG, NEXT_DCA_ID};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: DCAApp,
    // QUEST #3.3
    // Replace this with the custom instantiate message type and set the `Config` object with it.
    msg: Empty,
) -> AppResult {
    let config: Config = Config::default();

    CONFIG.save(deps.storage, &config)?;
    NEXT_DCA_ID.save(deps.storage, &0)?;
    // Example instantiation that doesn't do anything
    Ok(Response::new())
}
