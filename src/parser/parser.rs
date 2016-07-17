pub use super::lexer::{Token, LexError};
use super::lexer::{Lexer, StringIter};
use ::core::obj::{LispObj, AsLispObjRef};

use std::convert::Into;
use std::io::{self, Read};
use std::fmt;
use std::fs::File;

#[derive(Debug)]
pub enum ParserError<E> {
    // unexpected, expecting, line no, col no
    UnexpectedToken(Token, Option<Token>, u32, u32),
    UnexpectedEndOfInput(ParserState, u32, u32),
    UnexpectedDelimiter(Token, u32, u32),
    LexError(LexError<E>),
}

pub type ParseResult<E> = Result<LispObj, ParserError<E>>;

#[derive(Debug, Copy, Clone)]
pub enum ParserState {
    Idle,
    List,
}

#[must_use]
pub struct Parser<I, E> 
        where I: Iterator<Item=Result<char,E>> {
    stack: Vec<(ParserState, Vec<LispObj>)>,
    stream: Lexer<I,E>,
}

impl<E: fmt::Debug> ParserError<E> {
    pub fn map_string(self) -> ParserError<String> {
        match self {
            ParserError::UnexpectedToken(a, b, c, d)
                => ParserError::UnexpectedToken(a, b, c, d),
            ParserError::UnexpectedEndOfInput(a, b, c)
                => ParserError::UnexpectedEndOfInput(a, b, c),
            ParserError::UnexpectedDelimiter(a, b, c)
                => ParserError::UnexpectedDelimiter(a, b, c),
            ParserError::LexError(err) 
                => ParserError::LexError(LexError::ReadError(format!("{:?}", err))),
        }
    }
}

impl Parser<StringIter, ()> {
    pub fn from_string<Source, Name>(s: Source, name: Name) -> Self 
                where Source: Into<String>, Name: Into<String> {
        Parser {
            stack: Vec::new(),
            stream: Lexer::from_string(s, name)
        }
    }
}

impl Parser<io::Chars<File>, io::CharsError> {
    pub fn from_file(path: String) -> Result<Self, io::Error> {
        File::open(path.clone()).map(|file| Self::new(file.chars(), path))
    }
}

impl<I: Iterator<Item=Result<char,E>>, E> Parser<I, E> {
    pub fn new(source: I, source_name: String) -> Self {
        Parser { stack: Vec::new(),
                 stream: Lexer::new(source, source_name)
        }
    }

    pub fn parse_all(self) -> Result<Vec<LispObj>, ParserError<E>> {
        let mut vec = Vec::new();

        for res in self {
            match res {
                Ok(obj) => vec.push(obj),
                Err(e) => return Err(e),
            };
        }

        Ok(vec)
    }

    pub fn source_name(&self) -> &str {
        &self.stream.source_name
    }

    fn stack_empty(&self) -> bool {
        self.stack.is_empty()
    }

    fn push_obj(&mut self, obj: LispObj) -> Option<LispObj> {
        if let Some(&mut (_, ref mut stack)) = self.stack.last_mut() {
            stack.push(obj);
            None
        } else {
            Some(obj)
        }
    }

    fn push_state(&mut self, st: ParserState) {
        self.stack.push((st, Vec::new()));
    }

    fn current_state(&self) -> Option<ParserState> {
        self.stack.last().map(|obj| obj.0)
    }

    fn pop(&mut self) -> Option<(ParserState, Vec<LispObj>)> {
        self.stack.pop()
    }
}

impl<I, E> Iterator for Parser<I, E> 
            where I: Iterator<Item=Result<char,E>> {
    type Item = ParseResult<E>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut tok;

        loop {
            tok = match self.stream.next() {
                Some(Ok(v)) => v,
                Some(Err(e)) => return Some(Err(ParserError::LexError(e))),
                None => {
                    if self.stack_empty() {
                        return None;
                    } else {
                        let err = ParserError::UnexpectedEndOfInput(self.current_state().unwrap(),
                        self.stream.line_no, self.stream.col_no);
                        return Some(Err(err));
                    }
                }
            };

            match tok.tok {
                Token::OpenParen => {
                    self.push_state(ParserState::List);
                    //return self.next();
                },

                Token::CloseParen => {
                    let list = match self.pop() {
                        Some((ParserState::List, vec)) => {
                            form_lisp_list(vec)
                        },
                        Some(_) => unimplemented!(),
                        None => {
                            let err = ParserError::UnexpectedDelimiter(Token::CloseParen,
                                                                        tok.line_no, tok.col_no);
                            return Some(Err(err));
                        },
                    };

                    match self.push_obj(list) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    };
                },

                Token::Number(n) => {
                    match self.push_obj(int!(n)) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    }
                },

                Token::Float(n) => {
                    match self.push_obj(float!(n)) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    }
                },

                Token::Ident(name) => {
                    match self.push_obj(symbol!(name)) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    }
                },

                Token::QuotedString(string) => {
                    match self.push_obj(string!(string)) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    }
                },

                Token::SpecialChar(c) => {
                    match self.push_obj(LispObj::LSpecialChar(c)) {
                        Some(obj) => return Some(Ok(obj)),
                        None => {}
                    }
                }
            };
        }
    }
}

// Make sure to parse:
// (1 . 2)
// as the result of: 
//  (cons 1 2)
// and not:
//  (list 1 2)
fn form_lisp_list(mut vec: Vec<LispObj>) -> LispObj {
    let len = vec.len();

    if len < 3 {
        LispObj::to_lisp_list(vec.into_iter().map(|o| o.to_obj_ref()))
    } else if vec[len - 2].eq(&symbol!(".")) {
        let mut end = vec.split_off(len - 3);

        let (a, b): (LispObj, LispObj);
        b = end.pop().unwrap();
        debug_assert!(end.pop().unwrap().eq(&symbol!(".")));
        a = end.pop().unwrap();

        let mut out = cons!(a, b); //LispObj::cons(a, b);

        for obj in vec.into_iter().rev() {
            out = LispObj::cons(obj, out);
        }

        out
    } else {
        LispObj::to_lisp_list(vec.into_iter().map(|o| o.to_obj_ref()))
    }
}
