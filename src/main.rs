mod helper;
mod handlers;
mod lab;
mod app;
mod client;

use client::EthereumClient;

use std::{
    sync::RwLock,
    collections::HashMap,
};

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_files as fs;
use tera::Tera;
use app::{
    model::State as AppState,
    debugservice::DebugService,
};

fn create_tera() -> Tera {
    let mut tera = match Tera::new("templates/*.html") {
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
    tera.autoescape_on(vec![]);

    tera
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(|| {
        let logger = Logger::default();

        let eth_client = match EthereumClient::new() {
            Ok(eth_client) => eth_client,
            Err(e) => {
                println!("Error: {}", e);
                ::std::process::exit(1);
            }
        };

        let state = AppState { 
            tmpl: create_tera(),
            eth_client,
            contracts: RwLock::new(HashMap::new()),
            debug_service: DebugService::new(),
        };
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(state))
            .service(fs::Files::new("static", "templates/static"))
            .configure(handlers::setup_handlers)
            .configure(lab::the_blockchain_messenger_handlers)
    })
    .workers(1) //TODO multiple workers
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
