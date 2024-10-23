use actix_macro::handle;
use actix_web::get;

#[handle]
#[get("/")]
pub async fn test() -> String {
    "hello".to_string()
}

#[handle]
#[get("/test2")]
pub async fn test2() -> String {
    "hello world".to_string()
}

