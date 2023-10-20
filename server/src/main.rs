use std::cell::RefCell;

use actix_files as fs;
use actix_web::http::header::ContentType;
use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse };
use minijinja::value::Value;
use minijinja::{path_loader, Environment};
use r2d2_sqlite::SqliteConnectionManager;

mod models;

thread_local! {
    static CURRENT_REQUEST: RefCell<Option<HttpRequest>> = RefCell::default()
}

fn with_bound_req<F, R>(req: &HttpRequest, f: F) -> R
where 
    F: FnOnce() -> R, 
{
    CURRENT_REQUEST.with(|current_req| *current_req.borrow_mut() = Some(req.clone()));
    let rv = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    CURRENT_REQUEST.with(|current_req| current_req.borrow_mut().take());
    match rv {
        Ok(rv) => rv,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

pub struct AppState {
    env: minijinja::Environment<'static>,
    pub pool: r2d2::Pool<SqliteConnectionManager>
}

impl AppState {
    pub fn render_template(&self, name: &str, req: &HttpRequest, ctx: Value) -> HttpResponse {
        with_bound_req(req, || {
            let tmpl = self.env.get_template(name).unwrap();
            let rv = tmpl.render(ctx).unwrap();
            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(rv)
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let mut env = Environment::new();
    env.set_loader(path_loader("pages"));

    let manager = SqliteConnectionManager::file("data.sqlite");
    let pool = r2d2::Pool::new(manager).unwrap();

    let state = web::Data::new(AppState { 
        env,
        pool 
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(fs::Files::new("/static", "./static").show_files_listing())
   })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
