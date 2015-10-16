use std::collections::HashMap;

pub type Pred<T> = Box<Fn(&T) -> bool>;

pub struct Constraints<T> {
    pub preconditions: HashMap<&'static str, Vec<(Pred<T>, String)>>,
    pub postconditions: HashMap<&'static str, Vec<(Pred<T>, String)>>,
    pub invariants: Vec<(Pred<T>, String)>,
    pub transitions: HashMap<(&'static str, &'static str), Vec<(Pred<T>, String)>>
}

impl<T> Constraints<T> {
    pub fn new() -> Constraints<T> {
        Constraints {
            preconditions: HashMap::new(),
            postconditions: HashMap::new(),
            invariants: Vec::new(),
            transitions: HashMap::new()
        }
    }

    pub fn check_preconditions(&self, state: &'static str, ctx: &T) -> Result<(), String> {
        Constraints::check_map(&self.preconditions, state, ctx)
    }

    pub fn check_postconditions(&self, state: &'static str, ctx: &T) -> Result<(), String> {
        Constraints::check_map(&self.postconditions, state, ctx)
    }

    pub fn check_invariants(&self, ctx: &T) -> Result<(), String> {
        Constraints::check_vec(&self.invariants, ctx)
    }

    pub fn check_transitions(&self, from: &'static str, to: &'static str, ctx: &T)
        -> Result<(), String> {

        match self.transitions.get(&(from, to)) {
            // TODO: This one is interesting.
            // If there isn't a specific transition listed should we fail it?
            // Keep in mind that if there are no restraints on transition, the predicate could
            // just always return true
            None => Ok(()),
            Some(functions) => {
                Constraints::check_vec(functions, ctx)
            }
        }
    }

    fn check_map(map: &HashMap<&'static str, Vec<(Pred<T>, String)>>,
                 state: &'static str,
                 ctx: &T) -> Result<(), String> {
        match map.get(state) {
            None => Ok(()),
            Some(functions) => {
                Constraints::check_vec(functions, ctx)
            }
        }
    }

    fn check_vec(vec: &Vec<(Pred<T>, String)>, ctx: &T) -> Result<(), String> {
        for &(ref f, ref msg) in vec {
            if !f(ctx) { return Err((*msg).clone()); }
        }
        Ok(())
    }
}

/// Take a constraints object ($c), the &'static str name of the state ($s), and a predicate
/// closure ($p). Box the closure and store it into the preconditions hashmap under it's state name
/// along with an associated error message to use if the predicate fails.
#[macro_export]
macro_rules! precondition {
    ($c:ident, $s:expr, $p:expr) => {{
        let f = Box::new($p);
        let err = constraints::errstr("precondition", $s, stringify!($p));
        let mut vec = $c.preconditions.entry($s).or_insert(Vec::new());
        vec.push((f, err));
    }}
}

/// Nearly identical to precondition!
#[macro_export]
macro_rules! postcondition {
    ($c:ident, $s:expr, $p:expr) => {{
        let f = Box::new($p);
        let err = constraints::errstr("postcondition", $s, stringify!($p));
        let mut vec = $c.postconditions.entry($s).or_insert(Vec::new());
        vec.push((f, err));
    }}
}

/// Pre/Postconditions are only checked in specific states. Invariants are checked in every state.
#[macro_export]
macro_rules! invariant {
    ($c:ident, $p:expr) => {{
        let f = Box::new($p);
        let err = format!("Failed invariant: {}", stringify!($p));
        $c.invariants.push((f, err));
    }}
}

#[macro_export]
macro_rules! transition {
    ($c:ident, $f:expr, $t:expr, $p:expr) => {{
        let f = Box::new($p);
        let err = format!("Failed transition predicate from state {} to {}: {}",
                          $f, $t, stringify!($p));
        let mut vec = $c.transitions.entry(($f, $t)).or_insert(Vec::new());
        vec.push((f, err));
    }}
}

pub fn errstr(constraint: &'static str, state: &'static str, expression: &'static str) -> String{
    format!("Failed {} for state {}: {}", constraint, state, expression)
}
