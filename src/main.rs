use actix_web::*;

mod cert;

async fn proxy(
    request: HttpRequest,
    path: web::Path<String>,
    bytes: web::Bytes
) -> impl Responder {

    let client = awc::Client::default();

    let host = request.connection_info();
    let host = host.host().split(":").collect::<Vec<&str>>();
    let host = host.get(0).expect("Error finding host");
    
    let url = format!("http://host.docker.internal:32772{}", request.path());

    let mut new_req = client.request_from(url, request.head());
    let mut new_req = new_req.send_body(bytes).await.unwrap();

    let mut new_res = HttpResponse::build(new_req.status());

    for (name, value) in new_req.headers() {
        new_res.append_header((name, value));
    }

    match new_req.body().await {
        Ok(body) => new_res.body(body),
        Err(_) => new_res.finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let cert = cert::generate();

    let ssl = rustls::server::ServerConfig::builder()
    .with_safe_default_cipher_suites()
    .with_safe_default_kx_groups()
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![rustls::Certificate(cert.public_key)], rustls::PrivateKey(cert.private_key))
    .expect("bad certificate/key");

    HttpServer::new(|| {
        App::new().route("/{tail:.*}", web::to(proxy))
    })
    .bind_rustls(("0.0.0.0", 443), ssl)?
    .run()
    .await
}