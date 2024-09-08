use std::sync::Arc;

use ethers::{abi::Token, providers::namehash, utils::keccak256};
use thiserror::Error;
use tracing::{debug, info};

use crate::multicoin::cointype::coins::CoinType;
use crate::multicoin::encoding::MulticoinEncoder;
use crate::{ccip::lookup::ResolverFunctionCall, state::GlobalState};

use super::{payload::ResolveCCIPPostPayload, signing::UnsignedPayload};

pub struct UnresolvedQuery<'a> {
    pub name: String,
    pub data: ResolverFunctionCall,
    pub calldata: &'a ResolveCCIPPostPayload,
}

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("Unknown error")]
    NotFound,
    #[error("Record not found: {0}")]
    NotFoundRecord(String),
    #[error("Unparsable")]
    Unparsable,
    #[error("Sender unparsable")]
    SenderUnparsable,
    #[error("Payload unparsable")]
    PayloadUnparsable,
    #[error("Hash mismatch")]
    HashMismatch,
}

impl UnresolvedQuery<'_> {
    pub async fn resolve(&self, state: Arc<GlobalState>) -> Result<UnsignedPayload, ResolveError> {
        let payload: Vec<Token> = match &self.data {
            ResolverFunctionCall::Text(alt_hash, record) => {
                info!(name = self.name, record = record, "Resolution");

                let hash = namehash(&self.name).to_fixed_bytes().to_vec();

                if alt_hash != &hash {
                    return Err(ResolveError::HashMismatch);
                }

                let value = state.db.text(&self.name, record).await;
                vec![Token::String(value)]
            }
            ResolverFunctionCall::AddrMultichain(_bf, chain) => {
                info!(
                    name = self.name,
                    chain = chain,
                    "Resolution Address Multichain"
                );

                let value: String = state.db.addr(&self.name, *chain).await;

                let bytes = CoinType::from(*chain as u32).encode(&value).map_err(|err| {
                    debug!("error while trying to encode {}: {}", chain, err);
                    ResolveError::Unparsable
                })?;

                vec![Token::Bytes(bytes)]
            }
            ResolverFunctionCall::Addr(_bf) => {
                info!(name = self.name, "Resolution Address");

                let chain = 60;
                let value = state.db.addr(&self.name, chain).await;

                let address = value.parse().map_err(|_| ResolveError::Unparsable)?;

                vec![Token::Address(address)]
            }
            _ => {
                info!("Unimplemented Method");

                Vec::new()
            }
        };

        let ttl = 3600;
        let expires = chrono::Utc::now().timestamp() as u64 + ttl;
        let sender = self
            .calldata
            .sender
            .parse()
            .map_err(|_| ResolveError::SenderUnparsable)?;
        let request_payload = hex::decode(self.calldata.data.trim_start_matches("0x"))
            .map_err(|_| ResolveError::PayloadUnparsable)?;
        let data = ethers::abi::encode(&payload);
        let request_hash = keccak256(request_payload).to_vec();
        let result_hash = keccak256(&data).to_vec();

        Ok(UnsignedPayload {
            data,
            expires,
            request_hash,
            result_hash,
            sender,
        })
    }
}
