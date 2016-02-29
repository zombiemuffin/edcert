use sodiumoxide::crypto::sign::ed25519;

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {

    let (pk, sk) = ed25519::gen_keypair();

    let private_key = bytes_to_vec(&sk.0);
    let public_key = bytes_to_vec(&pk.0);

	assert_eq!(64, private_key.len());
    assert_eq!(32, public_key.len());

    (public_key, private_key)
}

pub fn sign(data: &[u8], private_key: &Vec<u8>) -> Vec<u8> {

    assert_eq!(64, private_key.len());

    let sk = ed25519::SecretKey(vec_to_bytes64(private_key));

    let s = ed25519::sign(data, &sk);

    s
}

pub fn verify(data: &[u8], signature: &[u8], public_key: &Vec<u8>) -> bool {
    let b = vec_to_bytes32(public_key);
    let pk = ed25519::PublicKey::from_slice(&b);
    let pk = pk.as_ref().unwrap();

    let r = ed25519::verify(&signature, &pk);

    if r.is_err() {
        false
    } else {
        let bytes = r.unwrap();
        bytes == data
    }
}


fn bytes_to_vec(a: &[u8]) -> Vec<u8> {
    Vec::from(a)
}

fn vec_to_bytes64(a: &Vec<u8>) -> [u8; 64] {

    let mut r = [0; 64];
    let mut i = 0;

    for b in a {
        //        if i >= 64 {
        //            break;
        //        }

        r[i] = b.clone();
        i += 1;
    }

    r
}

fn vec_to_bytes32(a: &Vec<u8>) -> [u8; 32] {

    let mut r = [0; 32];
    let mut i = 0;

    for b in a {
        //        if i >= 32 {
        //            break;
        //        }
        r[i] = b.clone();
        i += 1;
    }

    r
}

#[test]
fn test_ed25519_simple() {
    let (pk, sk) = generate_keypair();

    println!("public: {:?}, private: {:?}", pk, sk);
    println!("");

    let msg = [0; 128];
    let mut sig = sign(&msg, &sk);

    println!("signature: {:?}", sig);
    println!("");

    sig[0] -= 1;

    assert_eq!(verify(&msg, &sig, &pk), false);

    sig[0] += 1;

    assert_eq!(verify(&msg, &sig, &pk), true);
}

#[test]
fn test_ed25519_shortmsg() {
    let (pk, sk) = generate_keypair();

    println!("public: {:?}, private: {:?}", pk, sk);
    println!("");

    let msg = [0; 32];
    let mut sig = sign(&msg, &sk);

    println!("signature: {:?}", sig);
    println!("");

    sig[0] -= 1;

    assert_eq!(verify(&msg, &sig, &pk), false);

    sig[0] += 1;

    assert_eq!(verify(&msg, &sig, &pk), true);
}
