use actix_web::*;
use settings::*;

mod cert;
mod proxy;
mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let certificate: cert::Certificate;

    if Some("true".to_owned()) == setting("USE_SELF_SIGNED_CERT") {
        // We should use a self-signed certificate in here instead of above
        println!("Using self-signed certificate");
        certificate = cert::self_signed::generate();
    } else {
        println!("Using let's encrypt certificate");
        if let Some(loaded_cert) = cert::load_cert() {
            println!("Existing certificate founded and loaded");
            certificate = loaded_cert;
        } else {
            println!("Existing certificate not found, generating");
            certificate = cert::lets_encrypt::request().await;
            cert::save_cert(&certificate);
            println!("Certificate generated and validated");
        }
    }

    println!("Starting proxy");
    run_proxy(certificate).await
}

async fn run_proxy(certificate: cert::Certificate) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .route("/{tail:.*}", web::to(proxy::route))
    })
    .bind(("0.0.0.0", 80))
    .expect("Couldn't bind to port 80")
    .bind_rustls(("0.0.0.0", 443), cert::make_rustls_config(certificate))
    .expect("Couldn't bind to port 443")
    .run()
    .await
}