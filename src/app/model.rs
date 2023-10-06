use std::{collections::HashMap, sync::RwLock};

use crate::client::{ContractInstanceType, EthereumClient};
use tera::Tera;

use crate::app::debugservice::DebugService;

pub struct State {
    pub tmpl: Tera,
    pub eth_client: EthereumClient,
    pub contracts: RwLock<HashMap<String, ContractInstanceType>>,
    pub debug_service: DebugService,
}
