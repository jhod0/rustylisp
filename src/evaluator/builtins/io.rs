use std::convert::AsRef;
use std::env as std_env;
use std::io::{self, Read, Write};
use std::path;

use ::core::{env, LispObj, LispObjRef, AsLispObjRef, EnvironmentRef};
use ::parser::Parser;
use ::evaluator::{self, EvalResult};


pub const DIRECTORY_STACK_NAME: &'static str = "*directory-stack*";

pub fn get_current_dir() -> EvalResult {
    let dir =  try!(std_env::current_dir());
    from_os_path(dir.as_path())
}

pub fn set_current_dir(input: &str) -> EvalResult {
    let path = try!(into_os_path(input));
    try!(std_env::set_current_dir(path));
    Ok(lisp_true!())
}

pub fn into_os_path(input: &str) -> EvalResult<path::PathBuf> {
    Ok(path::PathBuf::from(input))
}

fn from_os_path_to_str(input: &path::Path) -> EvalResult<&str> {
    match input.to_str() {
        Some(dirstr) => Ok(dirstr),
        None         => internal_error!("cannot convert {:?} to string", input),
    }
}

pub fn from_os_path(input: &path::Path) -> EvalResult {
    let path = try!(from_os_path_to_str(input));
    Ok(string!(path))
}

pub fn lisp_obj_to_path(obj: LispObjRef) -> EvalResult<path::PathBuf> {
    if let Some((hd, tl)) = obj.cons_split() {
        let mut path = try!(lisp_obj_to_path(hd));
        if tl.is_nil() {
            Ok(path)
        } else {
            let rest = try!(lisp_obj_to_path(tl));
            path.push(rest);
            Ok(path)
        }
    } else if let Some(name) = obj.symbol_ref() {
        Ok(path::PathBuf::from(String::from(name)))
    } else if let Some(string) = obj.string_ref() {
        Ok(path::PathBuf::from((*string).clone()))
    } else {
        type_error!("{} is not a path", obj)
    }
}

/***************** Lisp Functions **************/

pub fn lisp_get_current_dir(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    if args.len() != 0 {
        arity_error!("current-dir: expected no arguments, got {}", 
                     LispObj::to_lisp_list(args.iter()))
    }

    get_current_dir()
}

pub fn lisp_set_current_dir(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    unpack_args!(args => new_dir: LString);

    set_current_dir(&*new_dir)
}

pub fn load_file_handler(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => file_path: LString);

    let global = env::get_top_level(env.clone());

    let char_handlers = |c: char, obj: LispObj| {
        let handler = match env.borrow().get_char_handler(c) {
            Some(handler) => handler,
            None => return Err(None)
        };

        evaluator::apply(handler, lisp_list!(obj), env.clone())
            .map_err(|err| Some(err.into_lisp_obj()))
    };

    let file_parser = match Parser::from_file((*file_path).clone()) {
        Ok(file) => file,
        Err(errmsg) => io_error!("cannot open file: {:?}", errmsg),
    }.with_char_handler(char_handlers);

    let mut out = nil!();
    for parsed_obj in file_parser {
        let obj = match parsed_obj {
            Ok(obj) => obj,
            Err(e) => io_error!("error parsing file: {:?}", e)
        };
        out = try!(evaluator::eval(obj, global.clone()))
    }

    Ok(out)
}

pub fn print(args: &[LispObjRef], _: EnvironmentRef) -> EvalResult {
    for arg in args.iter() {
        match arg.as_ref() {
            &LispObj::LString(ref s) => print!("{}", s),
            other => print!("{}", other),
        }
    }

    match io::stdout().flush() {
        Ok(_) => {}
        Err(err) => io_error!("{:?}", err)
    }

    Ok(lisp_true!())
}

pub fn println(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let _ = try!(print(args, env));
    println!("");

    match io::stdout().flush() {
        Ok(_) => {}
        Err(err) => io_error!("{:?}", err)
    }

    Ok(lisp_true!())
}

pub fn pop_directory(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args);

    let retval = env.borrow().lookup(DIRECTORY_STACK_NAME);

    if let Some(dir_stack) = retval {
        if let Some((hd, tl)) = dir_stack.cons_split() {
            let old_dir = try!(lisp_obj_to_path(hd));
            let _  = try!(set_current_dir(try!(from_os_path_to_str(&old_dir))));
            let _ = env.borrow_mut().swap_values(DIRECTORY_STACK_NAME, tl)
                       .expect("directory stack should be defined");

            from_os_path(&old_dir)
        } else if dir_stack.is_nil() {
            environment_error!("{} is empty", DIRECTORY_STACK_NAME)
        } else {
            environment_error!("{} is not a list", DIRECTORY_STACK_NAME)
        }
    } else {
        environment_error!("{} is not defined", DIRECTORY_STACK_NAME)
    }
}

pub fn push_directory(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);

    let target_dir = try!(lisp_obj_to_path(arg.clone()));
    let retval = env.borrow().lookup(DIRECTORY_STACK_NAME);

    if let Some(dir_stack) = retval {
        let abspath = try!(from_os_path(&try!(target_dir.canonicalize())));
        let _ = try!(set_current_dir(try!(from_os_path_to_str(&target_dir))));
        let new_stack = cons!(abspath.clone(), dir_stack).to_obj_ref();
        let _ = env.borrow_mut().swap_values(DIRECTORY_STACK_NAME, new_stack)
                   .expect("directory stack should be defined");

        Ok(abspath)
    } else {
        environment_error!("{} is not defined", DIRECTORY_STACK_NAME)
    }
}

/// TODO handle special chars
pub fn read_handler(_: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let instream = Parser::new(io::stdin().chars(), "<stdin>");
    let top_level = env::get_top_level(env);

    let char_handler = |c, obj| {
        match top_level.borrow().get_char_handler(c) {
            Some(handler) => {
                evaluator::apply(handler, lisp_list!(obj), top_level.clone())
                           .map_err(|err| Some(err.into_lisp_obj()))
            },
            None => Err(None),
        }
    };

    match instream.with_char_handler(char_handler).next() {
        Some(Ok(obj))   => Ok(obj),
        Some(Err(err))  => read_error!("{:?}", err),
        None            => read_error!("end of input"),
    }
}

//pub fn 
