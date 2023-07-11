#![allow(unused)]
use cw_orch::{
    anyhow,
    prelude::{networks::parse_network, DaemonBuilder},
    tokio::runtime::Runtime,
};

use abstract_dca_app::{contract::DCA_APP_ID, DCAApp};
use abstract_interface::AppDeployer;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();
    let chain = parse_network("juno-1");
    use dotenv::dotenv;
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let rt = Runtime::new()?;
    let chain = DaemonBuilder::default()
        .chain(chain)
        .handle(rt.handle())
        .build()?;
    let app = DCAApp::new(DCA_APP_ID, chain);
    // QUEST #6
    // Now that we have our app we want to deploy it. Use the abstract AppDeployer trait to deploy the app.
    // Hint: You already used this trait in the integration tests!

    // After doing this, notice how you didn't have to provide any Abstract-related information to deploy your app! 
    // This is because cw-orch allows you to ship contract addresses with the interfaces. So an `Abstract::load_from(juno)`
    // will automatically load the correct contract address for you! 
    Ok(())
}
