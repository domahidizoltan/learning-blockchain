mod helper;
mod lab;
mod app;
mod client;

use client::EthereumClient;

use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use std::collections::HashMap;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_files as fs;
use tera::{Context, Tera};
use lab::the_blockchain_messenger_processor;
use app::model::State as AppState;
use ethers::{
    providers::{Provider, Http},
    core::types::Bytes,
    contract::{abigen, ContractFactory},
    core::utils::Anvil,
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware,
};
use ethers_solc::{Solc, ProjectPathsConfig, Project, Artifact};

#[get("/")]
async fn index(app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("my_var", &"this is my var");
    let rendered = app_state.tmpl.render("index.html", &context).unwrap();

    HttpResponse::Ok().body(rendered)
}

#[get("/{name}")]
async fn hello(name: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let mut context = Context::new();
    context.insert("name", name.as_str());
    let file_name = format!("{name}.html");
    let rendered = app_state.tmpl.render(&file_name, &context).unwrap();

    HttpResponse::Ok().body(rendered)
}

#[get("/lab/{path}")]
async fn lab_handler(path: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let context = Context::new();
    let p = path.replace("-", "_");
    let file_name = format!("lab/{p}/form.html");
    let rendered = app_state.tmpl.render(&file_name, &context).unwrap();

    HttpResponse::Ok().body(rendered)
}

abigen!(
    TheBlockchainMessenger,
    r#"[
        function updateTheMessage(string)
        function changeCounter()(uint)
        function theMessage()(string)
    ]"#
);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();

        let eth_client = EthereumClient::new().unwrap();
        println!("eth_client: {:?}", eth_client.get_client());
   
        let mut tera = match Tera::new("src/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        let labs_tera = match Tera::new("src/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.extend(&labs_tera).unwrap();

        let state = AppState { 
            tmpl: tera,
            eth_client,
            contracts: HashMap::from([
                // ("TheBlockchainMessenger".to_owned(), the_blockchain_messenger_contract)
                ]),
        };
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(state))
            .service(fs::Files::new("static", "templates/static"))
            .service(index)
            .service(hello)
            .service(lab_handler)
            .service(the_blockchain_messenger_processor)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

//todo: auto menu