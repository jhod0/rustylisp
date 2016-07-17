use std::cell::RefCell; 
use std::collections::HashMap;
use std::rc::Rc;

use super::LispObjRef;

pub type EnvironmentRef = Rc<RefCell<Environment>>;

pub fn get_top_level(env: EnvironmentRef) -> EnvironmentRef {
    match env.borrow().parent {
        Some(ref par) => return get_top_level(par.clone()),
        None => {}
    };
    env
}

#[derive(Debug)]
pub struct Environment {
    parent: Option<EnvironmentRef>,
    bindings: HashMap<String, LispObjRef>,
    // TODO figure out good way to store macros
    macros: HashMap<String, LispObjRef>,
}

impl Environment {
    pub fn empty() -> Self {
        Environment {
            parent: None,
            bindings: HashMap::new(),
            macros: HashMap::new(),
        }
    }
    
    pub fn new_with_bindings<It>(bindings: It) -> Self 
            where It: Iterator<Item=(String, LispObjRef)> {
        let mut out = Self::empty();

        for (name, val) in bindings {
            let res = out.bindings.insert(name, val);
            debug_assert!(res.is_none());
        }

        out
    }

    pub fn with_parent(parent: EnvironmentRef) -> Self {
        let mut out = Self::empty();
        out.parent = Some(parent.clone());
        out
    }

    pub fn to_env_ref(self) -> EnvironmentRef {
        Rc::new(RefCell::new(self))
    }

    pub fn is_top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn let_macro(&mut self, name: String, value: LispObjRef) -> Option<LispObjRef> {
        self.macros.insert(name, value)
    }

    pub fn lookup_macro(&self, name: &str) -> Option<LispObjRef> {
        let lookup = self.macros.get(name);

        if lookup.is_some() {
            lookup.map(|o| o.clone())
        } else {
            match &self.parent {
                &Some(ref par) => par.borrow().lookup_macro(name),
                &None => None,
            }
        }
    }

    // Returns the previous value, if there was one.
    // If name was not previously registered, no change occurs.
    pub fn swap_values(&mut self, name: &str, new_val: LispObjRef) -> Option<LispObjRef> {
        if let Some(old) = self.bindings.remove(name) {
            let is_none = self.bindings.insert(String::from(name), new_val).is_none();
            assert!(is_none);
            Some(old)
        } else {
            match &mut self.parent {
                &mut Some(ref mut par) => par.borrow_mut().swap_values(name, new_val),
                &mut None => None,
            }
        }
    }

    pub fn let_new(&mut self, name: String, value: LispObjRef) -> Option<LispObjRef> {
        self.bindings.insert(name, value)
    }

    pub fn lookup(&self, name: &str) -> Option<LispObjRef> {
        let lookup = self.bindings.get(name);

        if lookup.is_some() {
            lookup.map(|o| o.clone())
        } else {
            // We don't have a binding for this name,
            // check if parent frame does
            match &self.parent {
                &Some(ref par) => par.borrow().lookup(name),
                &None => None,
            }
        }
    }
}


// for debugging purposes
/*
impl Drop for Environment {
    fn drop(&mut self) {
        println!("dropping environment with bindings: {:?}", self.bindings);
    }
}
*/
