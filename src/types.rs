use std::str::{from_utf8, Utf8Error};
use std::error::Error;

#[derive(Debug)]
pub enum Node<'a> {
    Nil,
    Symbol(Option<&'a [u8]>, &'a[u8]), // namespace/name
    Keyword(Option<&'a [u8]>, &'a[u8]), // namespace/name
    Bool(&'a [u8]), //Bool(bool),
    Int(&'a [u8], bool), //Int(i64), N-precision
    Float(&'a [u8], &'a [u8], &'a [u8], bool), // integral, fraction, exponent, M-precision
    Char(char),
    String(&'a [u8]), //String(&'a String),
    List(Vec<Node<'a>>),
    Vector(Vec<Node<'a>>),
    Map, // ??
    Set(Vec<Node<'a>>),
    Tagged(&'a [u8], &'a [u8]), // tagName/tagValue
    Discard(Box<Node<'a>>)
}

fn coll_to_str<'a>(v: &Vec<Node<'a>>) -> String {
    v.into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join(" ")
}

impl<'a> ToString for Node<'a> {
    // TODO: remove unwrap here
    fn to_string(&self) -> String {
        match *self {
            Node::Nil => "nil".to_string(),
            Node::Bool(v) => from_utf8(v).unwrap().to_owned(),
            Node::Int(v, precision) => {
                let vs = from_utf8(v).unwrap().to_owned();
                if precision { format!("{}M", vs) } else { format!("{}", vs) }
            },
            Node::String(v)  => format!("\"{}\"",from_utf8(v).unwrap().to_owned()),
            Node::Keyword(Some(ns), name) => format!(":{}/{}",
                                                     from_utf8(ns).unwrap().to_owned(),
                                                     from_utf8(name).unwrap().to_owned()),
            Node::Keyword(None, name) => format!(":{}", from_utf8(name).unwrap().to_owned()),
            Node::Vector(ref v) => format!("[{}]", coll_to_str(v)),
            Node::List(ref v) => format!("({})", coll_to_str(v)),
            _ => format!("{:?}", self)
        }
    }
}
