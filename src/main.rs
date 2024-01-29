use std::cell::RefCell;

use actix_files as fs;
use actix_web::http::header::ContentType;
use actix_web::web::ServiceConfig;
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse };
use minijinja::value::Value;
use minijinja::{path_loader, Environment};
use database::LibSqlConnectionManager;
use shuttle_actix_web::ShuttleActixWeb;

mod models;
mod database;

pub struct AppState {
    env: minijinja::Environment<'static>,
    pub pool: bb8::Pool<LibSqlConnectionManager>
}

impl AppState {
    pub fn render_template(&self, name: &str, ctx: Value) -> HttpResponse {
        let tmpl = self.env.get_template(name).unwrap();
        let rv = tmpl.render(ctx).unwrap();
        HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(rv)
    }
}

#[shuttle_runtime::main]
async fn main() -> std::io::Result<()> {
    
    let mut env = Environment::new();
    env.set_loader(path_loader("pages"));

    let manager = database::LibSqlConnectionManager{};
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let state = web::Data::new(AppState { 
        env,
        pool 
    });

    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(state.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing());
    };

    Ok(config.into())
}
