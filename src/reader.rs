use ::types::*;
use std::collections::{LinkedList};

const ESCAPE_CHAR: u8 = 92;

#[derive(Debug, PartialEq)]
enum Lexeme { String, Atom, VecParen, MapParen, ListParen }

#[derive(Debug)]
struct LexedNode<'a> {
    lexeme: Lexeme,
    value: &'a [u8]
}

fn get_matching_paren(c: Lexeme) -> Option<u8> {
    match c {
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
    println!("lexing {:?} as {:?}", String::from_utf8(z.to_vec()), lexeme);
    stack.push_back(LexedNode{lexeme: lexeme, value: z});
}

fn lex<'a>(s: &'a [u8]) -> Result<LinkedList<LexedNode>, &str> {
    let mut tokens: LinkedList<LexedNode> = LinkedList::new();
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
//
// fn handle_collection<'a>(token: &LexedNode, collection: Vec<&'a Node<'a>>) -> Node<'a> {
//     let start = token.value.as_bytes()[0] as char;
//     if start == '(' {
//         Node::List(collection)
//     } else if start == '[' {
//         Node::Vector(collection)
//     } else {
//         Node::Map
//     }
// }
//
// fn handle_tagged<'a>(token: &'a LexedNode, next: &'a LexedNode) -> Node<'a> {
//     Node::Tagged(&token.value, &next.value)
// }
//
// fn handle_atom<'a>(token: &LexedNode) -> Node<'a> {
//     Node::Nil
// }
//
// fn read_ahead<'a>(token: &'a LexedNode, tokens: &mut LinkedList<LexedNode>) -> Result<Node<'a>, String> {
//     if token.lexeme == Lexeme::Paren {
//         let closing_paren = get_matching_paren(token.value.as_bytes()[0]);
//         let mut collection: Vec<&'a Node> = Vec::new();
//         loop {
//             match tokens.pop_front() {
//                 Some(next) => {
//                     if next.lexeme == Lexeme::Paren && next.value.as_bytes()[0] == closing_paren {
//                         return Ok(handle_collection(token, collection))
//                     } else {
//                         match read_ahead(&next, tokens) {
//                             Ok(ref n) => collection.push(n),
//                             Err(e)    => return Err(e)
//                         }
//                     }
//                 },
//                 None => return Err("Unexpected EOF".to_string())
//             }
//         }
//     } else {
//         if token.value.len() > 0 && token.value.as_bytes()[0] == ('#' as u8) {
//             match tokens.pop_front() {
//                 Some(next) => match read_ahead(&next, tokens) {
//                     Ok(ref n)  => Ok(handle_tagged(&token, &next)),
//                     Err(e) => Err(e)
//                 },
//                 None => Err("No value after tag".to_string())
//             }
//         } else {
//             Ok(handle_atom(token))
//         }
//     }
//     //Ok(Node::String("FUCK".to_string()))
// }

pub fn edn_read(s: &[u8]) -> Result<Node, String> {
    let mut tokens = lex(s);
    Ok(Node::String("FUCK".as_bytes()))
    // println!("{:?}", tokens);
    // match tokens.pop_front() {
    //     Some(ref token) => read_ahead(&token, &mut tokens),
    //     None            => Err("No parseable tokens found".to_string())
    // }
}

pub fn edn_write(node: Node) -> Result<&str, &str> {
    to_string(node)
}
