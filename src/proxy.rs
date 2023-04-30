use super::*;

pub async fn route(req: HttpRequest, body: web::Bytes) -> impl Responder {
    // We place health checks before ssl force to allow http health checks
    if let Some(res) = health_check(&req) {
        return res;
    }

    if let Some(res) = force_ssl(&req) {
        return res;
    }

    proxy_request(req, body).await
}

// Reverse Proxy Health Check
fn health_check(req: &HttpRequest) -> Option<HttpResponse> {
    // If the path is matches, respond with the relavent reply
    match req.path() {
        "/health/reverse-proxy" => Some(HttpResponse::Ok().finish()),
        _ => None,
    }
}

fn force_ssl(req: &HttpRequest) -> Option<HttpResponse> {
    let app_config = req.app_config();

    // If the path is matches, respond with the relavent reply
    match app_config.secure() {
        true => None,
        false => {
            let con_info = req.connection_info();
            let redirect = format!("https://{}{}", con_info.host(), req.path());

            Some(
                HttpResponse::PermanentRedirect()
                    .append_header(("Location", redirect))
                    .finish(),
            )
        }
    }
}

async fn proxy_request(req: HttpRequest, body: web::Bytes) -> HttpResponse {
    let client = awc::Client::default();

    let url = {
        let path = req.path();
        let target_ip = match get_host(&req).as_str() {
            "localhost" => "host.docker.internal",
            _ => {
                todo!("Add support for non localhost domains via environment variables");
                
                #[allow(unreachable_code)]
                {
                    // This should be what is returned when
                    // an unknown domain is requested after
                    // the todo is completed.
                    return HttpResponse::NotFound().finish();
                }
            }
        };

        format!("http://{}:80{}", target_ip, path)
    };

    let mut proxy_res = client
        .request_from(url, req.head())
        .send_body(body)
        .await
        .unwrap();

    let mut res = HttpResponse::build(proxy_res.status());

    for (name, value) in proxy_res.headers() {
        res.append_header((name, value));
    }

    match proxy_res.body().await {
        Ok(body) => res.body(body),
        Err(_) => res.finish(),
    }
}

fn get_host(req: &HttpRequest) -> String {
    let host = req.connection_info();
    let host = host.host().split(":").collect::<Vec<&str>>();
    let host = host.get(0).expect("Error finding host");

    host.to_string()
}
