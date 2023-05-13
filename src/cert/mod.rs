use super::*;

pub mod lets_encrypt;
pub mod self_signed;

pub struct Certificate {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
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

pub fn save_cert(cert: &Certificate) {
    std::fs::write("/certs/cert.txt", &cert.public_key).expect("Couldn't write cert.txt");
    std::fs::write("/certs/key.txt", &cert.private_key).expect("Couldn't write key.txt");
}

pub fn load_cert() -> Option<Certificate> {
    let public_key = std::fs::read("/certs/cert.txt").ok()?;
    let private_key = std::fs::read("/certs/key.txt").ok()?;

    Some(Certificate {
        public_key,
        private_key,
    })
}