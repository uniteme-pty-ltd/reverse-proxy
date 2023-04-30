use rcgen::generate_simple_self_signed;

pub struct Certificate {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub fn generate_self_signed() -> Certificate {
    let subject_alt_names = vec!["localhost".to_string()];

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
