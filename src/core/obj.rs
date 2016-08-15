use std::fmt::{self, Display};
use std::rc::Rc;

pub use ::evaluator::EvalResult;
use super::{error, EnvironmentRef};
use self::LispObj::*;

pub type NativeFuncSignature        = fn(&[LispObjRef], EnvironmentRef) -> EvalResult;
pub type LispObjRef<Obj=LispObj>    = Rc<Obj>;

#[derive(Clone)]
pub struct NativeFunc(Rc<NativeFuncSignature>);

#[derive(Clone, Debug)]
pub struct ArityObj { 
    pub argnames: Vec<String>,
    pub rest: Option<String>,
}

#[derive(Clone)]
pub struct Procedure {
    pub env: EnvironmentRef,
    pub name: Option<String>,
    pub id: u32,
    pub documentation: Option<String>,
    pub body: Vec<(ArityObj, Vec<LispObjRef>)>,
}

impl fmt::Debug for Procedure {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let body_as_string: Vec<(_, Vec<String>)> = self.body.iter().map(|&(ref ar, ref body)| {
            (ar, body.iter().map(|obj| format!("{}", obj)).collect())
        }).collect();
        fmt.debug_struct("Procedure")
           .field("name", &self.name)
           .field("body", &body_as_string)
           .finish()
    }
}

/// A type which represents Lisp objects.
// TODO: Implement Clone
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
    LLazyCons(LispObjRef, Procedure),

    /// The empty list
    LNil,

    /// A Vector
    LVector(Vec<LispObjRef>),

    /// A function implemented in Rust
    /// LNativeFunc(name, documentation, func)
    LNativeFunc(String, Option<Rc<String>>, NativeFunc),

    /// A function
    LProcedure(Procedure),

    /// A caught error
    LError(error::RuntimeError),

    /*
    /// Various parser types
    LParserFileStream(Rc<RefCell<parser::Parser<io::Chars<fs::File>, io::CharsError>>>),
    LParserFromString(Rc<RefCell<parser::Parser<parser::StringIter, ()>>>),
    */
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
            (&LVector(ref me), &LVector(ref you))           => me == you,
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

impl ArityObj {
    pub fn new(names: Vec<String>, rest: Option<String>) -> Self {
        ArityObj {
            argnames: names, rest: rest
        }
    }
}

impl fmt::Display for ArityObj {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "<arity-obj:"));
        try!(write!(fmt, "("));

        for argname in self.argnames.iter() {
            try!(write!(fmt, "{} ", argname));
        }

        match &self.rest {
            &Some(ref name) => try!(write!(fmt, ". {}", name)),
            &None => {},
        }

        write!(fmt, ")>")
    }
}

impl Procedure {
    /// Creates a new procedure object
    pub fn new(env: EnvironmentRef, name: Option<String>,
               doc: Option<String>, body: Vec<(ArityObj, Vec<LispObjRef>)>) -> Procedure {
        assert!(body.len() > 0);
        let id = env.borrow_mut().next_procedure_id();
        Procedure {
            env: env, name: name,
            id: id,
            documentation: doc, body: body
        }
    }

    pub fn single_arity(env: EnvironmentRef, ar: ArityObj, body: Vec<LispObjRef>) -> Self {
        Self::new(env, None, None, vec![(ar, body)])
    }

    pub fn multiple_arity(env: EnvironmentRef, body: Vec<(ArityObj, Vec<LispObjRef>)>) -> Self {
        Self::new(env, None, None, body)
    }

    pub fn set_doc(&mut self, doc: String) {
        self.documentation = Some(doc);
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn with_doc<S: Into<String>>(mut self, doc: S) -> Self {
        self.set_doc(doc.into());
        self
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.set_name(name.into());
        self
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
                for obj in me {
                    try!(obj.fmt(fmt));
                }
                write!(fmt, "]")
            },
            &LNativeFunc(ref name,_,_)
                                => write!(fmt, "<native-procedure:{}>", name),
            &LProcedure(ref procd)
                                => {
                match procd.name {
                    Some(ref name) => {
                        write!(fmt, "<named-procedure:{}>", name)
                    },
                    None => write!(fmt, "<anonymous-function>")
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

/// To generalize over types which can be converted
/// to references to LispObj's
pub trait AsLispObjRef {
    fn to_obj_ref(self) -> LispObjRef;
}

impl AsLispObjRef for LispObjRef {
    fn to_obj_ref(self) -> LispObjRef {
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

    /// Converts a name to a symbol
    ///
    /// Also see the `symbol!(name)` macro
    pub fn make_symbol<S: Into<String>>(name: S) -> Self {
        LSymbol(name.into())
    }

    pub fn make_string<S: Into<String>>(contents: S) -> Self {
        LString(Rc::new(contents.into()))
    }

    pub fn make_native<S: Into<String>>(name: S, val: NativeFuncSignature, doc: Option<S>) -> Self {
        LNativeFunc(name.into(), doc.map(|s| Rc::new(s.into())),
                    NativeFunc(Rc::new(val)))
    }

    pub fn make_error(err: super::RuntimeError) -> Self {
        LError(err)
    }

    /// Forms a cons-cell of two objects.
    ///
    /// Also see the `cons!(car, cdr)` macro
    pub fn cons<Obj1, Obj2>(car: Obj1, cdr: Obj2) -> Self 
            where Obj1: AsLispObjRef, Obj2: AsLispObjRef {
        LCons(car.to_obj_ref(), cdr.to_obj_ref())
    }


    pub fn symbol_ref(&self) -> Option<&str> {
        match self {
            &LSymbol(ref s) => Some(&*s),
            _ => None,
        }
    }

    pub fn string_ref(&self) -> Option<&str> {
        match self {
            &LString(ref s) => Some(&*s),
            _ => None,
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

    pub fn unwrap_native(self) -> Rc<NativeFuncSignature> {
        match self {
            LNativeFunc(_,_,NativeFunc(ref f)) => f.clone(),
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
