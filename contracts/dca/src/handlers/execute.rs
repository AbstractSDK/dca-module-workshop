#![allow(clippy::too_many_arguments)]
#![allow(unused)]

use abstract_core::objects::{AssetEntry, DexName};
use abstract_dex_adapter::msg::OfferAsset;
use abstract_sdk::features::AbstractResponse;
use cosmwasm_std::{
    wasm_execute, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128,
};
use cw_asset::{Asset, AssetList, AssetListUnchecked};

use crate::contract::{AppResult, DCAApp};

use crate::error::AppError;
use crate::msg::{DCAExecuteMsg, ExecuteMsg, Frequency};
use crate::state::{Config, DCAEntry, CONFIG, DCA_LIST, NEXT_ID};
use abstract_dex_adapter::api::DexInterface;
use abstract_sdk::AbstractSdkResult;
use croncat_app::croncat_integration_utils::{CronCatAction, CronCatTaskRequest};
use croncat_app::{CronCat, CronCatInterface};

/// Helper to attach funds for creation or refilling a task
fn task_creation_assets(config: &Config) -> AssetListUnchecked {
    AssetList::from(vec![Asset::native(
        config.native_denom.clone(),
        config.dca_creation_amount,
    )])
    .into()
}

/// Helper to for task creation message
fn create_convert_task_internal(
    env: Env,
    dca: DCAEntry,
    dca_id: String,
    cron_cat: CronCat<DCAApp>,
    config: Config,
) -> AbstractSdkResult<CosmosMsg> {
    let interval = dca.frequency.to_interval();
    let task = CronCatTaskRequest {
        interval,
        boundary: None,
        stop_on_fail: true,
        actions: vec![CronCatAction {
            // Cron cat action that will get scheduled
            // #3.1
            msg: wasm_execute(
                // We want croncat to call this dca contract and execute specific method
                "",
                // Note: ExecuteMsg::from(DCAExecuteMsg)
                // More info: https://docs.abstract.money/4_get_started/3_module_builder.html#execute
                &cosmwasm_std::Empty {},
                vec![],
            )?
            .into(),
            gas_limit: Some(300_000),
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let assets = task_creation_assets(&config);
    // Generate create task message
    // # 3.2
    // Hint: Use Cron Cat Api
    Ok(CosmosMsg::Custom(cosmwasm_std::Empty {}))
}

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    msg: DCAExecuteMsg,
) -> AppResult {
    match msg {
        DCAExecuteMsg::UpdateConfig {
            new_native_denom,
            new_dca_creation_amount,
            new_refill_threshold,
            new_max_spread,
        } => update_config(
            deps,
            info,
            app,
            new_native_denom,
            new_dca_creation_amount,
            new_refill_threshold,
            new_max_spread,
        ),
        DCAExecuteMsg::CreateDCA {
            source_asset,
            target_asset,
            frequency,
            dex,
        } => create_dca(
            deps,
            env,
            info,
            app,
            source_asset,
            target_asset,
            frequency,
            dex,
        ),
        DCAExecuteMsg::UpdateDCA {
            dca_id,
            new_source_asset,
            new_target_asset,
            new_frequency,
            new_dex,
        } => update_dca(
            deps,
            env,
            info,
            app,
            dca_id,
            new_source_asset,
            new_target_asset,
            new_frequency,
            new_dex,
        ),
        DCAExecuteMsg::CancelDCA { dca_id } => cancel_dca(deps, info, app, dca_id),
        DCAExecuteMsg::Convert { dca_id } => convert(deps, env, info, app, dca_id),
    }
}

/// Update the configuration of the app
fn update_config(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: DCAApp,
    new_native_denom: Option<String>,
    new_dca_creation_amount: Option<Uint128>,
    new_refill_threshold: Option<Uint128>,
    new_max_spread: Option<Decimal>,
) -> AppResult {
    // Only the admin should be able to call this
    // #1
    // Hint: https://docs.rs/abstract-app/0.17.0/abstract_app/state/struct.AppContract.html#

    let old_config = CONFIG.load(deps.storage)?;

    CONFIG.save(
        deps.storage,
        &Config {
            native_denom: new_native_denom.unwrap_or(old_config.native_denom),
            dca_creation_amount: new_dca_creation_amount.unwrap_or(old_config.dca_creation_amount),
            refill_threshold: new_refill_threshold.unwrap_or(old_config.refill_threshold),
            max_spread: new_max_spread.unwrap_or(old_config.max_spread),
        },
    )?;

    Ok(app.tag_response(Response::default(), "update_config"))
}

/// Create new DCA
fn create_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    source_asset: OfferAsset,
    target_asset: AssetEntry,
    frequency: Frequency,
    dex_name: DexName,
) -> AppResult {
    // Only the admin should be able to create dca
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let _config = CONFIG.load(deps.storage)?;

    // Simulate swap first
    // #2
    // Using the DEX API
    // What is an API: https://docs.abstract.money/4_get_started/4_sdk.html#apis
    // The Dex API: https://github.com/AbstractSDK/abstract/blob/main/modules/contracts/adapters/dex/src/api.rs

    // Generate DCA ID
    let id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    let dca_id = format!("dca_{id}");

    let dca_entry = DCAEntry {
        source_asset,
        target_asset,
        frequency,
        dex: dex_name,
    };
    DCA_LIST.save(deps.storage, dca_id.clone(), &dca_entry)?;

    // Generate task message
    // #3
    // Hint: use one of the helpers
    // And you are going to need the Cron Cat API: https://github.com/AbstractSDK/abstract/blob/main/modules/contracts/apps/croncat/src/api.rs
    let task_msg = CosmosMsg::Custom(cosmwasm_std::Empty {});

    Ok(app.tag_response(
        Response::new()
            .add_message(task_msg)
            .add_attribute("dca_id", dca_id),
        "create_dca",
    ))
}

/// Update existing dca
fn update_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    dca_id: String,
    new_source_asset: Option<OfferAsset>,
    new_target_asset: Option<AssetEntry>,
    new_frequency: Option<Frequency>,
    new_dex: Option<DexName>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Only if frequency is changed we have to re-create a task
    let recreate_task = new_frequency.is_some();

    let old_dca = DCA_LIST.load(deps.storage, dca_id.clone())?;
    let new_dca = DCAEntry {
        source_asset: new_source_asset.unwrap_or(old_dca.source_asset),
        target_asset: new_target_asset.unwrap_or(old_dca.target_asset),
        frequency: new_frequency.unwrap_or(old_dca.frequency),
        dex: new_dex.unwrap_or(old_dca.dex),
    };

    // Simulate swap for a new dca
    // #2 (re)

    DCA_LIST.save(deps.storage, dca_id.clone(), &new_dca)?;

    let response = if recreate_task {
        // Remove and create task!
        // #3.4
        // Hint: using Cron Cat API
        let remove_task_msg = CosmosMsg::Custom(cosmwasm_std::Empty {});
        let create_task_msg = CosmosMsg::Custom(cosmwasm_std::Empty {});
        Response::new().add_messages(vec![remove_task_msg, create_task_msg])
    } else {
        Response::new()
    };
    Ok(app.tag_response(response, "update_dca"))
}

/// Remove existing dca, remove task from cron_cat
fn cancel_dca(deps: DepsMut, info: MessageInfo, app: DCAApp, dca_id: String) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    DCA_LIST.remove(deps.storage, dca_id.clone());

    // Remove task from Cron Cat
    // #3.3
    // Hint: Using the Cron Cat API
    let remove_task_msg = CosmosMsg::Custom(cosmwasm_std::Empty {});

    Ok(app.tag_response(Response::new().add_message(remove_task_msg), "cancel_dca"))
}

/// Execute swap if called my croncat manager
/// Refill task if needed
fn convert(deps: DepsMut, _env: Env, info: MessageInfo, app: DCAApp, dca_id: String) -> AppResult {
    let config = CONFIG.load(deps.storage)?;
    let dca = DCA_LIST.load(deps.storage, dca_id.clone())?;

    // Check if this method called by Cron Cat Manager
    // #4
    // Hint: Cron Cat API has method to get manager addr
    let manager_addr = cosmwasm_std::Addr::unchecked("");
    if manager_addr != info.sender {
        return Err(AppError::NotManagerConvert {});
    }
    let mut messages = vec![];

    // In case task running out of balance - refill it
    // #5
    // If task balance is below config threshold - refill it
    // Hint: You can get task balance and refill using Cron Cat API :)
    // Don't be shy to unwrap here, since this method is called
    // by Cron Cat of this DCA task and it is not possible that there is no balance!
    let task_balance = Uint128::new(1);
    if task_balance < config.refill_threshold {
        let assets = task_creation_assets(&config);
        let refill_task_msg = CosmosMsg::Custom(cosmwasm_std::Empty {});
        messages.push(refill_task_msg)
    }

    // Finally do the swap!
    // #6
    // Hint: Using Dex API
    // After you done - remove allow(unused) to make sure you used everything, and run a test
    messages.push(CosmosMsg::Custom(cosmwasm_std::Empty {}));
    Ok(app.tag_response(Response::new().add_messages(messages), "convert"))
}
