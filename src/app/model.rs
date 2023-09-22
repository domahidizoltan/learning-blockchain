use std::collections::HashMap;

use crate::client::{EthereumClient, ContractInstanceType};
use tera::Tera;

pub struct State {
    pub tmpl: Tera,
    pub eth_client: EthereumClient,
    pub contracts: HashMap<String, ContractInstanceType>
}