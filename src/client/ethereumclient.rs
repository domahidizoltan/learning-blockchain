use crate::helper::get_env_var;

use std::{
    path::Path,
    sync::Arc,
};
use ethers::{
    prelude::Wallet,
    providers::{Provider, Http},
    contract::ContractFactory,
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
    contract::ContractInstance,
};
use ethers_solc::{Solc, CompilerOutput};
use k256::Secp256k1;

const PRIVATE_KEY: &str = "PRIVATE_KEY";
const ENDPOINT: &str = "ENDPOINT";
const CHAIN_ID: &str = "CHAIN_ID";
const CONTRACTS_PATH: &str = "CONTRACTS_PATH";

pub type ContractInstanceType = ContractInstance<Arc<SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>>, SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>>;

pub struct EthereumClient {
    client: std::sync::Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    contracts: CompilerOutput,
}

impl EthereumClient {
    pub fn new() -> Result<Self, String> {

        let private_key = get_env_var(PRIVATE_KEY)?;
        let endpoint = get_env_var(ENDPOINT)?;
        let chain_id = get_env_var(CHAIN_ID)?;

        let chain_id = match chain_id.parse::<u64>() {
            Ok(id) => Some(id),
            Err(_) => return Err("chain id could not be parsed".to_owned()),
        };

        let provider = match Provider::<Http>::try_from(&endpoint) {
            Ok(provider) => Some(provider),
            Err(_) => return Err(format!("could not connect to endpoint {}", &endpoint)),
        };

        let wallet = match private_key[2..].parse::<LocalWallet>() {
            Ok(wallet) => Some(wallet),
            Err(_) => return Err("private key could not be parsed".to_owned()),
        };

        let wallet_with_chain_id = wallet.unwrap().with_chain_id(chain_id.unwrap());
        let client = SignerMiddleware::new(provider.unwrap(), wallet_with_chain_id);

        match EthereumClient::compile_contracts() {
            Ok(contracts) => Ok(EthereumClient{
                client: std::sync::Arc::new(client),
                contracts: contracts,
            }),
            Err(e) => return Err(e),
        }
    }

    fn compile_contracts() -> Result<CompilerOutput, String> {
        let path = get_env_var(CONTRACTS_PATH)?;
        let source = match Path::new(&path).canonicalize() {
            Ok(path) => Some(path),
            Err(_) => return Err(format!("could not find contract source on path {}", &path)),
        };

        match Solc::default().compile_source(source.unwrap()) {
            Ok(compiled) => Ok(compiled),
            Err(e) => return Err(format!("could not compile contracts {}", e)),
        }
    }

    pub fn get_client(&self) -> std::sync::Arc<SignerMiddleware<Provider<Http>, LocalWallet>> {
        self.client.clone()
    }

    pub async fn deploy_contract(&self, contract_name: &str) -> Result<ContractInstanceType, String> {
        let compiled = self.contracts.find(contract_name);

        let (abi, bytecode, _runtime_bytecode) = match compiled {
            Some(compiled) => compiled.into_parts_or_default(),
            None => return Err(format!("could not find contract {}", contract_name)),
        };
        
        let factory = ContractFactory::new(abi, bytecode, self.client.clone());

        let deployer = match factory.deploy(()) {
            Ok(deployer) => Some(deployer),
            Err(_) => return Err("could not create deployer".to_owned()),
        };

         let contract = deployer.unwrap()
            .confirmations(0usize)
            .send().await;

        match contract {
            Ok(contract) => Ok(contract),
            Err(_) => return Err("could not deploy contract".to_owned()),
        }

    }


}
