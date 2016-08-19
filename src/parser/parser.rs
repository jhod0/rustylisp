pub use super::lexer::{Token, LexError};
use super::lexer::{self, Lexer, StringIter};
use ::core::obj::{LispObj, AsLispObjRef};

use std::convert::{AsRef, Into};
use std::io::{self, Read};
use std::fmt;
use std::fs::File;

#[derive(Debug)]
pub enum ParserError<E> {
    // unexpected, expecting, line no, col no
    UnexpectedToken(Token, Option<Token>, u32, u32),
    UnexpectedEndOfInput(ParserState, u32, u32),
    UnexpectedDelimiter(Token, u32, u32),
    // If no handler for special character,
    // error the character, its argument
    SpecialCharError(char, LispObj),
    NoCharHandler(char),
    LexError(LexError<E>),
}

pub type ParseResult<E> = Result<LispObj, ParserError<E>>;

#[derive(Debug, Copy, Clone)]
pub enum ParserState {
    Idle, ReaderChar(char), List, Vector
}

pub type DummyFn = fn(char, LispObj) -> Result<LispObj, Option<LispObj>>;

#[must_use]
pub struct Parser<I, E, F=DummyFn> 
        where I: Iterator<Item=Result<char,E>> {
    stack: Vec<(ParserState, Vec<LispObj>)>,
    stream: Lexer<I,E>,
    char_handler: Option<F>,
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
            ParserError::SpecialCharError(c, obj)
                => ParserError::SpecialCharError(c, obj),
            ParserError::NoCharHandler(c)
                => ParserError::NoCharHandler(c),
            ParserError::LexError(err) 
                => ParserError::LexError(LexError::ReadError(format!("{:?}", err))),
        }
    }
}

impl<I,E,F> fmt::Debug for Parser<I,E,F>
    where I: Iterator<Item=Result<char,E>> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut fmter = fmt.debug_struct("Parser");
        let mut fmtref = match self.stack.last() {
            Some(&(ref state, _)) => fmter.field("state", state),
            None => &mut fmter,
        };
        fmtref.field("lexer", &self.stream).finish()
    }
}

impl Parser<StringIter, ()> {
    pub fn from_string<Source, Name>(s: Source, name: Name) -> Self 
                where Source: Into<String>, Name: Into<String> {
        Self::new(lexer::CharIter::from_string(s), name)
    }
}

impl<I: Iterator<Item=char>> Parser<lexer::CharIter<I>, ()> {
    pub fn from_iter<S: Into<String>>(stream: I, name: S) -> Self {
        Self::new(lexer::CharIter::new(stream), name)
    }
}

impl Parser<io::Chars<File>, io::CharsError> {
    pub fn from_file<P: AsRef<::std::path::Path>>(path: P) -> Result<Self, io::Error> {
        File::open(path.as_ref())
              .map(|file| Self::new(file.chars(), format!("{:?}", path.as_ref())))
    }
}

impl<I, E, F> Parser<I, E, F>
            where I: Iterator<Item=Result<char,E>>,
                  F: Fn(char, LispObj) -> Result<LispObj, Option<LispObj>> {
    fn try_apply_reader(&self, c: char, obj: LispObj) -> Result<LispObj, ParserError<E>> {
        match &self.char_handler {
            &Some(ref handler) => handler(c, obj)
                .map_err(|err| match err {
                    Some(err_obj) => ParserError::SpecialCharError(c, err_obj),
                    None          => ParserError::NoCharHandler(c),
                }),
            &None => Err(ParserError::NoCharHandler(c)),
        }
    }

    fn push_obj(&mut self, obj: LispObj) -> Option<Result<LispObj, ParserError<E>>> {
        let c = match self.stack.last_mut() {
            Some(&mut (ParserState::List, ref mut stack)) |
            Some(&mut (ParserState::Vector, ref mut stack)) => {
                stack.push(obj);
                return None
            },
            Some(&mut (ParserState::ReaderChar(c), ref mut stack)) => {
                assert!(stack.len() == 0);
                c
            },
            Some(&mut ref top) => panic!("Parser::push_obj on state {:?}", top),
            None => return Some(Ok(obj))
        };

        match self.try_apply_reader(c, obj.clone()) {
            Ok(res)  => {
                match self.pop().expect("at least have one character") {
                    (ParserState::ReaderChar(_), _) => {},
                    _ => unreachable!()
                };
                self.push_obj(res)
            },
            Err(err) => {
                self.stack.clear();
                Some(Err(err))
            },
        }
    }

    pub fn parse_all(self) -> Result<Vec<LispObj>, ParserError<E>> {
        self.collect()
    }
}

impl<I, E> Parser<I, E>
        where I: Iterator<Item=Result<char,E>> {
    pub fn new<S: Into<String>>(source: I, source_name: S) -> Self {
        Parser { stack: Vec::new(),
                 stream: Lexer::new(source, source_name.into()),
                 char_handler: None,
        }
    }
}

impl<I, E, F> Parser<I, E, F> 
        where I: Iterator<Item=Result<char,E>> {
    pub fn with_char_handler<FNew>(self, f: FNew) -> Parser<I, E, FNew> {
        Parser { char_handler: Some(f), stream: self.stream, stack: self.stack }
    }

    pub fn source_name(&self) -> &str {
        &self.stream.source_name
    }

    fn stack_empty(&self) -> bool {
        self.stack.is_empty()
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

impl<I, E, F> Iterator for Parser<I, E, F> 
            where I: Iterator<Item=Result<char,E>>,
                  F: Fn(char, LispObj) -> Result<LispObj, Option<LispObj>> {
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
                Token::OpenParen => self.push_state(ParserState::List),

                Token::CloseParen => {
                    let list = match self.pop() {
                        Some((ParserState::List, vec)) => {
                            form_lisp_list(vec)
                        },
                        _ => {
                            let err = ParserError::UnexpectedDelimiter(Token::CloseParen,
                                                                        tok.line_no, tok.col_no);
                            return Some(Err(err));
                        },
                    };

                    match self.push_obj(list) {
                        Some(obj) => return Some(obj),
                        None => {}
                    };
                },

                Token::OpenBracket => self.push_state(ParserState::Vector),

                Token::CloseBracket => {
                    let vec = match self.pop() {
                        Some((ParserState::Vector, vec)) => {
                            LispObj::make_vector(vec.into_iter())
                        },
                        _ => {
                            let err = ParserError::UnexpectedDelimiter(Token::CloseBracket,
                                                                        tok.line_no, tok.col_no);
                            return Some(Err(err));
                        }
                    };

                    match self.push_obj(vec) {
                        Some(obj) => return Some(obj),
                        None => {},
                    };
                },

                Token::Number(n) => {
                    match self.push_obj(int!(n)) {
                        Some(obj) => return Some(obj),
                        None => {}
                    }
                },

                Token::Float(n) => {
                    match self.push_obj(float!(n)) {
                        Some(obj) => return Some(obj),
                        None => {}
                    }
                },

                Token::Ident(name) => {
                    match self.push_obj(symbol!(name)) {
                        Some(obj) => return Some(obj),
                        None => {}
                    }
                },

                Token::QuotedString(string) => {
                    match self.push_obj(string!(string)) {
                        Some(obj) => return Some(obj),
                        None => {}
                    }
                },

                Token::SpecialChar(c) => self.push_state(ParserState::ReaderChar(c)),
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
