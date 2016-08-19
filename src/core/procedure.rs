use std::fmt;
pub use super::{LispObjRef, EnvironmentRef};

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

impl fmt::Display for ArityObj {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(fmt, "#<arity-obj:"));
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

impl ArityObj {
    pub fn new(names: Vec<String>, rest: Option<String>) -> Self {
        ArityObj {
            argnames: names, rest: rest
        }
    }
}

impl fmt::Debug for Procedure {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let body_as_string: Vec<(_, Vec<String>)> = self.body.iter().map(|&(ref ar, ref body)| {
            (ar, body.iter().map(|obj| format!("{}", obj)).collect())
        }).collect();
        fmt.debug_struct("Procedure")
           .field("name", &self.name)
           .field("documentation", &self.documentation)
           .field("body", &body_as_string)
           .finish()
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
