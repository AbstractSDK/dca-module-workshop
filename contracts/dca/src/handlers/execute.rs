#![allow(clippy::too_many_arguments)]

use abstract_app::abstract_core::objects::{AnsAsset, AssetEntry, DexName};
use abstract_app::abstract_sdk::{
    features::{AbstractNameService, AbstractResponse},
    AbstractSdkResult,
};
use abstract_dex_adapter::api::DexInterface;
use cosmwasm_std::{wasm_execute, CosmosMsg, Decimal, DepsMut, Empty, Env, MessageInfo, Response, Uint128};
use croncat_app::{
    croncat_integration_utils::{CronCatAction, CronCatTaskRequest},
    CronCat, CronCatInterface,
};
use cw_asset::{Asset, AssetInfoBase, AssetList};

use crate::{
    contract::{AppResult, DCAApp},
    error::DCAError,
    msg::{DCAExecuteMsg, ExecuteMsg, Frequency},
    state::{Config, DCAEntry, DCAId, CONFIG, DCA_LIST, NEXT_ID},
};

/// Helper to for task creation message
fn create_convert_task_internal(
    env: Env,
    dca: DCAEntry,
    dca_id: DCAId,
    cron_cat: CronCat<DCAApp>,
    config: Config,
) -> AbstractSdkResult<CosmosMsg> {
    let interval = dca.frequency.to_interval();
    let task = CronCatTaskRequest {
        interval,
        boundary: None,
        stop_on_fail: true,
        actions: vec![CronCatAction {
            msg: wasm_execute(
                env.contract.address,
                // QUEST #3.1
                // With the macro from 3.0, we implement the `From` trait that converts our custom message into the top-level message.
                // We want to call this contract's `Convert` method but need to wrap it in the `ExecuteMsg` enum that we generated
                // in the last quest.
                &Empty {},
                vec![],
            )?
            .into(),
            gas_limit: Some(300_000),
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let _assets = AssetList::from(vec![Asset::native(
        config.native_denom,
        config.dca_creation_amount,
    )]);
    // QUEST #2.3
    // Generate create task message
    // Croncat API: https://github.com/AbstractSDK/abstract/blob/main/modules/contracts/apps/croncat/src/api.rs
    todo!()
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
            native_asset,
            new_dca_task_balance,
            task_refill_threshold,
            max_spread,
        } => update_config(
            deps,
            info,
            app,
            native_asset,
            new_dca_task_balance,
            task_refill_threshold,
            max_spread,
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
    _msg_info: MessageInfo,
    app: DCAApp,
    new_native_asset: Option<AssetEntry>,
    new_dca_creation_amount: Option<Uint128>,
    new_refill_threshold: Option<Uint128>,
    new_max_spread: Option<Decimal>,
) -> AppResult {
    // QUEST #1
    // Only the admin should be able to call this so add an admin assertion.
    // Hint: https://docs.rs/abstract-app/0.17.0/abstract_app/state/struct.AppContract.html#

    let old_config = CONFIG.load(deps.storage)?;
    let new_native_denom = new_native_asset
        .map(|asset| {
            let asset = app.name_service(deps.as_ref()).query(&asset)?;
            if let AssetInfoBase::Native(native) = asset {
                Ok(native)
            } else {
                Err(DCAError::NotNativeAsset {})
            }
        })
        .transpose()?;

    CONFIG.save(
        deps.storage,
        &Config {
            native_denom: new_native_denom.unwrap_or(old_config.native_denom),
            dca_creation_amount: new_dca_creation_amount.unwrap_or(old_config.dca_creation_amount),
            refill_threshold: new_refill_threshold.unwrap_or(old_config.refill_threshold),
            max_spread: new_max_spread.unwrap_or(old_config.max_spread),
        },
    )?;

    Ok(app.response("update_config"))
}

/// Create new DCA
fn create_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    source_asset: AnsAsset,
    target_asset: AssetEntry,
    frequency: Frequency,
    dex_name: DexName,
) -> AppResult {
    // Only the admin should be able to create dca
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    // QUEST #2.1
    // Here we want to validate that a swap can be performed between the two assets.
    // We can check this by doing a swap simulation using the DEX API
    // If the simulation fails, we should return an error
    // What is an API: https://docs.abstract.money/4_get_started/4_sdk.html
    // The Dex API: https://github.com/AbstractSDK/abstract/blob/main/modules/contracts/adapters/dex/src/api.rs

    // Generate DCA ID
    let dca_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id.next_id()))?;

    let dca_entry = DCAEntry {
        source_asset,
        target_asset,
        frequency,
        dex: dex_name,
    };
    DCA_LIST.save(deps.storage, dca_id, &dca_entry)?;

    // QUEST #2.0
    // Pass on the Cron Cat API: https://github.com/AbstractSDK/abstract/blob/main/modules/contracts/apps/croncat/src/api.rs
    // to generate the cron task message.

    Ok(app
        .response("create_dca")
        .add_attribute("dca_id", dca_id))
}

/// Update existing dca
fn update_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: DCAApp,
    dca_id: DCAId,
    new_source_asset: Option<AnsAsset>,
    new_target_asset: Option<AssetEntry>,
    new_frequency: Option<Frequency>,
    new_dex: Option<DexName>,
) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // Only if frequency is changed we have to re-create a task
    let recreate_task = new_frequency.is_some();

    let old_dca = DCA_LIST.load(deps.storage, dca_id)?;
    let new_dca = DCAEntry {
        source_asset: new_source_asset.unwrap_or(old_dca.source_asset),
        target_asset: new_target_asset.unwrap_or(old_dca.target_asset),
        frequency: new_frequency.unwrap_or(old_dca.frequency),
        dex: new_dex.unwrap_or(old_dca.dex),
    };

    // QUEST #2.2 (same as 2.1)
    // Here we want to validate that a swap can be performed between the two assets.
    // We can check this by doing a swap simulation using the DEX API
    // If the simulation fails, we should return an error
    // Simulate swap for a new dca

    DCA_LIST.save(deps.storage, dca_id, &new_dca)?;

    let response = app.response("update_dca");
    let response = if recreate_task {
        let config = CONFIG.load(deps.storage)?;
        let cron_cat = app.cron_cat(deps.as_ref());
        let remove_task_msg = cron_cat.remove_task(dca_id)?;
        let create_task_msg = create_convert_task_internal(env, new_dca, dca_id, cron_cat, config)?;
        response.add_messages(vec![remove_task_msg, create_task_msg])
    } else {
        response
    };
    Ok(response)
}

/// Remove existing dca, remove task from cron_cat
fn cancel_dca(deps: DepsMut, info: MessageInfo, app: DCAApp, dca_id: DCAId) -> AppResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    // QUEST #2.4
    // Remove task from Cron Cat
    DCA_LIST.remove(deps.storage, dca_id.clone());

    Ok(app.response("cancel_dca"))
}

/// Execute swap if called my croncat manager
/// Refill task if needed
fn convert(deps: DepsMut, env: Env, info: MessageInfo, app: DCAApp, dca_id: DCAId) -> AppResult {
    let cron_cat = app.cron_cat(deps.as_ref());

    let manager_addr = cron_cat.query_manager_addr(env.contract.address.clone(), dca_id)?;
    if manager_addr != info.sender {
        return Err(DCAError::NotManagerConvert {});
    }

    let config = CONFIG.load(deps.storage)?;
    let dca = DCA_LIST.load(deps.storage, dca_id)?;

    let mut messages = vec![];

    // In case task running out of balance - refill it
    let task_balance = cron_cat
        .query_task_balance(env.contract.address, dca_id)?
        .balance
        .unwrap();
    if task_balance.native_balance < config.refill_threshold {
        messages.push(
            cron_cat.refill_task(
                dca_id,
                AssetList::from(vec![Asset::native(
                    config.native_denom,
                    config.dca_creation_amount,
                )])
                .into(),
            )?,
        );
    }

    // QUEST #2.5
    // Finally do the swap!
    
    Ok(app.response("convert").add_messages(messages))
}
