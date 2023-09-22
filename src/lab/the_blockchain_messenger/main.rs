use actix_web::{post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_files as fs;
use tera::{Context, Tera};
use serde::Deserialize;
use crate::app::model::State as AppState;
use crate::client::EthereumClient;
use crate::client::ethereumclient::ContractInstanceType;
use ethers::contract::{abigen, ContractFactory};
use crate::SignerMiddleware;
use ethers_providers::Http;
use ethers::prelude::Wallet;
use ecdsa::SigningKey;
use k256::Secp256k1;

#[derive(Deserialize)]
struct FormData {
    message: String
}

type TheBlockchainMessengerType = TheBlockchainMessenger<SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>>;

abigen!(
    TheBlockchainMessenger,
    r#"[
        function updateTheMessage(string)
        function changeCounter()(uint)
        function theMessage()(string)
    ]"#
);


//TODO: create smaller debug pane, add input to debug and output as transaction details

#[post("/lab/the-blockchain-messenger")]
pub async fn process(form: web::Form<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    //submit path from var
    // println!("Message: {}", form.message);
    // let context = Context::new();
    // let file_name = format!("form.html");
    // let rendered = app_state.tmpl.render(&file_name, &context).unwrap();

    // HttpResponse::Ok().body(rendered)

    // let client = app_state.eth_client.get_client().clone();
    // let contract = match app_state.contracts.get("TheBlockchainMessenger") {
    //     Some(contract) => contract,
    //     None => return HttpResponse::NotFound().finish(),
    // };
    
    let contract = app_state.eth_client.deploy_contract("TheBlockchainMessenger").await.unwrap();
            
    println!("the_blockchain_messenger_contract: {:?}", contract);


    let messenger: TheBlockchainMessengerType = TheBlockchainMessenger::new(contract.address(), contract.client());
    let receipt = messenger.update_the_message("hello world".to_owned()).send().await.unwrap().await.unwrap();
    println!("receipt: {:?}", receipt);
    //print gas used, block number, tx hash, from addr
    let counter = messenger.change_counter().call().await.unwrap();
    println!("counter: {}", counter);
    let msg = messenger.the_message().call().await.unwrap();
    println!("msg: {}", msg);

    HttpResponse::Ok().body(format!("Message: {}", form.message))
}
