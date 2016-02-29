extern crate chrono;
extern crate time;
extern crate crypto;
extern crate rustc_serialize;
extern crate rand;
extern crate sodiumoxide;
extern crate lzma;

pub mod ed25519;
pub mod meta;
pub mod signature;
pub mod certificate;

use certificate::Certificate;

fn main() {
	let stdin = std::io::stdin();
	let mut line = String::new();
	stdin.read_line(&mut line).expect("Failed to read line");
	
	let line = line.trim();
	
	if line == "gen-master" {
		
		use std::io::Write;
		
		println!("Generiere master keypair");
		
		let (pubkey, prvkey) = ed25519::generate_keypair();
		
		std::fs::DirBuilder::new().create("master").expect("Failed to create folder 'master'");
	
		let mut prvfile = std::fs::File::create("master/master.prv").expect("Failed to create private keyfile");
		let mut pubfile = std::fs::File::create("master/master.pub").expect("Failed to create public keyfile");
		
		pubfile.write_all(&*pubkey).expect("Failed to write public key");
		prvfile.write_all(&*prvkey).expect("Failed to write public key");
	} else if line == "gen-cert" {
		
	}
	
	println!("Beende");
}

fn info(master_pk : Vec<u8>, cert : &Certificate) {
	
	use std::collections::BTreeMap;
	
	println!("Metadaten:");
	
	let values : &BTreeMap<String, String> = cert.get_meta().get_values();
	
	for a in values {
		println!("{}: {}", a.0, a.1);
	}
	
	println!("Läuft ab: {0}", cert.get_expires());
	
	let reason = cert.is_valid(&master_pk);
	
	match reason {
		Ok(_) => {
			println!("Zertifikat ist gültig!");
		},
		Err(string) => {
			println!("Zertifikat ungültig: {}", string);
		}
	}
}