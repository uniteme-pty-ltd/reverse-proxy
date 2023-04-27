use actix_web::*;

async fn proxy(
    request: HttpRequest,
    path: web::Path<String>,
    bytes: web::Bytes
) -> impl Responder {

    let client = awc::Client::default();

    let host = request.connection_info();
    let host = host.host().split(":").collect::<Vec<&str>>();
    let host = host.get(0).expect("Error finding host");
    
    let url = format!("http://{}:32768{}", host, request.path());

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
    HttpServer::new(|| {
        App::new().route("/{tail:.*}", web::to(proxy))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}