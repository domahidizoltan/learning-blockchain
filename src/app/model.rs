use async_rwlock::RwLock;
use std::{collections::HashMap, env};

use crate::client::{ContractInstanceType, EthereumClient};
use ethers::types::Address;
use tera::Tera;

use crate::app::debugservice::DebugService;

pub struct State {
    pub tmpl: Tera,
    pub eth_client: EthereumClient,
    pub contracts: RwLock<HashMap<String, ContractInstanceType>>,
    pub debug_service: DebugService,
    pub accounts: Vec<Address>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("env var {} must be set", .0)]
    KeyNotSetError(String, #[source] env::VarError),

    #[error("could not parse address")]
    AddressParseError(#[source] Box<dyn std::error::Error>),
}
