use actix_web::{get, HttpResponse, Responder};

#[get("/health")]
pub async fn route() -> impl Responder {
    HttpResponse::Ok()
}
