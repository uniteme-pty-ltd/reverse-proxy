use super::*;

#[derive(Debug)]
struct DomainMap {
    domain: String,
    target_host: String, // Local IP or Domain
}

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

        let domains = {
            let domain_names_setting = setting("DOMAIN_MAPS").expect("No domain maps found");
            
            let raw = domain_names_setting.split(" ").collect::<Vec<&str>>();

            let mut domains: Vec<DomainMap> = Vec::new();

            for map in raw.into_iter() {
                let map = map.split(":").collect::<Vec<&str>>();

                let domain = map.get(0).expect("Error finding domain");
                let target_host = map.get(1).expect("Error finding target ip");

                domains.push(DomainMap {
                    domain: domain.to_string(),
                    target_host: target_host.to_string(),
                });
            }

            domains
        };

        let target_host = {
            let host = get_host(&req);

            let mut target_host = None;

            for domain in domains.into_iter() {
                if domain.domain == host {
                    target_host = Some(domain.target_host);
                }
            }

            if target_host.is_none() {
                return HttpResponse::NotFound().finish();
            } else {
                target_host.unwrap()
            }
        };

        format!("http://{}:80{}", target_host, path)
    };

    let mut proxy_res = client
        .request_from(url, req.head())
        .send_body(body)
        .await
        .unwrap();

    let mut res = HttpResponse::build(proxy_res.status());

    for (name, value) in proxy_res.headers() {
        if format!("{}", name).to_lowercase() == "content-encoding" {
            continue;
        }
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
