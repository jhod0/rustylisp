use std::convert::{Into, From};
use std::fmt;
use std::io;
use std::mem;

use super::{LispObj, LispObjRef, AsLispObjRef};

pub type EvalResult<Res=LispObj> = Result<Res, RuntimeError>;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeError {
    pub errname: String,
    pub value:   Option<LispObjRef>,
    pub cause:   Option<Box<RuntimeError>>,
    pub source:  Option<LispObjRef>,
}

impl RuntimeError {
    pub fn new<S, O>(errtype: S, val: Option<O>, cause: Option<RuntimeError>, source: Option<O>) -> Self 
                where S: Into<String>,
                      O: AsLispObjRef {
        RuntimeError {
            errname: errtype.into(), value: val.map(|o| o.to_obj_ref()), 
            cause: cause.map(Box::new), source: source.map(|o| o.to_obj_ref())
        }
    }

    pub fn error<S>(errtype: S) -> Self 
                where S: Into<String> {
        Self::new::<S, LispObj>(errtype, None, None, None)
    }

    pub fn new_from(cause: RuntimeError, source: LispObjRef) -> Self {
        RuntimeError::new(cause.errname.clone(), cause.value.clone(), Some(cause), Some(source))
    }

    pub fn with_source(self, source: LispObjRef) -> Self {
        Self::new(self.errname, self.value, self.cause.map(|b| *b), Some(source))
    }

    pub fn with_cause(self, cause: RuntimeError) -> Self {
        Self::new(self.errname, self.value, Some(cause), self.source)
    }

    fn pop_cause(&mut self) -> Option<Self> {
        let mut out = None;
        mem::swap(&mut out, &mut self.cause);
        out.map(|obj| *obj)
    }

    pub fn into_traceback(self) -> Vec<Self> {
        let mut out = vec![];
        let mut val = self;

        while val.cause.is_some() {
            val = {
                let new = val.pop_cause().unwrap();
                out.push(val);
                new
            };
        }

        out.push(val);
        out
    }

    pub fn dump_traceback(self) {
        let trace = self.into_traceback();

        for err in trace {
            let val    = err.value .clone().map_or(String::new(), |val| format!("{}", val));
            println!("{}: {}", err.errname, val);
            match err.source {
                Some(ref source) => println!("\tfrom {}", source),
                None => {},
            }
        }
    }

    pub fn into_lisp_obj(self) -> LispObj {
        LispObj::LError(self)
    }
}

impl From<io::Error> for RuntimeError {
    fn from(err: io::Error) -> Self {
        RuntimeError::new("io-error", Some(string!(format!("{}", err))), None, None)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "#<ERROR-OBJ {}", self.errname));

        match &self.value {
            &Some(ref val) => { try!(write!(fmt, " value: {}", val)); },
            &None => {},
        };

        match &self.source {
            &Some(ref obj) => { try!(write!(fmt, " source: {}", obj)); },
            &None => {},
        };

        write!(fmt, ">")
    }
}
