use std::path::Path;

use actix_web::{get, post, web::{self, Redirect}, App, HttpResponse, HttpServer, Responder, middleware::{Logger, self}, http::header::ContentRange};
use actix_files as fs;
use tera::{Context, Tera};
use serde::Deserialize;
use crate::app::{model::State as AppState, self};
use crate::client::EthereumClient;
use crate::client::ethereumclient::ContractInstanceType;
use ethers::{contract::{abigen, ContractFactory}, etherscan::contract};
use ethers::middleware::SignerMiddleware;
use ethers_providers::Http;
use ethers::prelude::Wallet;
use ecdsa::SigningKey;
use k256::Secp256k1;

#[derive(Deserialize)]
struct FormData {
    message: String
}

// type TheBlockchainMessengerType = TheBlockchainMessenger<SignerMiddleware<ethers_providers::Provider<Http>, Wallet<ecdsa::SigningKey<Secp256k1>>>>;

abigen!(
    TheBlockchainMessenger,
    r#"[
        function updateTheMessage(string)
        function changeCounter()(uint)
        function theMessage()(string)
    ]"#
);

const CONTRACT_NAME: &str = "TheBlockchainMessenger";
const LAB_PATH: &str = "lab/the_blockchain_messenger";
const LAB_BASEURL: &str = "/lab/the-blockchain-messenger";

pub fn setup_handlers(cfg: &mut web::ServiceConfig) {
    cfg
        .service(web::resource(LAB_BASEURL).route(web::get().to(load_template_handler)))
        .service(web::resource(format!("{}/result", LAB_BASEURL)).route(web::get().to(tx_result_handler)))
        .service(web::resource(format!("{}/deploy", LAB_BASEURL)).route(web::post().to(deploy_handler)))
        .service(web::resource(format!("{}/form", LAB_BASEURL)).route(web::post().to(submit_handler)));
}

//TODO: create smaller debug pane, add input to debug and output as transaction details

async fn load_template_handler(app_state: web::Data<AppState>) -> impl Responder {
    let readme_path = format!("src/{}/README.md", LAB_PATH);
    let template_path = format!("{}/template.html", LAB_PATH);

    let html = match markdown::file_to_html(Path::new(&readme_path)) {
        Ok(html) => html,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let mut context = Context::new();
    context.insert("readme", &html);
    let rendered = match app_state.tmpl.render(&template_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(rendered)
}

async fn tx_result_handler(app_state: web::Data<AppState>) -> impl Responder {
    let result_path = format!("{}/result.html", LAB_PATH);

    let mut context = Context::new();
    context.insert("contract_address", "0x123");
    context.insert("message", "hello world");
    context.insert("counter", "42");
    let rendered = match app_state.tmpl.render(&result_path, &context) {
        Ok(rendered) => rendered,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    HttpResponse::Ok().body(rendered)
}

async fn deploy_handler(app_state: web::Data<AppState>) -> impl Responder {
        // let client = app_state.eth_client.get_client().clone();
    // let contract = match app_state.contracts.get("TheBlockchainMessenger") {
    //     Some(contract) => contract,
    //     None => return HttpResponse::NotFound().finish(),
    // };

    app_state.debug_service.send_debug_event(format!("deploying contract")).await;

    if app_state.contracts.read().unwrap().get(CONTRACT_NAME).is_none() {
        let contract = app_state.eth_client.deploy_contract(CONTRACT_NAME).await.unwrap();
            
        println!("the_blockchain_messenger_contract: {:?}", contract);
        // let r = app_state.contracts.write().unwrap()
        let mut c = app_state.contracts.write().unwrap();
        c.insert(CONTRACT_NAME.to_owned(), contract);
    }

    let evt = format!("____contracts: {:?}", app_state.contracts.read().unwrap());
    

    Redirect::to(LAB_BASEURL.to_owned() + "/form").see_other()
}

async fn submit_handler(form: web::Form<FormData>, app_state: web::Data<AppState>) -> impl Responder {
    //submit path from var
    // println!("Message: {}", form.message);
    // let context = Context::new();
    // let file_name = format!("form.html");
    // let rendered = app_state.tmpl.render(&file_name, &context).unwrap();

    // HttpResponse::Ok().body(rendered)

    log::info!("tttttttt: {}", form.message);
    let read_lock = app_state.contracts.read().unwrap();
    println!("read_lock: {:?}", read_lock);
    let contract = read_lock.get(CONTRACT_NAME).unwrap();
    println!("contract: {:?}", contract);
    let messenger = TheBlockchainMessenger::new(contract.address(), contract.client());
    let receipt = messenger.update_the_message("hello world".to_owned()).send().await.unwrap().await.unwrap();
    println!("receipt: {:?}", receipt);
    //print gas used, block number, tx hash, from addr
    let counter = messenger.change_counter().call().await.unwrap();
    println!("counter: {}", counter);
    let msg = messenger.the_message().call().await.unwrap();
    println!("msg: {}", msg);

    HttpResponse::Ok().body(format!("Message: {}", form.message))
}
