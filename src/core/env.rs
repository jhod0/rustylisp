use std::cell::RefCell; 
use std::collections::HashMap;
use std::rc::Rc;

use super::LispObjRef;

pub fn get_top_level(env: EnvironmentRef) -> EnvironmentRef {
    match env.borrow().parent {
        Some(ref par) => return get_top_level(par.clone()),
        None => {}
    };
    env
}

pub type EnvironmentRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    parent: Option<EnvironmentRef>,
    bindings: HashMap<String, LispObjRef>,
    max_procedure_id: u32,
    // These are Options so that they are not allocated unless
    // they are really needed
    macros: Option<HashMap<String, LispObjRef>>,
    special_chars: Option<HashMap<char, LispObjRef>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent:             None,
            bindings:           HashMap::new(),
            max_procedure_id:   0,
            macros:             None,
            special_chars:      None,
        }
    }

    pub fn new_with_bindings<It>(bindings: It) -> Self 
            where It: Iterator<Item=(String, LispObjRef)> {
        Self::new().with_bindings(bindings)
    }

    pub fn with_bindings<It>(self, bindings: It) -> Self 
            where It: Iterator<Item=(String, LispObjRef)> {
        let mut bindmap = HashMap::new();
        for (name, val) in bindings {
            let _ = bindmap.insert(name, val);
        }
        Environment { bindings: bindmap, ..self }
    }

    pub fn with_macros<It>(self, bindings: It) -> Self
            where It: Iterator<Item=(String, LispObjRef)> {
        let mut macros = HashMap::new();
        for (name, mac) in bindings {
            let _ = macros.insert(name, mac);
        }
        Environment { macros: Some(macros), ..self }
    }

    pub fn with_special_chars<It>(self, bindings: It) -> Self
            where It: Iterator<Item=(char, LispObjRef)> {
        let mut chars = HashMap::new();
        for (name, handler) in bindings {
            let _ = chars.insert(name, handler);
        }
        Environment { special_chars: Some(chars), ..self }
    }

    pub fn from_parent(parent: EnvironmentRef) -> Self {
        let mut out = Self::new();
        out.parent = Some(parent.clone());
        out
    }

    pub fn to_env_ref(self) -> EnvironmentRef {
        Rc::new(RefCell::new(self))
    }

    pub fn is_top_level(&self) -> bool {
        self.parent.is_none()
    }

    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
        self.macros = None;
        self.special_chars = None;
    }

    pub fn next_procedure_id(&mut self) -> u32 {
        match self.parent {
            Some(ref par) => {
                par.borrow_mut().next_procedure_id()
            },
            None => {
                let cur = self.max_procedure_id;
                self.max_procedure_id += 1;
                cur
            },
        }
    }

    // TODO Only sets char handler in this environment - should it be set in parent
    // environment?
    pub fn set_char_handler(&mut self, name: char, value: LispObjRef) -> Option<LispObjRef> {
        let map = match self.special_chars {
            Some(ref mut chars) => return chars.insert(name, value),
            None => {
                let mut chars  = HashMap::new();
                let inserted   = chars.insert(name, value);
                debug_assert!(inserted.is_none());
                chars
            },
        };
        self.special_chars = Some(map);
        None
    }

    pub fn get_char_handler(&self, name: char) -> Option<LispObjRef> {
        let lookup = match &self.special_chars {
            &Some(ref m) => m.get(&name),
            &None => None,
        };

        if lookup.is_some() {
            lookup.map(|o| o.clone())
        } else {
            match &self.parent {
                &Some(ref par) => par.borrow().get_char_handler(name),
                &None => None,
            }
        }
    }

    // TODO Only sets macro in this environment - should it be set in parent
    // environment?
    pub fn let_macro(&mut self, name: String, value: LispObjRef) -> Option<LispObjRef> {
        let map = match self.macros {
            Some(ref mut macros) => return macros.insert(name, value),
            None => {
                let mut macros = HashMap::new();
                let inserted   = macros.insert(name, value);
                debug_assert!(inserted.is_none());
                macros
            },
        };
        self.macros = Some(map);
        None
    }

    pub fn lookup_macro(&self, name: &str) -> Option<LispObjRef> {
        let lookup = match &self.macros {
            &Some(ref m) => m.get(name),
            &None => None,
        };

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
