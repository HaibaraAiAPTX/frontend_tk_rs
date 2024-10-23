use actix_extension::AppContainer;
use actix_web::{App, HttpServer};
use services::project;

mod doamins;
mod dtos;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let app = App::new();
        AppContainer::new().set_app(&app);
        app.service(project::test)
    })
    .bind(("0.0.0.0", 3090))?
    .run()
    .await
}