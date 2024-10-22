use actix_web::{App, HttpServer};

mod doamins;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
    }).bind(("0.0.0.0", 3090))?
    .run()
    .await
}
