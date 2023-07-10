use actix_web::*;
use settings::*;

mod cert;
mod proxy;
mod settings;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let certificate: cert::Certificate;

    let use_self_signed_cert = setting("USE_SELF_SIGNED_CERT");

    if use_self_signed_cert.is_none() {
        panic!("USE_SELF_SIGNED_CERT is not set");
    }

    let use_self_signed_cert = use_self_signed_cert.unwrap();

    if "true".to_owned() == use_self_signed_cert {
        // We should use a self-signed certificate in here instead of above
        println!("Using self-signed certificate");
        certificate = cert::self_signed::generate();
    } else if "false".to_owned() == use_self_signed_cert {
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
    } else {
        panic!("USE_SELF_SIGNED_CERT is not a valid lowercase boolean");
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