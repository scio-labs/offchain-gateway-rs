use subxt::config::Hasher;
use subxt::utils::AccountId32;
use subxt::{OnlineClient, PolkadotConfig};
use subxt::ext::codec::{Decode, Encode};
use std::collections::HashMap;

#[subxt::subxt(runtime_metadata_path = "azero_metadata.scale")]
pub mod azero {}

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

    pub async fn addr(&self, _name: &str, _coin_type: u64) -> String {
        unimplemented!()
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
