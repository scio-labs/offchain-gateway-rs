use std::{env, path::Path, fs::File, io::BufReader, str::FromStr};
use std::collections::HashMap;
use subxt::utils::AccountId32;
use dotenvy::dotenv;
use ethers::signers::{LocalWallet, Signer};
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub mod ccip;
pub mod database;
pub mod gateway;
mod http;
pub mod multicoin;
pub mod state;
pub mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let filter = EnvFilter::new(format!("offchain_gateway={}", Level::DEBUG));

    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let url = env::var("PROVIDER_URL").unwrap_or("wss://ws.test.azero.dev".into());
    
    let path = env::var("SUPPORTED_TLD_PATH").unwrap_or("supported-tlds.json".into());
    let path = Path::new(&path);
    let tld_to_contract = get_supported_tlds(path);

    let db = database::bootstrap(url, tld_to_contract).await;

    let wallet: LocalWallet = LocalWallet::from_str(
        env::var("PRIVATE_KEY")
            .expect("Could not find PRIVATE_KEY")
            .as_str(),
    )
    .unwrap();

    let address = format!("{:?}", wallet.address());
    info!("Signing with address: {}", address);

    let state = state::GlobalState { db, wallet };

    http::serve(state).await;

    info!("Shutting down");
}

fn get_supported_tlds(path: &std::path::Path) -> Vec<(String, AccountId32)> {
    let file = File::open(path).expect("file couldn't be opened");
    let reader = BufReader::new(file);

    let tld_to_contract: HashMap<String, String> = serde_json::from_reader(reader).expect("failed to parse");
    
    tld_to_contract
        .into_iter()
        .map(|(tld, contract)| (tld, contract.parse().expect("decode failed")))
        .collect()
}

// let mut records = HashMap::new();
// records.insert(
//     "avatar".to_string(),
//     Some(
//         "https://metadata.ens.domains/goerli/avatar/luc.myeth.id?timestamp=1700508402907"
//             .to_string(),
//     ),
// );
// let addresses = HashMap::new();
// // let h = hex::decode("0123456789ABCDEF0123456789ABCDEF").unwrap();
// let h = namehash("luc.myeth.id").to_fixed_bytes().to_vec();
// db.upsert(&h, &records, &addresses).await;
// let r = db
//     .get_records(&h, &vec!["avatar", "display", "header"])
//     .await;
// println!("{:?}", r);
