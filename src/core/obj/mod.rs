pub mod vec;
pub use self::vec::PersistentVec;

use std::fmt::{self, Display};
use std::iter::FromIterator;
use std::rc::Rc;

pub use super::procedure::Procedure;
use super::{error, EnvironmentRef, EvalResult, RuntimeError};
use self::LispObj::*;

pub type NativeFuncSignature        = fn(&[LispObjRef], EnvironmentRef) -> EvalResult;
pub type LispObjRef<Obj=LispObj>    = Rc<Obj>;

#[derive(Clone)]
pub struct NativeFunc(Rc<NativeFuncSignature>);

pub struct ListIter {
    list: LispObjRef
}

/// A type which represents Lisp objects.
#[derive(Clone, Debug)]
pub enum LispObj {
    /// An integer
    LInteger(i64),

    /// A float
    LFloat(f64),

    /// A string
    // Rc is to prevent the overhead of copying the string's contents
    // on calls to clone
    LString(Rc<String>),

    /// Representation of a symbol
    LSymbol(String),

    /// A character
    LChar(char),

    /// A Cons cell: A fundamental lisp type
    LCons(LispObjRef, LispObjRef),

    /// A Lazy Cons cell
    // Procedure must be a thunk (0-argument function)
    LLazyCons(LispObjRef, Box<Procedure>),

    /// The empty list
    LNil,

    /// A Vector
    LVector(vec::PersistentVec<LispObjRef>),

    /// A function implemented in Rust
    /// LNativeFunc(name, documentation, func)
    LNativeFunc(String, Option<Rc<String>>, NativeFunc),

    /// A function
    LProcedure(Box<Procedure>),

    /// A caught error
    LError(Box<error::RuntimeError>),

    /*
    /// Various parser types
    LParserFileStream(Rc<RefCell<parser::Parser<io::Chars<fs::File>, io::CharsError>>>),
    LParserFromString(Rc<RefCell<parser::Parser<parser::StringIter, ()>>>),
    */
}

impl Iterator for ListIter {
    type Item = Result<LispObjRef, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let (new_list, ret) = match *self.list {
            LCons(ref car, ref cdr) => {
                (cdr.clone(), Ok(car.clone()))
            },
            LNil => return None,
            _    => (LNil.to_obj_ref(), Err(()))
        };

        self.list = new_list;
        Some(ret)
    }
}

impl PartialEq for LispObj {
    // TODO Equality for named procedures, lambdas
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&LInteger(ref me), &LInteger(ref you))         => me == you,
            (&LFloat(ref me), &LFloat(ref you))             => me.eq(you),
            (&LString(ref me), &LString(ref you))           => me == you,
            (&LSymbol(ref me), &LSymbol(ref you))           => me == you,
            (&LChar(ref me), &LChar(ref you))               => me == you,
            (&LCons(ref hme, ref tme), &LCons(ref hyou, ref tyou))                 
                                                            => hme == hyou && tme == tyou,
            (&LNil, &LNil) => true,
            (&LVector(ref me), &LVector(ref you))           => me.eq(you),
            (&LNativeFunc(ref me,_,_), &LNativeFunc(ref you,_,_)) => me == you,
            (_, _) => false,
        }
    }
}

impl fmt::Debug for NativeFunc {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "NativeFunc(_)")
    }
}

impl Eq for LispObj { }

impl Display for LispObj {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &LInteger(ref me)   => write!(fmt, "{}", me),
            &LFloat(ref me)     => write!(fmt, "{}", me),
            &LString(ref me)    => write!(fmt, "\"{}\"", me),
            &LSymbol(ref me)    => write!(fmt, "{}", me),
            &LChar(ref me)      => {
                match *me {
                    ' '  => write!(fmt, "\\space"),
                    '\t' => write!(fmt, "\\tab"),
                    '\n' => write!(fmt, "\\newline"),
                    _    => write!(fmt, "\\{}", me),
                }
            },
            // TODO implement
            &LCons(ref head, ref tail)
                                => {
                try!(write!(fmt, "("));
                // Deref from &LispObjRef to LispObj
                try!((**head).fmt(fmt));

                if !tail.is_nil() {
                    try!(write!(fmt, " "));
                }

                let mut rest = tail.clone();
                while !rest.is_nil() {
                    if let Some((hd, tl)) = rest.cons_split() {
                        try!(write!(fmt, "{}", *hd));

                        if tl.is_nil() {
                            break;
                        }

                        try!(write!(fmt, " "));
                        rest = tl;
                    } else {
                        try!(write!(fmt, ". {}", *rest));
                        break;
                    }
                }

                write!(fmt, ")")
            },
            &LLazyCons(_, _)    => write!(fmt, "#<lazy-cons>"),
            &LNil               => write!(fmt, "()"),
            &LVector(ref me)    => {
                try!(write!(fmt, "["));
                let mut iter = me.iter();
                if let Some(obj) = iter.next() {
                    try!(obj.fmt(fmt));
                }
                for obj in iter {
                    try!(write!(fmt, " "));
                    try!(obj.fmt(fmt));
                }
                write!(fmt, "]")
            },
            &LNativeFunc(ref name,_,_)
                                => write!(fmt, "#<native-procedure:{}>", name),
            &LProcedure(ref procd)
                                => {
                match procd.name {
                    Some(ref name) => {
                        write!(fmt, "#<named-procedure:{}>", name)
                    },
                    None => write!(fmt, "#<anonymous-procedure:{}>", procd.id)
                }
            },
            &LError(ref err)    => write!(fmt, "{}", err),
                                /*
            &LParserFileStream(ref stream) => write!(fmt, "<parser-stream:{}>", stream.borrow().source_name()),
            &LParserFromString(ref stream) => write!(fmt, "<parser-stream:{}>", stream.borrow().source_name()),
            */
        }
    }
}

impl<A: AsLispObjRef> FromIterator<A> for LispObj {
    fn from_iter<I>(iter: I) -> Self
        where I: IntoIterator<Item=A> {
        Self::to_lisp_list(iter.into_iter())
    }
}

/// To generalize over types which can be converted
/// to references to LispObj's
pub trait AsLispObjRef {
    fn to_obj_ref(self) -> LispObjRef;
}

impl AsLispObjRef for LispObjRef {
    fn to_obj_ref(self) -> LispObjRef {
        //println!("converting {} to a LispObjRef", self);
        self
    }
}

impl<'a> AsLispObjRef for &'a LispObjRef {
    fn to_obj_ref(self) -> LispObjRef {
        self.clone()
    }
}

impl AsLispObjRef for LispObj {
    fn to_obj_ref(self) -> LispObjRef {
        Rc::new(self)
    }
}

impl<'a> AsLispObjRef for &'a LispObj {
    fn to_obj_ref(self) -> LispObjRef {
        Rc::new(self.clone())
    }
}

impl LispObj {
    /// Returns true if self is a 'falsey' value,
    ///
    /// Falsey values include the empty list, (), a 0-length
    /// vector, the number 0, the empty string, and the symbol 'false.
    ///
    /// ```
    /// # #[macro_use]
    /// # extern crate rustylisp;
    /// # fn main() {
    /// assert!(nil!().falsey());
    /// assert!(int!(0).falsey());
    /// assert!(string!("").falsey());
    /// assert!(symbol!("false").falsey());
    /// # }
    /// ```
    pub fn falsey(&self) -> bool {
        match self {
            &LNil => true,
            &LInteger(ref n)    => *n == 0,
            &LString(ref s)     => s.is_empty(),
            &LSymbol(ref s)     => s == "false",
            &LVector(ref vec)   => vec.is_empty(),
            _ => false,
        }
    }

    /// Converts an iterator of LispObjs into a properly-formed lisp list
    pub fn to_lisp_list<O: AsLispObjRef, I: Iterator<Item=O>>(it: I) -> Self {
        let mut out = LispObj::LNil;
        let objs: Vec<_> = it.collect();
        for item in objs.into_iter().rev() {
            out = cons!(item.to_obj_ref(), out.to_obj_ref());
        }
        out
    }

    pub fn list_to_vec(&self) -> Option<Vec<LispObjRef>> {
        let (head, mut tmp) = match self {
            &LCons(ref a, ref b) => (a.to_obj_ref(), b.clone()),
            &LNil => return Some(vec![]),
            _ => return None,
        };

        let mut out = vec![head];

        loop {
            tmp = if let LCons(ref hd, ref tl) = *tmp {
                out.push(hd.clone());
                tl.clone()
            } else {
                break
            };
        }

        match *tmp {
            LNil => Some(out),
            _ => None,
        }
    }

    pub fn list_iter(&self) -> ListIter {
        ListIter { list: self.to_obj_ref() }
    }

    /// Converts a name to a symbol
    ///
    /// Also see the `symbol!(name)` macro
    pub fn make_symbol<S: Into<String>>(name: S) -> Self {
        LSymbol(name.into())
    }

    pub fn make_string<S: Into<String>>(contents: S) -> Self {
        LString(Rc::new(contents.into()))
    }

    /// Converts an iterator into a Lisp vector
    pub fn make_vector<O: AsLispObjRef, I: Iterator<Item=O>>(it: I) -> Self {
        LVector(it.map(|o| o.to_obj_ref()).collect())
    }

    pub fn collect_into_vector<Iter: Iterator<Item=EvalResult>>(it: Iter) -> EvalResult {
        struct Adapter<Iter> {
            iter: Iter,
            err: Option<RuntimeError>,
        }

        impl<T: AsLispObjRef, Iter: Iterator<Item=EvalResult<T>>> Iterator for Adapter<Iter> {
            type Item = LispObjRef;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self.iter.next() {
                    Some(Ok(obj))  => Some(obj.to_obj_ref()),
                    Some(Err(obj)) => {
                        self.err = Some(obj);
                        None
                    },
                    None => None,
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                if let (_, Some(high)) = self.iter.size_hint() {
                    (0, Some(high))
                } else {
                    panic!("LispObj::collect_into_vector : iterator must implement size_hint")
                }
            }
        }

        let mut adapter = Adapter { iter: it, err: None };
        let vec: vec::PersistentVec<_> = FromIterator::from_iter(adapter.by_ref());
        return match adapter.err {
            Some(err) => Err(err),
            None      => Ok(LispObj::LVector(vec).to_obj_ref()),
        }
    }

    pub fn make_native<S: Into<String>>(name: S, val: NativeFuncSignature, doc: Option<S>) -> Self {
        LNativeFunc(name.into(), doc.map(|s| Rc::new(s.into())),
                    NativeFunc(Rc::new(val)))
    }

    pub fn make_proc(p: Procedure) -> Self {
        LProcedure(Box::new(p))
    }

    pub fn make_error(err: super::RuntimeError) -> Self {
        LError(Box::new(err))
    }

    /// Forms a cons-cell of two objects.
    ///
    /// Also see the `cons!(car, cdr)` macro
    pub fn cons<Obj1, Obj2>(car: Obj1, cdr: Obj2) -> Self 
            where Obj1: AsLispObjRef, Obj2: AsLispObjRef {
        LCons(car.to_obj_ref(), cdr.to_obj_ref())
    }

    pub fn lazy_cons<Obj>(car: Obj, cdr: Procedure) -> Self 
            where Obj: AsLispObjRef {
        LLazyCons(car.to_obj_ref(), Box::new(cdr))
    }


    pub fn symbol_ref(&self) -> Option<&str> {
        match self {
            &LSymbol(ref s) => Some(&*s),
            _ => None,
        }
    }

    pub fn string_ref(&self) -> Option<Rc<String>> {
        match self {
            &LString(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn vec_ref(&self) -> Option<&vec::PersistentVec<LispObjRef>> {
        match self {
            &LVector(ref v) => Some(v),
            _ => None
        }
    }

    pub fn procedure_id(&self) -> Option<u32> {
        match self {
            &LProcedure(ref p) => Some(p.id),
            _ => None
        }
    }

    /*
    pub fn parser_next(&self) -> Result<Option<Result<LispObj, parser::ParserError<String>>>, ()> {
        match self {
            &LParserFileStream(ref stream) => Ok(stream.borrow_mut().next().map(|res| res.map_err(|err| err.map_string()))),
            &LParserFromString(ref stream) => Ok(stream.borrow_mut().next().map(|res| res.map_err(|err| err.map_string()))),
            _ => Err(()),
        }
    }
    */

    pub fn unwrap_symbol(self) -> String {
        match self {
            LSymbol(s) => s,
            val => panic!("unwrap_symbol performed on non-symbol: {}", val),
        }
    }

    pub fn unwrap_vec(&self) -> &vec::PersistentVec<LispObjRef> {
        match self {
            &LVector(ref v) => v,
            val => panic!("unwrap_native performed on non-native-func {}", val),
        }
    }

    pub fn unwrap_native(&self) -> Rc<NativeFuncSignature> {
        match self {
            &LNativeFunc(_,_,NativeFunc(ref f)) => f.clone(),
            val => panic!("unwrap_native performed on non-native-func {}", val),
        }
    }

    pub fn unwrap_proc(&self) -> &Procedure {
        match self {
            &LProcedure(ref procd) => procd,
            val => panic!("Cannot unwrap_proc on non-procedure {}", val)
        }
    }

    pub fn unwrap_err(&self) -> &super::RuntimeError {
        match self {
            &LError(ref err) => err,
            val => panic!("Cannot unwrap_proc on non-procedure {}", val)
        }
    }

    pub fn symbol_equal(&self, other: &str) -> bool {
        match self {
            &LSymbol(ref s) => s == other,
            _ => false,
        }
    }

    pub fn list_length(&self) -> Option<usize> {
        let mut current = self.to_obj_ref();
        let mut len = 0;

        loop {
            current = match *current {
                LCons(_, ref tl) => tl.clone(),
                LNil => return Some(len),
                _ => return None,
            };

            len += 1;
        }
    }

    pub fn list_index(&self, mut ind: u32) -> Option<LispObjRef> {
        let mut current = self.to_obj_ref();

        loop {
            current = match *current {
                LCons(ref hd, ref tl) => {
                    if ind == 0 {
                        return Some(hd.clone());
                    } else {
                        tl.clone()
                    }
                },
                _ => return None,
            };

            ind -= 1;
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            &LInteger(_) => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            &LFloat(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            &LString(_) => true,
            _ => false,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            &LSymbol(_) => true,
            _ => false,
        }
    }

    pub fn is_char(&self) -> bool {
        match self {
            &LChar(_) => true,
            _ => false,
        }
    }

    pub fn is_cons(&self) -> bool {
        match self {
            &LCons(_, _) => true,
            _ => false,
        }
    }

    pub fn is_list(&self) -> bool {
        let mut cell = self.to_obj_ref();
        loop {
            cell = match *cell {
                LCons(_, ref cdr) => cdr.clone(),
                LNil => return true,
                _    => return false,
            }
        }
    }

    pub fn is_lazy_cons(&self) -> bool {
        match self {
            &LLazyCons(_, _) => true,
            _ => false
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            &LNil => true,
            _ => false,
        }
    }

    pub fn is_vector(&self) -> bool {
        match self {
            &LVector(_) => true,
            _ => false,
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            &LNativeFunc(_,_,_) => true,
            _ => false
        }
    }

    pub fn is_proc(&self) -> bool {
        match self {
            &LProcedure(_) => true,
            _ => false
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            &LError(_) => true,
            _ => false
        }
    }

    /*
    pub fn is_parser(&self) -> bool {
        match self {
            &LParserFileStream(_) => true,
            &LParserFromString(_) => true,
            _ => false,
        }
    }
    */

    pub fn car(&self) -> Option<LispObjRef> {
        match self {
            &LCons(ref car, _) => Some(car.clone()),
            _ => None,
        }
    }

    pub fn lazy_car(&self) -> Option<LispObjRef> {
        match self {
            &LLazyCons(ref car, _) => Some(car.clone()),
            _ => None,
        }
    }

    pub fn cdr(&self) -> Option<LispObjRef> {
        match self {
            &LCons(_, ref cdr) => Some(cdr.clone()),
            _ => None,
        }
    }

    pub fn lazy_cdr(&self) -> Option<&Procedure> {
        match self {
            &LLazyCons(_, ref cdr) => Some(cdr),
            _ => None,
        }
    }

    pub fn cons_split(&self) -> Option<(LispObjRef, LispObjRef)> {
        match self {
            &LCons(ref car, ref cdr) => Some((car.clone(), cdr.clone())),
            _ => None,
        }
    }
}
