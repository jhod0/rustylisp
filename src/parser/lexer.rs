use std::iter::Peekable;
use std::{fmt, error, convert};
use std::convert::{Into, From};
use std::str;
use std::vec;

macro_rules! opt_try {
    ( $exp:expr ) => {
        match $exp {
            Some(v) => v,
            None    => return None,
        }
    }
}

static SPECIAL_CHARS: &'static [char] = 
    &['@', '\'', '`', ',', '#', '\\'];

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Number(i64),
    Float(f64),
    Ident(String),
    QuotedString(String),
    SpecialChar(char)
}

#[derive(Debug)]
pub enum LexError<E> {
    EndOfInput,
    UnknownReadError,
    ReadError(E),
    UnexpectedEndOfInput(String),
    // For invalid string escapes
    UnknownEscape(String),
}

#[derive(Debug, Clone)]
pub struct LexedToken {
    pub tok: Token,
    pub line_no: u32, pub col_no: u32
}

pub type LexResult<T, E> = Result<T, LexError<E>>;

/// Must use, a lexer does nothing unless consumed
#[must_use]
pub struct Lexer<I: Iterator<Item=Result<char,E>>, E> {
    pub source_name: String,
    pub line_no: u32, 
    pub col_no: u32,
    special_chars: Vec<char>,
    source: Peekable<I>,
}

pub struct CharIter<I> {
    s: I
}

pub type StringIter = CharIter<vec::IntoIter<char>>;

impl<I: Iterator<Item=Result<char,E>>, E> fmt::Debug for Lexer<I,E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_struct("Lexer")
           .field("source_name", &self.source_name)
           .field("line_no", &self.line_no)
           .field("col_no", &self.col_no)
           .finish()
    }
}

impl Lexer<StringIter, ()> {
    pub fn from_string<Source, Name>(input: Source, source_name: Name) -> Self 
            where Source: Into<String>, Name: Into<String> {
        Self::new(CharIter::from_string(input),
                        source_name.into())
    }
}

impl<I: Iterator<Item=char>> Lexer<CharIter<I>, ()> {
    pub fn from_iter(it: I, name: String) -> Self {
        Self::new(CharIter::new(it), name)
    }
}

impl<I: Iterator<Item=Result<char, E>>, E> Lexer<I, E> {
    pub fn new(it: I, name: String) -> Self {
        Lexer { source_name: name,
                line_no: 0, col_no: 0,
                special_chars: Vec::from(SPECIAL_CHARS),
                source: it.peekable() }
    }

    pub fn with_special_chars<C>(self, chars: C) -> Self 
            where Vec<char>: From<C> {
        Lexer { special_chars: Vec::from(chars), ..self }
    }

    pub fn to_vec(self) -> LexResult<Vec<LexedToken>, E> {
        let mut out = Vec::new();

        for lexed in self {
            out.push(try!(lexed));
        }

        Ok(out)
    }

    fn is_special_char(&self, c: char) -> bool {
        self.special_chars.contains(&c)
    }

    fn advance(&mut self) -> LexResult<char, E> {
        match self.source.next() {
            Some(Ok(c)) => {
                if c == '\n' {
                    self.line_no += 1;
                    self.col_no = 0;
                } else {
                    self.col_no += 1;
                }
                Ok(c)
            },
            Some(Err(e)) => Err(LexError::ReadError(e)),
            None => Err(LexError::EndOfInput)
        }
    }

    fn peek(&mut self) -> LexResult<&char, E> {
        match self.source.peek() {
            Some(&Ok(ref c))  => Ok(c),
            Some(&Err(_)) => Err(LexError::UnknownReadError),
            None              => Err(LexError::EndOfInput)
        }
    }

    fn parse_word(&mut self) -> LexResult<String, E> {
        let mut s = String::new();
        loop {
            let ch = match self.peek() {
                Ok(c)  => *c,
                Err(LexError::EndOfInput) 
                       => break,
                Err(e) => return Err(e),
            };
            if is_whitespace(ch) || ch == '(' || ch == ')' ||
                ch == '[' || ch == ']' ||
                self.is_special_char(ch) {
                break
            } else {
                s.push(ch)
            };

            try!(self.advance());
        }

        Ok(s)
    }

    fn make_token(&self, tok: Token) -> LexedToken {
        LexedToken { 
            tok: tok, 
            line_no: self.line_no, 
            col_no: self.col_no
        }
    }

    fn make_token_with(&self, tok: Token, ln: u32, coln: u32) -> LexedToken {
        LexedToken { 
            tok: tok, 
            line_no: ln,
            col_no: coln,
        }
    }

    fn get_location(&self) -> (u32, u32) {
        (self.line_no, self.col_no)
    }
}

impl<I: Iterator<Item=Result<char, E>>, E> Iterator for Lexer<I, E> {
    /// Token, line no, col no
    type Item = LexResult<LexedToken, E>;

    fn next(&mut self) -> Option<Self::Item> {
        // Trim whitespace
        let mut ch = match self.advance() {
            Ok(c) => c,
            Err(LexError::EndOfInput) 
                   => return None,
            Err(e) => return Some(Err(e)),
        };
        while is_whitespace(ch) {
            ch = match self.advance() {
                Ok(c) => c,
                Err(LexError::EndOfInput) 
                       => return None,
                Err(e) => return Some(Err(e)),
            };
        }

        Some(match ch {
            '(' => Ok(self.make_token(Token::OpenParen)),
            ')' => Ok(self.make_token(Token::CloseParen)),

            '[' => Ok(self.make_token(Token::OpenBracket)),
            ']' => Ok(self.make_token(Token::CloseBracket)),

            // Comment, read till newline
            ';' => {
                let mut next = self.advance();

                while let Ok(c) = next {
                    if c == '\n' {
                        return self.next();
                    }
                    next = self.advance();
                }

                match next {
                    Ok(_) => unreachable!(),
                    Err(LexError::EndOfInput) => return None,
                    Err(e) => return Some(Err(e)),
                }
            }

            // String handler
            // TODO check for escaped strings (i.e. "quote: \" still string")
            '"' => {
                let (line, col) = self.get_location();

                let mut next = self.advance();
                let mut s = String::new();
                while let Ok(c) = next {
                    if c == '"' {
                        break;
                    } else if c == '\\' {
                        if let Ok(cn) = self.advance() {
                            match cn {
                                '\\' => s.push('\\'),
                                't'  => s.push('\t'),
                                'n'  => s.push('\n'),
                                '"'  => s.push('"'),
                                uc   => {
                                    let errmsg = format!("Unknown string escape: \\{}", uc);
                                    return Some(Err(LexError::UnknownEscape(errmsg)))
                                },
                            }
                        } else {
                            break;
                        };
                    } else {
                        s.push(c)
                    }

                    next = self.advance();
                }

                match next {
                    Ok('"')  => {},
                    Ok(_)    => unreachable!("should only break on closing quote"),
                    Err(LexError::EndOfInput) => {
                        let errmsg = String::from("EOF reached before string terminator");
                        return Some(Err(LexError::UnexpectedEndOfInput(errmsg)))
                    },
                    Err(e) => return Some(Err(e)),
                };

                Ok(self.make_token_with(Token::QuotedString(s), line, col))
            },

            // Number handler
            '0'...'9' | '+' | '-' => {
                let (line, col) = self.get_location();

                let mut s = match self.parse_word() {
                    Ok(s)  => s,
                    Err(LexError::EndOfInput) 
                           => String::with_capacity(1),
                    Err(e) => return Some(Err(e)),
                };

                if ('0'...'9').contains(ch) {
                    s.insert(0, ch);
                }

                // Try and parse an integer
                if let Ok(mut num) = s.parse::<i64>() {
                    if ch == '-' {
                        num = -num;
                    }
                    Ok(self.make_token_with(Token::Number(num), line, col))
                }

                // Try and parse a floating-point
                else if let Ok(mut num) = s.parse::<f64>() {
                    if ch == '-' {
                        num = -num;
                    }
                    Ok(self.make_token_with(Token::Float(num), line, col))
                }

                // Could not parse, this is an identifier
                else {
                    if ch == '+' || ch == '-' {
                        s.insert(0, ch);
                    }
                    Ok(self.make_token_with(Token::Ident(s), line, col))
                }
            },

            _ => {
                let (line, col) = self.get_location();

                if self.is_special_char(ch) {
                    Ok(self.make_token_with(Token::SpecialChar(ch), line, col))
                } else {
                    let mut s = String::with_capacity(1);
                    s.push(ch);
                    let ident = self.parse_word()
                                    .map(|mut wd| {
                                    wd.insert(0, ch);
                                    wd
                                }).unwrap_or(s);
                    Ok(self.make_token_with(Token::Ident(ident), line, col))
                }
            }
        })
    }
}


/******************** LexError Implementations ***************/

impl<E: error::Error> fmt::Display for LexError<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}

impl<E: error::Error> error::Error for LexError<E> {
    fn description(&self) -> &str {
        match self {
            &LexError::EndOfInput       => "End of input reached",
            &LexError::UnknownReadError => "Unknown read error",
            &LexError::ReadError(ref e) => e.description(),
            &LexError::UnexpectedEndOfInput(ref s) => &s,
            &LexError::UnknownEscape(ref s) => &s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &LexError::EndOfInput       => None,
            &LexError::UnknownReadError => None,
            &LexError::ReadError(ref e) => Some(e),
            &LexError::UnexpectedEndOfInput(_) => None,
            &LexError::UnknownEscape(_) => None,
        }
    }
}

impl<E> convert::From<E> for LexError<E> {
    fn from(e: E) -> Self {
        LexError::ReadError(e)
    }
}

/******************* StringIter for use above *******************/

impl CharIter<vec::IntoIter<char>> {
    pub fn from_string<S: Into<String>>(s: S) -> Self {
        CharIter {
            s: s.into().chars().collect::<Vec<_>>().into_iter()
        }
    }
}

impl<I> CharIter<I> {
    pub fn new(s: I) -> Self {
        CharIter { s: s }
    }
}

impl<I: Iterator<Item=char>> Iterator for CharIter<I> {
    type Item = Result<char, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.s.next() {
            Some(c) => Some(Ok(c)),
            None => None
        }
    }
}

/****************** Helper functions *******************/

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}
