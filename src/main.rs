#![feature(core)]
#![feature(collections)]
use std::fmt;

static DEFAULT_ATOM_SIZE: usize = 32;

enum SymbolicExpr {
    Number(f64),
    Symbol(String),
    ListExpr(Vec<SymbolicExpr>)
}


impl fmt::Display for SymbolicExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SymbolicExpr::Number(num) => write!(f, "(Number {})", num),
            SymbolicExpr::Symbol(ref sym) => write!(f, "(Symbol {})", sym),
            SymbolicExpr::ListExpr(ref sexprs) => {
                try!(f.write_str("(List"));
                for s in sexprs.iter() {
                    try!(f.write_str(" "));
                    try!(s.fmt(f));
                }
                f.write_str(")")
            }
        }
    }
}

impl fmt::Debug for SymbolicExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(PartialEq, Copy)]
enum State {
    Start,
    Symbol,
    Integer,
    IncompleteFloating,
    Floating,
}

fn to_atom(state: State, accum: String) -> Result<SymbolicExpr, &'static str> {
    match state {
        State::Symbol => {Ok(SymbolicExpr::Symbol(accum))}
        State::Integer | State::Floating => {
            match accum.parse::<f64>() {
                Some(i) => {
                    Ok(SymbolicExpr::Number(i))
                }
                None => Err("Cannot parse number")
            }
        }
        _ => Err("Invalid atom")
    }
}

// Non-recursive parse using state machine
fn read(code: &str) -> Result<Vec<SymbolicExpr>, &'static str> {
    let mut accum = String::with_capacity(DEFAULT_ATOM_SIZE);
    let mut exprs = Vec::new();
    let mut stack = Vec::new();
    let mut state = State::Start;

    for c in code.chars() {
        match c {
            // Whitespace which can only terminate atoms
            ' ' | '\n' | '\r' | '\t' => {
                if state != State::Start {
                    match to_atom(state, accum.clone()) {
                        Ok(sexpr) => {
                            exprs.push(sexpr);
                            accum.clear();
                            state = State::Start;
                        }
                        Err(s) => return Err(s)
                    }
                }
            }

            _ => {
                match (state, c) {
                    (_, '(') => {
                        if state != State::Start {
                            match to_atom(state, accum.clone()) {
                                Ok(sexpr) => {
                                    exprs.push(sexpr);
                                    accum.clear();
                                }
                                Err(s) => return Err(s)
                            }
                        }
                        state = State::Start;
                        stack.push(exprs);
                        exprs = Vec::new();
                    }

                    (_, ')') => {
                        if state != State::Start {
                            match to_atom(state, accum.clone()) {
                                Ok(sexpr) => {
                                    exprs.push(sexpr);
                                    accum.clear();
                                }
                                Err(s) => return Err(s)
                            }
                        }
                        let list = SymbolicExpr::ListExpr(exprs);
                        state = State::Start;
                        exprs = match stack.pop() {
                            Some(mut parent) => {
                                parent.push(list);
                                parent
                            }
                            None => return Err("Missing '('")
                        }
                    }

                    (State::Start, '0' ... '9') => {
                        state = State::Integer;
                        accum.push(c);
                    }

                    (State::Start, _) => {
                        state = State::Symbol;
                        accum.push(c);
                    }

                    (State::Integer, '.') => {
                        state = State::IncompleteFloating;
                        accum.push(c);
                    }

                    (State::IncompleteFloating, '0' ... '9') => {
                        state = State::Floating;
                        accum.push(c);
                    }

                    (State::Integer, '0' ... '9') | (State::Floating, '0' ... '9') => {
                        accum.push(c);
                    }

                    (State::Integer, _) | (State::Floating, _) | (State::IncompleteFloating, _) => {
                        return Err("Invalid number")
                    }

                    (State::Symbol, _) => {
                        accum.push(c);
                    }
                }
            }
        }
    }

    if state != State::Start {
        match to_atom(state, accum.clone()) {
            Ok(sexpr) => {
                exprs.push(sexpr);
            }
            Err(s) => return Err(s)
        }
    }

    if stack.len() == 0 {
        return Ok(exprs)
    } else {
        return Err("Unmatched '('")
    }
}

fn print_read(ast: Result<Vec<SymbolicExpr>, &str>) {
    match ast {
        Ok(sexprs) => {
            for s in sexprs.iter() {
                println!("{}", s);
            }
        }
        Err(s) => println!("{}", s)
    }
}

fn main() {
    let code = "12.3";
    print_read(read(code));

    let sym = "+";
    print_read(read(sym));

    let list = "()";
    print_read(read(list));

    let add = "(+ 1 2)";
    print_read(read(add));

    let magsqr = "(* (+ 1 2) (+ 3 4))";
    print_read(read(magsqr));
}
