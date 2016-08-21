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

pub fn set_current_dir<P: AsRef<path::Path>>(path: P) -> EvalResult {
    try!(std_env::set_current_dir(path));
    Ok(lisp_true!().to_obj_ref())
}

fn from_os_path_to_str(input: &path::Path) -> EvalResult<&str> {
    match input.to_str() {
        Some(dirstr) => Ok(dirstr),
        None         => internal_error!("cannot convert {:?} to string", input),
    }
}

pub fn from_os_path(input: &path::Path) -> EvalResult {
    let path = try!(from_os_path_to_str(input));
    Ok(string!(path).to_obj_ref())
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

fn pop_directory(env: EnvironmentRef) -> EvalResult<path::PathBuf> {
    let retval = env.borrow().lookup(DIRECTORY_STACK_NAME);

    if let Some(dir_stack) = retval {
        if let Some((hd, tl)) = dir_stack.cons_split() {
            let old_dir = try!(lisp_obj_to_path(hd));
            let _  = try!(set_current_dir(try!(from_os_path_to_str(&old_dir))));
            let _ = env.borrow_mut().swap_values(DIRECTORY_STACK_NAME, tl)
                       .expect("directory stack should be defined");

            Ok(old_dir)
        } else if dir_stack.is_nil() {
            environment_error!("{} is empty", DIRECTORY_STACK_NAME)
        } else {
            environment_error!("{} is not a list", DIRECTORY_STACK_NAME)
        }
    } else {
        environment_error!("{} is not defined", DIRECTORY_STACK_NAME)
    }
}

fn push_directory<P: AsRef<path::Path>>(path: P, env: EnvironmentRef) -> EvalResult {
    let dirstack = env.borrow().lookup(DIRECTORY_STACK_NAME);
    let new_path = path.as_ref();

    if let Some(stack) = dirstack {
        let new_stack = cons!(try!(get_current_dir()), stack).to_obj_ref();
        let abspath = try!(from_os_path(&try!(new_path.canonicalize())));
        let _ = try!(set_current_dir(new_path));
        let _ = env.borrow_mut().swap_values(DIRECTORY_STACK_NAME, new_stack)
                   .expect("directory stack should be defined");

        Ok(abspath)
    } else {
        environment_error!("{} is not defined", DIRECTORY_STACK_NAME)
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
    unpack_args!(args => lisp_path: Any);

    let mut file_path = try!(lisp_obj_to_path(lisp_path));
    let global = env::get_top_level(env.clone());

    let char_handlers = |c: char, obj: LispObj| {
        let handler = match env.borrow().get_char_handler(c) {
            Some(handler) => handler,
            None => return Err(None)
        };

        evaluator::apply(handler, lisp_list!(obj), env.clone())
            .map(|obj| (*obj).clone())
            .map_err(|err| Some(err.into_lisp_obj()))
    };

    if file_path.extension().is_none() {
        file_path = file_path.with_extension("lisp");
    }

    let file_parser = match Parser::from_file(&file_path) {
        Ok(file) => file,
        Err(errmsg) => io_error!("cannot open file: {:?}", errmsg),
    }.with_char_handler(char_handlers);

    if file_path.is_file() {
        let canon = try!(file_path.canonicalize());
        let _ = try!(push_directory(canon.parent().expect("all files should have parent dir"),
                                    global.clone()));
    }

    let mut out = nil!().to_obj_ref();

    for parsed_obj in file_parser {
        let obj = match parsed_obj {
            Ok(obj) => obj,
            Err(e) => io_error!("error parsing file: {:?}", e)
        };
        out = try!(evaluator::eval(obj, global.clone()))
    }

    let _ = try!(pop_directory(global));

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

    Ok(lisp_true!().to_obj_ref())
}

pub fn println(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let _ = try!(print(args, env));
    println!("");

    match io::stdout().flush() {
        Ok(_) => {}
        Err(err) => io_error!("{:?}", err)
    }

    Ok(lisp_true!().to_obj_ref())
}

pub fn lisp_pop_directory(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args);

    let path = try!(pop_directory(env));
    from_os_path(&path)
}

pub fn lisp_push_directory(args: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    unpack_args!(args => arg: Any);

    let target_dir = try!(lisp_obj_to_path(arg.clone()));
    push_directory(&target_dir, env)
}

/// TODO handle special chars
pub fn read_handler(_: &[LispObjRef], env: EnvironmentRef) -> EvalResult {
    let instream = Parser::new(io::stdin().chars(), "<stdin>");
    let top_level = env::get_top_level(env);

    let char_handler = |c, obj| {
        match top_level.borrow().get_char_handler(c) {
            Some(handler) => {
                evaluator::apply(handler, lisp_list!(obj), top_level.clone())
                           .map(|obj| (*obj).clone())
                           .map_err(|err| Some(err.into_lisp_obj()))
            },
            None => Err(None),
        }
    };

    match instream.with_char_handler(char_handler).next() {
        Some(Ok(obj))   => Ok(obj.to_obj_ref()),
        Some(Err(err))  => read_error!("{:?}", err),
        None            => runtime_error!(value symbol!("eof"); super::super::err_msgs::READ_ERROR)
    }
}
