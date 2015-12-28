use ::types::*;
use std::str::{from_utf8};
use std::collections::{LinkedList};

const ESCAPE_CHAR: u8 = 92;

#[derive(Debug, PartialEq)]
enum Lexeme { String, Atom, VecParen, MapParen, ListParen }

#[derive(Debug)]
struct LexedNode<'a> {
    lexeme: Lexeme,
    value: &'a [u8]
}

fn get_matching_paren(c: &Lexeme) -> Option<u8> {
    match *c {
        Lexeme::ListParen => Some(b')'),
        Lexeme::MapParen  => Some(b'}'),
        Lexeme::VecParen  => Some(b']'),
        _                 => None
    }
}

fn get_paren_lexeme(c: u8) -> Option<Lexeme> {
    match c {
        b'(' | b')' => Some(Lexeme::ListParen),
        b'{' | b'}' => Some(Lexeme::MapParen),
        b'[' | b']' => Some(Lexeme::VecParen),
        _           => None
    }
}

fn is_digit(c: u8) -> bool {
    match c {
        48...57 => true,
        _ => false
    }
}

fn is_paren(c: &u8) -> bool {
    match *c {
        b'(' | b')' | b'[' | b']' | b'{' | b'}' => true,
        _ => false
    }
}

fn is_whitespace(c: &u8) -> bool {
   match *c {
       b'\t' | b'\n' | b'\r' | b' ' | b',' => true,
       _ => false
   }
}

fn push_lex<'a>(input: &'a [u8], start: usize, end: usize, lexeme: Lexeme, stack: &mut LinkedList<LexedNode<'a>>) {
    let z = &input[start .. end];
    //println!("lexing {:?} as {:?}", String::from_utf8(z.to_vec()), lexeme);
    stack.push_back(LexedNode{lexeme: lexeme, value: z});
}

fn lex<'a>(s: &'a [u8]) -> Result<LinkedList<LexedNode<'a>>, &str> {
    let mut tokens: LinkedList<LexedNode<'a>> = LinkedList::new();
    let mut escaping = false;
    let mut in_string = false;
    let mut in_comment = false;
    let mut token: usize = 0;
    let mut pos: usize = 0;
    for c in s {
        if !in_string && *c == b';' && !escaping { in_comment = true; }
        if in_comment {
            if *c == b'\n' {
                in_comment = false;
                if token != pos {
                    push_lex(s, token, pos, Lexeme::Atom, &mut tokens);
                    token = pos;
                }
                pos += 1;
                continue;
            }
        }

        if *c == b'"' && !escaping {
            if in_string {
                push_lex(s, token, pos, Lexeme::String, &mut tokens);
                in_string = false;
            } else {
                in_string = true;
            }
            pos += 1;
            token = pos;
            continue;
        }

        if in_string {
            // if c == ESCAPE_CHAR && !escaping {
            //     escaping = true;
            //     // content.1 += 1;
            //     // continue;
            // }
            //
            // if escaping {
            //     escaping = false;
            //     // if c == b't' || c == b'n' || c == b'f' || c == b'r' {
            //     //     //content.push(ESCAPE_CHAR as char);
            //     //
            //     // }
            // }
            // content.push(c);
        } else if is_paren(c) || is_whitespace(c) {
            if token != pos {
                push_lex(s, token, pos, Lexeme::Atom, &mut tokens);
                token = pos;
            }
            if is_paren(c) {
                match get_paren_lexeme(*c) {
                    Some(l) => push_lex(s, token, pos, l, &mut tokens),
                    None    => return Err("WTF?")
                }
            }
            token += 1;
            // if is_whitespace(c) { token += 1; }
        } else {
            if escaping { escaping = false; }
            else if *c == ESCAPE_CHAR { escaping = true; }

            // if token == "#_" || (token.len() == 2 && token.as_bytes()[0] == ESCAPE_CHAR) {
            //     tokens.push_back(LexedNode{lexeme: Lexeme::Atom, value: token.clone()});
            //     token.clear();
            // }

            let diff = pos - token;
            if (diff > 0 && s[token .. pos] == [b'#', b'_']) || (diff == 2 && s[token] == ESCAPE_CHAR) {
                push_lex(s, token, pos, Lexeme::Atom, &mut tokens);
                token = pos;
            }
        }
        pos += 1;
    }
    if token != pos {
        push_lex(s, token, pos, Lexeme::Atom, &mut tokens);
    }

    Ok(tokens)
}

fn handle_collection<'a>(token: &LexedNode, collection: Vec<Node<'a>>) -> Result<Node<'a>, &'a str> {
    match token.lexeme {
        Lexeme::ListParen => Ok(Node::List(collection)),
        Lexeme::VecParen  => Ok(Node::Vector(collection)),
        Lexeme::MapParen  => Ok(Node::Map),
        _                 => Err("WTF?")
    }
}

fn handle_tagged<'a>(token: &'a LexedNode, next: &'a LexedNode) -> Node<'a> {
    Node::Tagged(&token.value, &next.value)
}

// TODO: use .split_at to split namespace/name
fn parse_symbol(s: &[u8]) -> Option<(Option<&[u8]>, &[u8])> {
    let mut l: usize = s.len();
    let mut slash_pos: usize = 0;
    if l == 0 || is_digit(s[0]) || s[0] == b':' || s[0] == b'#' || (s[0] == b'/' && l != 1) ||
       (l > 1 && (s[0] == b'-' || s[0] == b'+' || s[0] == b'.') && is_digit(s[1]))
    {
        return None
    }
    while l > 0 {
        let c = s[l-1];
        let v = match c {
            b'a' ... b'z' | b'A' ... b'Z' | b'.' | b'*' | b'+' | b'!' | b'-' | b'_' | b'?' |
            b'$' | b'%' | b'&' | b'=' | b':' | b'#' | b'/' => true,
            _ => false
        };
        if !is_digit(c) && !v { return None }
        if c == b'/' {
            if slash_pos != 0 { return None }
            slash_pos = l;
        }
        l -= 1;
    }
    Some((if slash_pos > 0 { Some(&s[0..(slash_pos-1)]) } else { None }, &s[slash_pos..]))
}

fn parse_keyword(s: &[u8]) -> Option<(Option<&[u8]>, &[u8])> {
    if s.len() == 0 || s[0] != b':' { return None }
    parse_symbol(&s[1..])
}

fn valid_bool(s: &[u8]) -> bool {
    s == [b't', b'r', b'u', b'e'] || s == [b'f', b'a', b'l', b's', b'e']
}

fn parse_unsigned_int(s: &[u8], allow_precision: bool) -> Option<(&[u8], bool)> {
    let len = s.len();
    let last = s.last();
    if len == 0 { return None }
    if *(s.last().unwrap()) == b'N' {
        if allow_precision {
            return parse_unsigned_int(&s[..(len-1)], false).map(|(x,y)| (x, true))
        } else {
            return None
        }
    }
    if s.into_iter().all(|x| is_digit(*x)) {
        Some((s, false))
    } else {
        None
    }
}

fn parse_int(s: &[u8], allow_precision: bool) -> Option<(&[u8], bool)> {
    if s.len() > 0 && (s[0] == b'-' || s[0] == b'+') {
        match parse_unsigned_int(&s[1..], allow_precision) {
            Some((_, p)) => Some((s, p)),
            None => None
        }
    } else {
        parse_unsigned_int(s, allow_precision)
    }
}

fn parse_float(s: &[u8], allow_precision: bool) -> Option<(&[u8], &[u8], &[u8], bool)> {
    let len = s.len();
    if len == 0 { return None; }
    if *(s.last().unwrap()) == b'M' {
        if allow_precision {
            match parse_float(&s[..(len-1)], false) {
                Some((x,y,z,_)) => return Some((x,y,z,true)),
                _ => return None
            }
        } else { return None; }
    }
    match s.iter().position(|x| *x == b'.') {
        Some(period_position) => {
            let (integral, rest) = s.split_at(period_position);
            let integral_int = parse_int(integral, false);
            if integral.len() != 0 && integral_int.is_none() { return None; }
            match rest[1..].iter().position(|x| *x == b'E') {
                Some(exp_pos) if exp_pos == rest.len() => None,
                Some(exp_pos) => {
                    let (fraction, exponent) = rest[1..].split_at(exp_pos);
                    let fraction_int = parse_unsigned_int(fraction, false);
                    let exponent_int = parse_int(&exponent[1..], false);
                    match (integral_int, fraction_int, exponent_int) {
                        (Some((i, _)), Some((f, _)), Some((e, _))) => Some((i, f, e, false)),
                        (None, Some((f, _)), Some((e, _))) => Some((&[], f, e, false)),
                        _ => None
                    }
                }
                None => {
                    let fraction_int = parse_unsigned_int(&rest[1..], false);
                    match (integral_int, fraction_int) {
                        (Some((i, _)), Some((f, _))) => Some((i, f, &[], false)),
                        (None, Some((f, _))) => Some((&[], f, &[], false)),
                        _ => None
                    }
                }
            }
        },
        _ =>  None
    }
}

fn handle_atom<'a>(token: &LexedNode<'a>) -> Result<Node<'a>, &'a str> {
    match token.lexeme {
        Lexeme::Atom if token.value == &[b'n',b'i',b'l'] => Ok(Node::Nil),
        Lexeme::Atom => if let Some((ns, name)) = parse_keyword(token.value) {
            Ok(Node::Keyword(ns, name))
        } else if let Some((ns, name)) = parse_symbol(token.value) {
            Ok(Node::Symbol(ns, name))
        } else if valid_bool(token.value) {
            Ok(Node::Bool(token.value))
        } else if let Some((v, p)) = parse_int(token.value, true) {
            Ok(Node::Int(v, p))
        } else if let Some((i, f, e, p)) = parse_float(token.value, true) {
            Ok(Node::Float(i, f, e, p))
        } else {
            println!("{}", String::from_utf8(token.value.to_owned()).unwrap());
            Err("Could not parse atom")
        },
        Lexeme::String => Ok(Node::String(token.value)),
        _ => {
            println!("{}", String::from_utf8(token.value.to_owned()).unwrap());
            Err("Could not parse atom")
        }
    }
}

fn read_ahead<'a>(token: &LexedNode<'a>, tokens: &mut LinkedList<LexedNode<'a>>) -> Result<Node<'a>, &'a str> {
    match get_matching_paren(&token.lexeme) {
        Some(_) => {
            let mut collection: Vec<Node> = Vec::new();
            loop {
                match tokens.pop_front() {
                    Some(ref next) => {
                        if next.lexeme == token.lexeme {
                            return handle_collection(token, collection)
                        } else {
                            match read_ahead(next, tokens) {
                                Ok(n)  => collection.push(n),
                                Err(e) => return Err(e)
                            }
                        }
                    },
                    None => return Err("Unexpected EOF")
                }
            }
        },
        None => {
            // if token.value.len() > 0 && token.value[0] == b'#' {
            //     match tokens.pop_front() {
            //         Some(ref next) => match read_ahead(next, tokens) {
            //             Ok(ref n)  => Ok(handle_tagged(&token, &next)),
            //             Err(e) => Err(e)
            //         },
            //         None => Err("No value after tag")
            //     }
            // } else {
            handle_atom(token)
            // }
        }
    }
    // Ok(Node::String("FUCK".to_string()))
}

pub fn edn_read<'a>(s: &'a [u8]) -> Result<Node<'a>, &'a str> {
    // Ok(Node::String("FUCK".as_bytes()))
    // println!("{:?}", tokens);
    let mut tokens = lex(s);
    match tokens {
        Err(e) => Err(e),
        Ok(ref mut tokens) => match tokens.pop_front() {
            Some(ref token) => read_ahead(token, tokens),
            None            => Err("No parseable tokens found")
        }
    }
}

pub fn edn_write(node: Node) -> String {
    node.to_string()
}
