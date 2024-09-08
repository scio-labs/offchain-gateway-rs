use subxt::config::Hasher;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt::ext::codec::{Decode, Encode};
use std::collections::HashMap;

#[subxt::subxt(runtime_metadata_path = "azero_metadata.scale")]
pub mod azero {}

const AZERO_COIN_TYPE: u64 = 643;
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub struct Database {
    api: OnlineClient<PolkadotConfig>,
    tld_to_contract: HashMap<String, AccountId32>,
}

/// Connects to database
pub async fn bootstrap(url: String, tld_to_contract: Vec<(String, AccountId32)>) -> Database {
    let api = OnlineClient::<PolkadotConfig>::from_url(url)
        .await
        .expect("Failed to connect");
    
    Database { 
        api,
        tld_to_contract: tld_to_contract.into_iter().collect(), 
    }
}

impl Database {
    pub async fn text(&self, domain: &str, key: &str) -> String {
        let (name, tld) = self.process_domain(domain);
        let contract = self.get_contract(&tld).expect("TLD not supported");

        let msg = (get_selector("get_record"), name, key).encode();
        let encoded_resp = self.call_contract(contract, msg).await;
        
        let value: Result<String, u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");        
        value.unwrap_or_default()
    }

    pub async fn addr(&self, domain: &str, coin_type: u64) -> String {
        let mut address = String::new();
        if coin_type == AZERO_COIN_TYPE {
            address = self.get_resolver_address(domain).await;
        } else {
            if let Some(alias) = get_alias(coin_type) {
                let service_key = format!("address.{alias}");
                address = self.text(domain, &service_key).await;
            }

            if address.is_empty() {
                let service_key = format!("address.{coin_type}");
                address = self.text(domain, &service_key).await;
            }
        }

        if address.is_empty() {
            address = match coin_type == 60 {
                true => ZERO_ADDRESS,
                false => "0x",
            }.to_string();
        }
        
        address
    }

    async fn call_contract(&self, contract: AccountId32, msg: Vec<u8>) -> Vec<u8> {
        let payload = azero::apis().contracts_api().call(
            AccountId32([1u8;32]),
            contract,
            0,
            None,
            None,
            msg,
        );
    
        let rs = self.api
            .runtime_api()
            .at_latest()
            .await
            .expect("connection failure")
            .call(payload)
            .await
            .expect("call failed");
    
        let mut data = rs
            .result
            .expect("execution without result")
            .data;
    
        // InkLang error check
        assert_eq!(data.remove(0), 0);
    
        data
    }

    pub fn get_contract(&self, tld: &str) -> Option<AccountId32> {
        self.tld_to_contract.get(tld).cloned()
    }

    async fn get_resolver_address(&self, domain: &str) -> String {
        let (name, tld) = self.process_domain(domain);
        let contract = self.get_contract(&tld).expect("TLD not supported");

        let msg = (get_selector("get_address"), name).encode();
        let encoded_resp = self.call_contract(contract, msg).await;
        
        let value: Result<AccountId32, u8> = Decode::decode(&mut &encoded_resp[..]).expect("failed to decode");        
        
        match value {
            Ok(v) => v.to_string(),
            Err(_) => String::new(),
        }
    }
    
    pub fn update_tld(&mut self, tld: String, contract: Option<AccountId32>) {
        match contract {
            None => self.tld_to_contract.remove(&tld),
            Some(contract) => self.tld_to_contract.insert(tld, contract)
        };
    }

    fn process_domain(&self, domain: &str) -> (String, String) {
        let mut labels = domain.split('.');
        let name = labels.next().expect("infallible");
        let tld = labels.collect::<Vec<_>>().join(".");

        (name.into(), tld)
    }
}

fn get_selector(name: &str) -> [u8; 4] {
    let bytes = subxt::config::substrate::BlakeTwo256::hash(name.as_bytes());
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

fn get_alias(coin_type: u64) -> Option<String> {
    let alias = HashMap::from([
        (0, "btc"),
        (60, "eth"),
        (354, "dot"),
        (434, "ksm"),
        (501, "sol"),
    ]);

    alias.get(&coin_type).map(|&i| i.to_string())
}
