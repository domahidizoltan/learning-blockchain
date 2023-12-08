use std::{collections::HashMap, env, sync::Arc};

use crate::app::debugservice::DebugService;
use crate::client::{ContractInstanceType, EthereumClient};
use ethers::types::Address;
use futures::lock::Mutex;
use tera::Tera;

pub struct State {
    pub tmpl: Tera,
    pub eth_client: EthereumClient,
    pub contracts: Arc<Mutex<HashMap<String, ContractInstanceType>>>,
    pub debug_service: DebugService,
    pub accounts: Vec<Address>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("env var {} must be set", .0)]
    KeyNotSetError(String, #[source] env::VarError),

    #[error("could not parse address")]
    AddressParseError(#[source] Box<dyn std::error::Error>),

    #[error("no block found at hash or number{}", .0)]
    NoBlockFoundError(String),
}
