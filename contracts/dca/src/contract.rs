#![allow(unused)]
use crate::{
    error::AppError,
    handlers,
    msg::{AppInstantiateMsg, DCAExecuteMsg, DCAQueryMsg},
};
use abstract_app::AppContract;
use abstract_core::objects::dependency::StaticDependency;
use abstract_dex_adapter::EXCHANGE;
use cosmwasm_std::{Empty, Response};

/// The version of your app
pub const DCA_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const DCA_APP_ID: &str = "abstract:dca";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// QUEST #3.2
/// The custom instantiate message is set to `Empty` but we want to set some state on instantiation.
/// Replace it with our custom instantiate message type.
pub type DCAApp = AppContract<AppError, Empty, DCAExecuteMsg, DCAQueryMsg, Empty>;

// #0
// This module application is dependent on two other modules: the CronCat and the Dex module.
// Find out how to set the dependency for this module.
// Hint: https://docs.abstract.money/4_get_started/3_module_builder.html?#dependencies
const CRONCAT_DEPENDENCY: StaticDependency = StaticDependency::new("", &["^0.0.1"]);
const DEX_DEPENDENCY: StaticDependency = StaticDependency::new(EXCHANGE, &["^0.17.0"]);

const DCA_APP: DCAApp = DCAApp::new(DCA_APP_ID, DCA_APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(DCA_APP, DCAApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(DCA_APP, DCAApp, DCAApp);
