use super::*;
use rcgen::generate_simple_self_signed;

#[derive(Debug)]
struct DomainMap {
    domain: String,
    target_host: String, // Local IP or Domain
}

pub fn generate() -> Certificate {
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
