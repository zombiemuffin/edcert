use certificate::Certificate;

struct CertificateLoader;

impl CertificateLoader {
    /// Saves this certificate into a folder: one file for the certificate and one file for the
    /// private key.
    pub fn save(cert: &Certificate, folder: &str) {
        use std::fs::File;
        use std::fs::DirBuilder;
        use std::fs::metadata;
        use std::io::Write;

        let folder: String = folder.to_string();

        if metadata(&folder).is_err() {
            DirBuilder::new().create(&folder).expect("Failed to create folder");
        }

        if cert.has_private_key() {
            let mut private_keyfile: File = File::create(folder.clone() + "/private.key")
                                                .expect("Failed to create private key file.");
            let bytes: &[u8] = cert.get_private_key().unwrap();
            private_keyfile.write_all(bytes).expect("Failed to write private key file.");
        }

        let folder: String = folder.to_string();
        let mut certificate_file: File = File::create(folder + "/certificate.ec")
                                             .expect("Failed to create certificate file.");

        let compressed = cert.to_json();
        certificate_file.write(&*compressed)
                        .expect("Failed to write certificate file.");
    }

    /// This method loads a certificate from a file.
    pub fn load_from_file(filename: &str) -> Result<Certificate, &'static str> {
        use std::fs::File;
        use std::io::Read;

        let filename: String = filename.to_string();
        let mut certificate_file: File = File::open(filename)
                                             .expect("Failed to open certificate file.");
        let mut compressed = Vec::new();
        certificate_file.read_to_end(&mut compressed).expect("Failed to read certificate");
        Certificate::from_json(&*compressed)
    }

    /// This method reads a private key from a file and sets it in this certificate.
    pub fn load_private_key(cert: &mut Certificate, filename: &str) -> Result<(), &'static str> {
        use std::fs::File;
        use std::io::Read;

        let filename: String = filename.to_string();
        let mut private_key_file: File = File::open(filename)
                                             .expect("Failed to open private kye file.");
        let mut private_key = Vec::new();
        private_key_file.read_to_end(&mut private_key).expect("Failed to read private key");

        cert.set_private_key(private_key);

        Ok(())
    }
}

#[test]
fn test_save() {
    use ed25519;
    use meta::Meta;
    use chrono::Timelike;
    use chrono::UTC;
    use time::Duration;
    use certificate_verificator::CertificateVerificator;
    use certificate_verificator::NoRevoker;

    let (mpk, msk) = ed25519::generate_keypair();
    let cv = CertificateVerificator::new(&mpk, NoRevoker);

    let meta = Meta::new_empty();
    let expires = UTC::now()
                      .checked_add(Duration::days(90))
                      .expect("Failed to add a day to expiration date.")
                      .with_nanosecond(0)
                      .unwrap();

    let mut cert = Certificate::generate_random(meta, expires);

    cert.sign_with_master(&msk);

    assert_eq!(true, cv.is_valid(&cert).is_ok());

    CertificateLoader::save(&cert, &expires.to_rfc3339());
    let mut cert = CertificateLoader::load_from_file(&format!("{}{}", &expires.to_rfc3339(), "/certificate.ec")).expect("Failed to load certificate from file");
    CertificateLoader::load_private_key(&mut cert, &format!("{}{}", &expires.to_rfc3339(), "/private.key")).expect("Failed to load private key from file");

    assert_eq!(true, cv.is_valid(&cert).is_ok());
}
