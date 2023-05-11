use rcgen::generate_simple_self_signed;
use super::settings::*;

#[derive(Debug)]
struct DomainMap {
    domain: String,
    target_host: String, // Local IP or Domain
}

pub struct Certificate {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub fn generate_self_signed() -> Certificate {

    let domains = {
        let domain_names_setting = setting("DOMAIN_MAPS").expect("No domain maps found");
        
        let raw = domain_names_setting.split(" ").collect::<Vec<&str>>();

        let mut domains: Vec<DomainMap> = Vec::new();

        for map in raw.into_iter() {
            let map = map.split("|").collect::<Vec<&str>>();

            let domain = map.get(0).expect("Error finding domain");
            let target_host = map.get(1).expect("Error finding target ip and port");

            domains.push(DomainMap {
                domain: domain.to_string(),
                target_host: target_host.to_string(),
            });
        }

        domains
    };

    let mut subject_alt_names = vec![];

    for domain in domains.iter() {
        subject_alt_names.push(domain.domain.clone());
    }

    

    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    Certificate {
        private_key: cert.serialize_private_key_der(),
        public_key: cert.serialize_der().unwrap(),
    }
}

pub fn make_rustls_config(cert: Certificate) -> rustls::server::ServerConfig {
    rustls::server::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![rustls::Certificate(cert.public_key)],
            rustls::PrivateKey(cert.private_key),
        )
        .expect("bad certificate/key")
}
