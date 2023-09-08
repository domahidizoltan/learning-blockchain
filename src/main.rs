use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use actix_files as fs;
use tera::{Context, Tera};

struct AppState {
    tmpl: Tera,
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        let logger = Logger::default();

        let tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };

        let state = AppState { tmpl: tera };
        App::new()
            .wrap(logger)
            .app_data(web::Data::new(state))
            .service(fs::Files::new("static", "templates/static"))
            .service(index)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

//todo: auto menu