use abstract_client::{AbstractClient, Namespace, Publisher};
use cw_orch::{
    anyhow,
    daemon::{networks::CONSTANTINE_3, Daemon},
    prelude::{DaemonBuilder, TxHandler},
    tokio::runtime::Runtime,
};
use dca_app::{contract::DCA_APP_ID, DCA};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    // Setup
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(CONSTANTINE_3)
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
