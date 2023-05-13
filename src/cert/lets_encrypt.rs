use super::*;

use acme_lib::create_p256_key;
use acme_lib::persist::FilePersist;
use acme_lib::{Directory, DirectoryUrl};

struct OrderDomains {
    primary: String,
    alternates: Vec<String>
}

impl OrderDomains {
    fn new() -> Self {
        let domain_names_setting = setting("DOMAIN_MAPS").expect("No domain maps found");

        let raw = domain_names_setting.split(" ").collect::<Vec<&str>>();

        let mut domains: Vec<String> = Vec::new();

        for map in raw.into_iter() {
            let map = map.split("|").collect::<Vec<&str>>();

            let domain = map.get(0).expect("Error finding domain");

            domains.push(domain.to_string());
        }

        let primary = {
            // Loop through domains and find the one with the least subdomains
            let mut current_primary = domains[0].clone();

            for domain in domains.iter() {
                if domain.split(".").count() < current_primary.split(".").count() {
                    current_primary = domain.clone();
                }
            }

            // Remove the primary domain from the alternate domains list
            let primary_index = domains.iter().position(|x| *x == current_primary).unwrap();
            domains.remove(primary_index);

            current_primary
        };

        OrderDomains {
            primary: primary,
            alternates: domains
        }
    }
}

#[derive(Clone)]
struct ChallengeDetails {
    token: String,
    proof: String,
}

struct ServerChallenges {
    challenges: Vec<ChallengeDetails>,
}

pub async fn request() -> Certificate {
    let url = DirectoryUrl::LetsEncryptStaging;
    let persist = FilePersist::new(".");
    let dir = Directory::from_url(persist, url).expect("Couldn't create directory");
    let acc = dir
        .account("admin@uniteme.app")
        .expect("Coudn't create account");

    let ord_domains = OrderDomains::new();

    println!("PRIMARY: {:#?}", &ord_domains.primary);
    println!("ALTERNATES: {:#?}", ord_domains.alternates.iter().map(|x| x.as_str()).collect::<Vec<&str>>().as_slice());

    let mut ord_new = acc
        .new_order(
            &ord_domains.primary,
            // "prerelease.studyscore.app",
            ord_domains.alternates.iter().map(|x| x.as_str()).collect::<Vec<&str>>().as_slice(),
            // &["api.prerelease.studyscore.app"]
        )
        .expect("Couldn't create new order");

    let ord_csr = loop {
        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        let auths = ord_new
            .authorizations()
            .expect("Couldn't get authorizations");

        let mut challenges = Vec::new();

        for auth in auths.into_iter() {
            challenges.push(auth.http_challenge());
        }

        let mut challenges_details = Vec::new();

        for challenge in challenges.iter() {
            challenges_details.push(ChallengeDetails {
                token: challenge.http_token().to_string(),
                proof: challenge.http_proof(),
            });
        }

        // SERVER //

        let srv = HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(ServerChallenges {
                    challenges: challenges_details.clone(),
                }))
                .service(web::resource("/.well-known/acme-challenge/{token}").route(web::to(handler)))
        })
        .bind(("0.0.0.0", 80))
        .expect("Failed to bind to port 80")
        .run();
        let srv_handle = srv.handle();
        println!("Starting server");
        // actix_web::rt::spawn(srv);
        let _server_task = std::thread::spawn(move || actix_web::rt::System::new().block_on(srv));
        println!("Server Started");

        // Allow the server to start before validating
        std::thread::sleep(std::time::Duration::from_secs(2));

        for challenge in challenges.into_iter() {
            challenge.validate(0).expect("Couldn't validate challenge");
        }

        ord_new.refresh().expect("Couldn't refresh order");

        srv_handle.stop(false).await;
    };

    let pkey_pri = create_p256_key();
    let ord_cert = ord_csr
        .finalize_pkey(pkey_pri, 5000)
        .expect("Couldn't finalize CSR");
    let cert = ord_cert
        .download_and_save_cert()
        .expect("Couldn't download and save certificate");

    println!("It worked!");
    Certificate {
        private_key: cert.private_key_der().into(),
        public_key: cert.certificate_der().into(),
    }
}

async fn handler(data: web::Data<ServerChallenges>, path: web::Path<(String,)>) -> HttpResponse {

    let provided_token = path.into_inner().0;

    for challenge in data.challenges.iter() {
        if challenge.token == provided_token {
            return HttpResponse::Ok().body(challenge.proof.clone());
        }
    }

    HttpResponse::NotFound().finish()
}