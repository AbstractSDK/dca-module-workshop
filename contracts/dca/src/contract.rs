use abstract_app::abstract_core::objects::dependency::StaticDependency;
use abstract_app::AppContract;
#[cfg(feature = "interface")]
use abstract_app::{
    abstract_core::{manager::ModuleInstallConfig, objects::module::ModuleInfo},
    abstract_interface::{AbstractInterfaceError, DependencyCreation, InstallConfig},
};
use cosmwasm_std::{Empty, Response};
#[cfg(feature = "interface")]
use croncat_app::contract::interface::Croncat;
use croncat_app::contract::{CRONCAT_ID, CRONCAT_MODULE_VERSION};

use crate::{
    error::DCAError,
    handlers,
    msg::{AppInstantiateMsg, DCAExecuteMsg, DCAQueryMsg},
};

/// The version of your app
pub const DCA_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const DCA_APP_ID: &str = "abstract:dca";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, DCAError>;

/// QUEST #3.2
///  The custom instantiate message is set to `Empty` but we want to set some state on instantiation.
/// Replace it with our custom instantiate message type.
/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type DCAApp = AppContract<DCAError, Empty, DCAExecuteMsg, DCAQueryMsg, Empty>;

const DCA_APP: DCAApp = DCAApp::new(DCA_APP_ID, DCA_APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_dependencies(&[
        // QUEST #0
        // This module application is dependent on two other modules: the CronCat and the Dex module.
        // Find out how to add the DEX adapter dependency for this module.
        StaticDependency::new(CRONCAT_ID, &[CRONCAT_MODULE_VERSION]),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(DCA_APP, DCAApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(DCA_APP, DCAApp, DCA);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> DependencyCreation for crate::DCA<Chain> {
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, AbstractInterfaceError> {
        let croncat_dependency_install_configs: Vec<ModuleInstallConfig> =
            <Croncat<Chain> as DependencyCreation>::dependency_install_configs(
                cosmwasm_std::Empty {},
            )?;
        let adapter_install_config = ModuleInstallConfig::new(
            ModuleInfo::from_id(
                abstract_dex_adapter::DEX_ADAPTER_ID,
                abstract_dex_adapter::contract::CONTRACT_VERSION.into(),
            )?,
            None,
        );
        let croncat_install_config = <Croncat<Chain> as InstallConfig>::install_config(
            &croncat_app::msg::AppInstantiateMsg {},
        )?;

        Ok([
            croncat_dependency_install_configs,
            vec![croncat_install_config, adapter_install_config],
        ]
        .concat())
    }
}
