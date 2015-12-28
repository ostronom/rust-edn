use std::str::{from_utf8, Utf8Error};
use std::error::Error;

#[derive(Debug)]
pub enum Node<'a> {
    Nil,
    Symbol(&'a [u8]),
    Keyword(Option<&'a [u8]>, &'a[u8]), // namespace/name
    Bool(&'a [u8]), //Bool(bool),
    Int(&'a [u8]), //Int(i64),
    Float(&'a [u8]), //Float(f64),
    Char(char),
    String(&'a [u8]), //String(&'a String),
    List(Vec<Node<'a>>),
    Vector(Vec<Node<'a>>),
    Map, // ??
    Set(Vec<Node<'a>>),
    Tagged(&'a [u8], &'a [u8]), // tagName/tagValue
    Discard(Box<Node<'a>>)
}

pub fn to_string<'a>(node: Node<'a>) -> Result<&'a str, &'a str> {
    match node {
        Node::Nil        => Ok("nil"),
        Node::Bool(v)    => from_utf8(v).map_err(|_| "UTF8 decoding failure"),
        Node::String(v)  => from_utf8(v).map_err(|_| "UTF8 decoding failure"),
        //Node::Keyword(Some(ns), name) => ns.map().or("")
        _                => { println!("{:?}", node); Err("UNKNOWN") }
    }
}
