use std::fs::File;
use std::io::BufReader;

use rustls::{self, ServerConfig};

pub fn read(cert_filename: &str, key_filename: &str) -> ServerConfig {
    let cert = {
        let file = File::open(cert_filename).unwrap_or_else(|e| {
            panic!(
                "Can't open tls certificate file: '{}'. {}.",
                cert_filename, e
            )
        });
        let mut rdr = BufReader::new(file);
        rustls::internal::pemfile::certs(&mut rdr)
            .unwrap_or_else(|()| panic!("Can't parse tls certificate file: {}", cert_filename))
    };

    let key = {
        let mut pkcs8 = {
            let file = File::open(&key_filename)
                .unwrap_or_else(|e| panic!("Can't open tls key file: '{}'. {}.", key_filename, e));
            let mut rdr = BufReader::new(file);
            rustls::internal::pemfile::pkcs8_private_keys(&mut rdr)
                .unwrap_or_else(|()| panic!("Can't parse pkcs8 tls key file: {}", key_filename))
        };

        // No pkcs8 key, try RSA PRIVATE PEM
        if !pkcs8.is_empty() {
            pkcs8.remove(0)
        } else {
            let file = File::open(key_filename)
                .unwrap_or_else(|e| panic!("Can't open tls key file: '{}'. {}.", key_filename, e));
            let mut rdr = BufReader::new(file);
            let mut rsa =
                rustls::internal::pemfile::rsa_private_keys(&mut rdr).unwrap_or_else(|()| {
                    panic!("Can't parse rsa_private tls key file: {}", key_filename)
                });

            if !rsa.is_empty() {
                rsa.remove(0)
            } else {
                panic!(
                    "TLS key path contains no private key. Check '{}' and '{}'.",
                    cert_filename, key_filename
                );
            }
        }
    };

    let mut tls = ServerConfig::new(rustls::NoClientAuth::new());
    tls.set_single_cert(cert, key).unwrap_or_else(|e| {
        panic!(
            "Can't set_single_cert: '{}',  '{}', {}.",
            cert_filename, key_filename, e
        )
    });
    tls.set_protocols(&["h2".into(), "http/1.1".into()]);
    tls
}
