// Import reexports from the Demia SDK
use demia_sdk::{
    identity::iota::iota::{IotaDID, NetworkName},
    iota_sdk::client::Client as IotaClient,
    iota_stronghold::Stronghold,
    streams::{transport::utangle::Client as StreamsClient, TransportMessage},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = dotenv::var("URL")?;

    // Initialize IOTA client
    let _iota_client = IotaClient::builder().with_node(&url)?.finish()?;

    // Example: Interact with the IOTA module
    // ...

    // Initialize Stronghold for secure storage
    let _stronghold = Stronghold::default();

    // Example: Use Stronghold for secure storage
    // ...

    // Example: Interact with the Identity module
    let iota_identity = IotaDID::from_alias_id("seed_for_identity", &NetworkName::try_from("smr").unwrap());

    // Print the created DID
    println!("Created DID: {}", iota_identity);

    // Example: Interact with Streams module
    let _client = StreamsClient::<TransportMessage>::new("mysql://user:password@localhost/db");

    Ok(())
}
