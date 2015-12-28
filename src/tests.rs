use ::reader::{edn_write, edn_read};

//TODO: use quickcheck
#[test]
fn reflexive() {
  let e = "[:ns/kw \"LONG STRING\" :a \"\" (1 2 3 4M 1.23 .34 .23E+1 .23E)]";
  let r = edn_read(e.as_bytes()).unwrap();
  let w = edn_write(r);
  assert_eq!(e.to_string(), w);
}
