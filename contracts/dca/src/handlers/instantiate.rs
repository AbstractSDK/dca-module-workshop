use abstract_app::abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{Addr, DepsMut, Empty, Env, MessageInfo, Response};
use cw_asset::AssetInfoBase;

use crate::{
    contract::{AppResult, DCAApp},
    error::DCAError,
    state::{Config, CONFIG, NEXT_ID},
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: DCAApp,
    // QUEST #3.3
    // Replace this with the custom instantiate message type and set the `Config` object with it.
    msg: Empty,
) -> AppResult {
    let name_service = app.name_service(deps.as_ref());
    let asset: AssetInfoBase<Addr>  = todo!("Query the ANS with the msg's native_asset to get the denom.");
    let native_denom = match asset {
        AssetInfoBase::Native(denom) => denom,
        _ => return Err(DCAError::NotNativeAsset {}),
    };

    let config: Config = todo!();

    CONFIG.save(deps.storage, &config)?;
    NEXT_ID.save(deps.storage, &Default::default())?;

    Ok(Response::new())
}
