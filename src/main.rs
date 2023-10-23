mod app;
mod client;
mod handlers;
mod helper;
mod lab;

use client::EthereumClient;

use std::{collections::HashMap, sync::RwLock};

use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpServer};
pub use app::{
    debugservice::DebugService as AppDebug, model::Error as AppError, model::State as AppState,
};
use tera::Tera;

fn create_tera() -> Result<Tera, tera::Error> {
    let mut tera = Tera::new("templates/*.html")?;
    let labs_tera = Tera::new("src/**/*.html")?;

    tera.extend(&labs_tera)?;
    tera.autoescape_on(vec![]);

    Ok(tera)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(|| {
        let logger = Logger::default();

        let eth_client = EthereumClient::new().unwrap();
        let tera = create_tera().unwrap();

        let addresses = helper::get_all_account_addresses().unwrap();

        let state = AppState {
            tmpl: tera,
            eth_client,
            contracts: RwLock::new(HashMap::new()),
            debug_service: AppDebug::new(),
            accounts: addresses,
        };
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(state))
            .service(fs::Files::new("static", "templates/static"))
            .configure(handlers::setup_handlers)
            .configure(lab::the_blockchain_messenger_handlers)
            .configure(lab::smart_money_handlers)
    })
    .workers(1) //TODO multiple workers
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
