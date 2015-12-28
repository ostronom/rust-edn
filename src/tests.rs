extern crate quickcheck;

use self::quickcheck::*;
use ::reader::{edn_write, edn_read};

#[derive(Clone, Debug)]
enum EdnNode {
    Vector(Vec<EdnNode>),
    List(Vec<EdnNode>),
    String(String),
    Bool(bool),
    Symbol(Option<String>, String),
    Keyword(Option<String>, String),
    Int(i64),
    Float(f64)
}

impl ToString for EdnNode {
    fn to_string(&self) -> String {
        match *self {
            EdnNode::Vector(ref xs) => {
                format!("[{}]", xs.into_iter().map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join(" "))
            },
            EdnNode::List(ref xs) => {
                format!("({})", xs.into_iter().map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join(" "))
            },
            EdnNode::String(ref s) => format!("\"{}\"", s.to_owned()),
            EdnNode::Bool(b) => if b { "true".to_string() }  else { "false".to_string() },
            EdnNode::Symbol(Some(ref ns), ref n) => format!("{}/{}", ns, n),
            EdnNode::Symbol(None, ref n) => format!("{}", n),
            EdnNode::Keyword(Some(ref ns), ref n) => format!(":{}/{}", ns, n),
            EdnNode::Keyword(None, ref n) => format!(":{}", n),
            EdnNode::Int(v) => format!("{}", v),
            EdnNode::Float(v) => format!("{}", v)
        }
    }
}

impl Arbitrary for EdnNode {
    fn arbitrary<G: Gen + Rng>(g: &mut G) -> Self {
        match g.next_u32() % 8 {
            0 => EdnNode::Vector(Arbitrary::arbitrary(g)),
            1 => EdnNode::List(Arbitrary::arbitrary(g)),
            2 => EdnNode::String(Arbitrary::arbitrary(g)),
            3 => EdnNode::Bool(Arbitrary::arbitrary(g)),
            4 => EdnNode::Symbol(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            5 => EdnNode::Keyword(Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            6 => EdnNode::Int(Arbitrary::arbitrary(g)),
            7 => EdnNode::Float(Arbitrary::arbitrary(g)),
            _ => panic!("WTF????")
        }
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        match *self {
            EdnNode::Vector(ref xs) => {
                let vec: Vec<EdnNode> = xs.clone().into_iter().collect();
                Box::new(vec.shrink().map(
                    |v| EdnNode::Vector(v.into_iter().collect::<Vec<EdnNode>>())
                ))
            },
            EdnNode::List(ref xs) => {
                let vec: Vec<EdnNode> = xs.clone().into_iter().collect();
                Box::new(vec.shrink().map(
                    |v| EdnNode::List(v.into_iter().collect::<Vec<EdnNode>>())
                ))
            },
            EdnNode::String(ref s) => {
                let ss: String = s.clone();
                Box::new(ss.shrink().map(|x| EdnNode::String(x)))
            },
            EdnNode::Bool(ref v) => {
                Box::new(v.shrink().map(|x| EdnNode::Bool(x)))
            },
            EdnNode::Symbol(ref ns, ref n) => {
                let pair = (ns.clone(), n.clone());
                Box::new(pair.shrink().map(|(x,y)| EdnNode::Symbol(x, y)))
            },
            EdnNode::Keyword(ref ns, ref n) => {
                let pair = (ns.clone(), n.clone());
                Box::new(pair.shrink().map(|(x,y)| EdnNode::Keyword(x, y)))
            },
            EdnNode::Int(ref n) => {
                let ns = n.clone();
                Box::new(n.shrink().map(|x| EdnNode::Int(x)))
            },
            EdnNode::Float(ref n) => {
                let ns = n.clone();
                Box::new(n.shrink().map(|x| EdnNode::Float(x)))
            }
        }
    }
}

#[test]
fn is_reversible() {
    fn prop(input: EdnNode) -> bool {
        let e = input.to_string();
        println!("TRYING TO PARSE {}", e);
        let r = edn_read(e.as_bytes()).unwrap();
        let w = edn_write(r);
        e == w
    }
    quickcheck(prop as fn(EdnNode) -> bool);
}
