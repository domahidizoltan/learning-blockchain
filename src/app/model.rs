use std::{collections::HashMap, env, sync::RwLock};

use crate::client::{ContractInstanceType, EthereumClient};
use tera::Tera;

use crate::app::debugservice::DebugService;

pub struct State {
    pub tmpl: Tera,
    pub eth_client: EthereumClient,
    pub contracts: RwLock<HashMap<String, ContractInstanceType>>,
    pub debug_service: DebugService,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("env var {} must be set", .0)]
    KeyNotSetError(String, #[source] env::VarError),
}
