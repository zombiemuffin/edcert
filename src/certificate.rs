use meta::Meta;
use signature::Signature;
use rustc_serialize::json;
use rustc_serialize::Encodable;
use rustc_serialize::Encoder;
use rustc_serialize::Decoder;
use chrono;
use ed25519;
use lzma;
use chrono::UTC;

/// This is the amount of bytes a single ed25519 signature takes up
pub const SIGNATURE_LEN: usize = 64 + CERTIFICATE_BYTE_LEN;

/// This is the amount of bytes a single key takes up.
pub const PRIVATE_KEY_LEN: usize = 64;
pub const PUBLIC_KEY_LEN: usize = 32;

pub const CERTIFICATE_BYTE_LEN: usize =
    25 /* expires as string */ + 64 /* hash of meta */ + PUBLIC_KEY_LEN;

#[derive(Clone,RustcDecodable,RustcEncodable,Debug)]
pub struct Certificate {
    /// The meta element contains data associated with the certificate
    /// Common data is "use-for" which contains a list of permissions
    meta: Meta,

    /// the public key of this certificate
    public_key: Vec<u8>,

    /// the private key, if it is known
    private_key: Option<Vec<u8>>,

    /// a timestamp when this certificate expires
    expires: String,

    /// a signature for trust-chaining certificates
    /// if the certificate is not signed yet, this is None
    signature: Option<Signature>,
}

impl Certificate {
    //    pub fn new(public_key: Vec<u8>,
    //               expires: chrono::DateTime<chrono::UTC>)
    //               -> Certificate {
    //        Certificate {
    //            meta: Meta::new(),
    //            signature: None,
    //            public_key: public_key,
    //            private_key: None,
    //            expires: expires,
    //        }
    //    }

    pub fn get_meta_mut(&mut self) -> &mut Meta {
        &mut self.meta
    }

    pub fn get_meta(&self) -> &Meta {
        &self.meta
    }

    pub fn get_public_key(&self) -> &Vec<u8> {
        &self.public_key
    }

    /// This method returns the private key, if it is known, or None if the certificate has been initialized without the private key
    pub fn get_private_key(&self) -> &Option<Vec<u8>> {
        &self.private_key
    }

    /// This method returns true, if the private key is saved in the certificate
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    pub fn get_expires(&self) -> &str {
    	&self.expires
    }

    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// This method signs the given data and returns the signature
    pub fn sign(&self, data: &[u8]) -> Option<Vec<u8>> {
        if self.has_private_key() {

            let signature = ed25519::sign(data, &self.private_key.as_ref().unwrap());
            Some(signature)

        } else {
            None
        }
    }

    pub fn sign_with_master(&mut self, master_private_key: &Vec<u8>) {
        let bytes = self.as_bytes();
        let hash = ed25519::sign(&bytes, master_private_key);
        self.signature = Some(Signature::new_without_parent(hash));
    }

    /// This method verifies that the given signature is valid for the given data
    pub fn verify(&self, data: &[u8], _: usize, signature: &Vec<u8>) -> bool {
        let result = ed25519::verify(data, &signature, &self.public_key);
        result
    }

    pub fn as_bytes(&self) -> [u8; CERTIFICATE_BYTE_LEN] {
        let mut bytes = [0; CERTIFICATE_BYTE_LEN];

        self.meta.fill_bytes(&mut bytes[0..64]);
        copy_bytes(&mut bytes[64..],
                   self.expires.as_bytes(),
                   0,
                   0,
                   25);
        copy_bytes(&mut bytes[89..], &*self.public_key, 0, 0, PUBLIC_KEY_LEN);

        bytes
    }

    pub fn get_parent(&self) -> Option<&Certificate> {
        if self.signature.is_some() {
            let sig = &self.signature.as_ref().unwrap();
            sig.get_parent()
        } else {
            None
        }
    }

    /// This method verifies that this certificate is valid by analyzing the trust chain
    pub fn is_valid(&self, master_pk: &Vec<u8>) -> Result<(), &'static str> {
        if !self.is_signed() {
            Err("This certificate isn't signed, so it can't be valid.")
        } else {
            let bytes: [u8; CERTIFICATE_BYTE_LEN] = self.as_bytes();

            let signature = self.signature.as_ref().expect("lel");

            if signature.is_signed_by_master() {

                let r = ed25519::verify(&bytes, signature.get_hash(), master_pk);

                if r {

                    // FIXME: hier expires checken

                    Ok(())
                } else {
                    Err("Failed to verify master signature")
                }

            } else {
                let parent: &Certificate = signature.get_parent().unwrap();
                let sign_real = parent.verify(&bytes, CERTIFICATE_BYTE_LEN, &signature.get_hash());
                let parent_real = parent.is_valid(&master_pk).is_ok();

                if sign_real {
                    if parent_real {

                        let expires = match chrono::DateTime::parse_from_rfc3339(&self.expires) {
                        	Err(_) => return Err("Failed to parse expiration time"),
                        	Ok(expires) => expires.with_timezone(&chrono::UTC),
                        };

                        if expires > chrono::UTC::now() {
                        	Ok(())
						} else {
							Err("The certificate is expired")
						}
                    } else {
                        Err("The parent is invalid.")
                    }
                } else {
                    Err("The signature of the parent isn invalid.")
                }
            }
        }
    }

    /// This method generates a random public/private keypair and a certificate for it
    pub fn generate_random(meta: Meta, expires: chrono::DateTime<chrono::UTC>) -> Certificate {

        let (public_key, private_key) = ed25519::generate_keypair();

        Certificate {
            private_key: Some(private_key),
            public_key: public_key,
            expires: expires.to_rfc3339(),
            meta: meta,
            signature: None,
        }
    }

    pub fn as_json(&self) -> Vec<u8> {
    	let jsoncode = json::encode(self).expect("Failed to encode certificate");
		let mut compressed = lzma::compress(&jsoncode.as_bytes(), 6).expect("failed to compress");
		let magic = "edcert".as_bytes();
		copy_bytes(&mut compressed[0..6], magic, 0, 0, 6);
		compressed
    }

    /// Saves this certificate into a folder: one file for the certificate and one file for the private key
    pub fn save(&self, folder: &str) {
        use std::fs::File;
        use std::fs::DirBuilder;
        use std::fs::metadata;
        use std::io::Write;

        let folder: String = folder.to_string();

        if metadata(&folder).is_err() {
            DirBuilder::new().create(&folder).expect("Failed to create folder");
        }

        if self.has_private_key() {
            let mut private_keyfile: File = File::create(folder.clone() + "/private.key")
                                                .expect("Failed to create private key file.");
            let bytes: &[u8] = self.private_key.as_ref().unwrap();
            private_keyfile.write_all(bytes).expect("Failed to write private key file.");
        }

        let folder: String = folder.to_string();
        let mut certificate_file: File = File::create(folder + "/certificate.ec")
                                             .expect("Failed to create certificate file.");

		let compressed = self.as_json();
        certificate_file.write(&*compressed)
                        .expect("Failed to write certificate file.");
    }

	pub fn load(compressed : &[u8]) -> Result<Certificate, &'static str> {

		let mut bytes : Vec<u8> = Vec::new();
		let magic : [u8; 6] = [0xfd, 0x37, 0x7a, 0x58, 0x5a, 0x00];
		bytes.extend(compressed);
		copy_bytes(&mut bytes[0..7], &magic,0,0,6);

		let o = lzma::decompress(&bytes[..]);
		if o.is_err() {
			return Err("Failed to decompress certificate");
		}

		let o = String::from_utf8(o.unwrap());
		if o.is_err() {
			return Err("Failed to read UTF8 from decompressed vector");
		}

        let o = json::decode(&o.unwrap());
        if o.is_err() {
        	Err("Failed to decode JSON")
        }
        else
        {
        	Ok(o.unwrap())
        }
	}

    /// This method loads a certificate from a folder
    pub fn load_from_folder(folder: &str) -> Result<Certificate, &'static str> {

        use std::fs::File;
        // use std::fs::DirBuilder;
        use std::io::Read;

        let folder: String = folder.to_string();
        let mut certificate_file: File = File::open(folder + "/certificate.ec")
                                             .expect("Failed to open certificate file.");
        let mut compressed = Vec::new();
        certificate_file.read_to_end(&mut compressed).expect("Failed to read certificate");
		Certificate::load(&*compressed)
    }

    /// This method signs another certificate with the private key of this certificate
    pub fn sign_certificate(&self, other: &mut Certificate) -> Result<(), &'static str> {

        if self.has_private_key() {
            let child_bytes = other.as_bytes();
            let signature_bytes = self.sign(&child_bytes).unwrap().to_vec();
            let parent = Box::new(self.clone());
            let signature = Signature::new(parent, signature_bytes);

            other.signature = Some(signature);

            Ok(())
        } else {
            Err("This certificate has no private key")
        }
    }
}

/// This is a simple copy function. This should be replaced by memcpy or something...
fn copy_bytes(dest: &mut [u8], src: &[u8], start_dest: usize, start_src: usize, len: usize) {
    for i in 0..len {
        dest[start_dest + i] = src[start_src + i];
    }
}

#[test]
fn test_generate_certificate() {

	use chrono::Timelike;
	use time::Duration;

	let meta = Meta::new_empty();
    let expires = UTC::now()
                  .checked_add(Duration::days(90))
                  .expect("Fehler: Ein Tag konnte nicht auf heute addiert werden.")
                  .with_nanosecond(0)
                  .unwrap();

	let a = Certificate::generate_random(meta, expires);

	let meta = Meta::new_empty();

	let b = Certificate::generate_random(meta, expires);

	assert!(a.get_public_key() != b.get_public_key());
}

#[test]
fn test_all() {

	use chrono::Timelike;
	use time::Duration;

	let mut meta_parent = Meta::new_empty();
    meta_parent.set("name", "Amke Root Certificate");
    meta_parent.set("use-for", "[amke.certificate-signing]");
    let meta_parent = meta_parent;

    let mut meta_child = Meta::new_empty();
    meta_child.set("name", "Amke Rombie Root Certificate");
    meta_child.set("use-for", "[amke.certificate-signing, amke.rombie.*]");
    let meta_child = meta_child;

    let expires = UTC::now()
                      .checked_add(Duration::days(90))
                      .expect("Fehler: Ein Tag konnte nicht auf heute addiert werden.")
                      .with_nanosecond(0)
                      .unwrap();

    let (master_pk, master_sk) = ed25519::generate_keypair();

    let mut child = Certificate::generate_random(meta_child, expires);
    let mut parent = Certificate::generate_random(meta_parent, expires);

    parent.sign_with_master(&master_sk);

    parent.sign_certificate(&mut child).expect("Failed to sign child!");

    let time_str = UTC::now().with_nanosecond(0).unwrap().to_rfc3339();

    println!("Ist Child valid? {:?}", child.is_valid(&master_pk));

    child.save(&time_str);

    child = Certificate::load_from_folder(&time_str).expect("Failed to load certificate");

    println!("Ist Child nach laden noch valid? {:?}",
             child.is_valid(&master_pk));

    child.save(&time_str);
}