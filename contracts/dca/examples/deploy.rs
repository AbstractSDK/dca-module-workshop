use abstract_app::abstract_interface::{AppDeployer, DeployStrategy};
use abstract_client::{AbstractClient, Namespace, Publisher};
use cw_orch::{
    anyhow, daemon::{ChainInfo, ChainKind, Daemon, NetworkInfo}, prelude::{DaemonBuilder, TxHandler}, tokio::runtime::Runtime
};
use dca_app::{contract::DCA_APP_ID, DCA};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ROLLKIT_NETWORK: NetworkInfo = NetworkInfo {
    id: "rollkit",
    pub_address_prefix: "wasm",
    coin_type: 118u32,
};

pub const LOCAL_ROLLKIT: ChainInfo = ChainInfo {
    kind: ChainKind::Local,
    chain_id: "celeswasm",
    gas_denom: "uwasm",
    gas_price: 0.025,
    grpc_urls: &["http://localhost:9290"],
    network_info: ROLLKIT_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Setup
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(LOCAL_ROLLKIT)
        .build()?;

    let app_namespace = Namespace::from_id(DCA_APP_ID)?;

    // Create an [`AbstractClient`]
    let abstract_client: AbstractClient<Daemon> = AbstractClient::new(chain.clone())?;

    // Get the [`Publisher`] that owns the namespace, otherwise create a new one and claim the namespace
    let publisher: Publisher<_> = abstract_client.publisher_builder(app_namespace).build()?;

    if publisher.account().owner()? != chain.sender() {
        panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
    }

    // QUEST #6
    // Publish the App to the Abstract Platform
    publisher.publish_app::<DCA<Daemon>>()?;
    Ok(())
}
