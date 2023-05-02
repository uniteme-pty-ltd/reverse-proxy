use actix_web::*;
use settings::*;

mod cert;
mod proxy;
mod settings;
mod health;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cert = cert::generate_self_signed();

    if Some("true".to_owned()) == setting("USE_SELF_SIGNED_CERT") {
        // We should use a self-signed certificate in here instead of above
    }

    HttpServer::new(|| App::new().wrap(middleware::Compress::default()).route("/{tail:.*}", web::to(proxy::route)).service(health::route))
        .bind(("0.0.0.0", 80))
        .expect("Couldn't bind to port 80")
        .bind_rustls(("0.0.0.0", 443), cert::make_rustls_config(cert))
        .expect("Couldn't bind to port 443")
        .run()
        .await
}
