use ::reader::{edn_write, edn_read};
use std::fs::{read_dir, File};
use std::io::{Read};

#[test]
fn is_reversible() {
    for entry in read_dir("./edn-tests/valid-edn").unwrap() {
        let mut f = File::open(entry.unwrap().path()).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s);
        println!("Parsing {} to {:?}", s, edn_read(s.as_bytes()));
        assert!(edn_read(s.as_bytes()).is_ok())
    }
}
