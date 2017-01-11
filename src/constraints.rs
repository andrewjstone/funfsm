use std::collections::HashMap;
use fsm::FsmTypes;

pub type Pred<T: FsmTypes> = Box<Fn(&T::Context) -> bool>;
pub type TransitionCheck<T: FsmTypes>
    = fn(&T::Context, &T::Context, &T::Msg, &Vec<T::Output>) -> Result<(), String>;

pub struct Constraints<T: FsmTypes> {
    pub preconditions: HashMap<&'static str, Vec<(Pred<T>, String)>>,
    pub invariants: Vec<(Pred<T>, String)>,
    pub transitions: HashMap<(&'static str, &'static str), TransitionCheck<T>>
}

impl<T: FsmTypes> Constraints<T> {
    pub fn new() -> Constraints<T> {
        Constraints {
            preconditions: HashMap::new(),
            invariants: Vec::new(),
            transitions: HashMap::new()
        }
    }

    pub fn check_preconditions(&self, state: &'static str, ctx: &T::Context) -> Result<(), String> {
        Constraints::<T>::check_map(&self.preconditions, state, ctx)
    }

    pub fn check_invariants(&self, ctx: &T::Context) -> Result<(), String> {
        Constraints::<T>::check_vec(&self.invariants, ctx)
    }

    /// Verify a transition result
    ///
    ///  `from` is the from state,
    ///  `to` is the to state,
    ///  `init_ctx` is the internal data state of the fsm **before** the transition
    ///  `final_ctx` is the internal data state of the fsm **after** the transition
    ///  `msg` is the message that caused the transition
    ///  `output` is the output messages as a result of the transition
    ///
    ///  Returns an error string if the transition check fails
    pub fn check_transition(&self,
                            from: &'static str,
                            to: &'static str,
                            init_ctx: &T::Context,
                            final_ctx: &T::Context,
                            msg: &T::Msg,
                            output: &Vec<T::Output>) -> Result<(), String>
    {
        match self.transitions.get(&(from, to)) {
            None => Ok(()),
            Some(check) => {
                check(init_ctx, final_ctx, msg, output)
            }
        }
    }

    fn check_map(map: &HashMap<&'static str, Vec<(Pred<T>, String)>>,
                 state: &'static str,
                 ctx: &T::Context) -> Result<(), String> {
        match map.get(state) {
            None => Ok(()),
            Some(functions) => {
                Constraints::<T>::check_vec(functions, ctx)
            }
        }
    }

    fn check_vec(vec: &Vec<(Pred<T>, String)>, ctx: &T::Context) -> Result<(), String> {
        for &(ref f, ref msg) in vec {
            if !f(ctx) { return Err((*msg).clone()); }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! check {
    ($err:expr, $predicate:expr) => {
        if $predicate {
            let res: Result<(), String> = Ok(());
            res
        } else {
            return Err(format!("Error: {} Predicate: {} File: {} Line: {}",
                               $err, stringify!($predicate), file!(), line!()));
        }
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
    ($constraints:ident, $from:expr => $to:expr, $check:expr) => {{
        $constraints.transitions.insert(($from, $to), $check);
    }}
}

pub fn errstr(constraint: &'static str, state: &'static str, expression: &'static str) -> String{
    format!("Failed {} for state {}: {}", constraint, state, expression)
}
