use rcgen::generate_simple_self_signed;

pub struct Cert {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>
}

pub fn generate() -> Cert {
    // Generate a certificate that's valid for "localhost" and "hello.world.example"
    let subject_alt_names = vec!["localhost".to_string()];

    let cert = generate_simple_self_signed(subject_alt_names).unwrap();
    // println!("{}", cert.serialize_pem().unwrap());
    // println!("{}", cert.serialize_private_key_pem());

    Cert {
        private_key: cert.serialize_private_key_der(),
        public_key: cert.serialize_der().unwrap()
    }
}