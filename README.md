[![Build
Status](https://travis-ci.org/andrewjstone/funfsm.svg?branch=master)](https://travis-ci.org/andrewjstone/funfsm)

[API Documentation](https://docs.rs/funfsm)

## Description
`funfsm` is a finite state machine library where every state in the FSM is a function. Each FSM also
has a corresponding `context` that maintains the data state for the FSM. On each receipt of a
message, the function representing the current state is called with the message and a mutable
reference to the context. The state function mutates the context as necessary and returns the next
state function as well as any outgoing messages destined for other FSMs.

## Usage
Add the following to your `Cargo.toml`

```toml
[dependencies]
funfsm = "0.1"
```

Add this to your crate root

```Rust
extern crate funfsm;
```

The following subsections all use code from the [bowl_fsm test
code](https://github.com/andrewjstone/funfsm/blob/master/tests/bowl_fsm.rs) as an example.

#### Creating an FSM
The first step in using funfsm is to determine the type of the context, input msg and output
messages used by your fsms. These types are declared by implementing the
[`FsmTypes`](https://github.com/andrewjstone/funfsm/blob/master/src/fsm.rs#L23) trait.

Using the example of a state machine for feeding a cat, where the states are the states of the cat
food bowl, we'd define our types like this:

```Rust
use funfsm::FsmTypes;

const MAX_RESERVES: u8 = 10;
const REFILL_THRESHOLD: u8 = 9;

#[derive(Debug, Clone)]
pub struct Context {
    pub contents: u8, // % of the bowl that is full
    pub reserves: u8, // The amount of bowls of food left in the bag
}

impl Context {
    pub fn new() -> Context {
        Context {
            contents: 0, // The bowl starts off empty
            reserves: MAX_RESERVES,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CatMsg {
    Meow,
    Eat(u8) // % of food to eat
}

#[derive(Debug, Clone)]
pub enum StoreReq {
    Buy(u8)
}

#[derive(Debug, Clone)]
pub enum StoreRpy {
    Bowls(u8)
}

#[derive(Debug, Clone)]
pub enum BowlMsg {
    CatMsg(CatMsg),
    StoreRpy(StoreRpy)
}

#[derive(Debug)]
pub struct BowlTypes;

impl FsmTypes for BowlTypes {
    type Context = Context;
    type Msg = BowlMsg;
    type Output = StoreReq;
}
```

We can see that `Context` maintains information about the cat food bowl and supply that will help
determine when to transition between states. We see that a `BowlMsg` is used to instruct the state
machine to make a transition, and can consist of an action from the cat, or a reply from the pet
store. Finally, we may be running low on food at some point, and thus need to re-order from the pet
store. Therfore any output messages caused by state transitions are `StoreReq` messages.

Now that we have our types declared, we need to create the actual state functions that represent our
state machine. In our case there are only 2 states that the bowl can be in: `empty` or `full`. Note
that `full` in this case means any bowl that has some food in it. Once again, note that each state
takes a mutable `Context` and a `BowlMsg` as paramters and returns a tuple containing the next state
and a Vec containing any outgoing messages destined for other FSMs, etc... In order to remove some
verbosity from the return type signature, the type `StateFn` is used to represent the next state.
It's just a wrapper around a two-tuple containing a `&'static str` representing the name of the
state function and a function pointer to the actual state function.

```Rust
use funfsm::{Fsm, StateFn};

pub fn empty(ctx: &mut Context, msg: BowlMsg) -> (StateFn<BowlTypes>, Vec<StoreReq>) {
    if let BowlMsg::CatMsg(CatMsg::Meow) = msg {
        if ctx.reserves > 0 {
            // Fill the bowl
            ctx.contents = 100;
            ctx.reserves -= 1;
            if ctx.reserves <= REFILL_THRESHOLD {
                let output = vec![StoreReq::Buy(10)];
                return next!(full, output);
            }
            return next!(full);
        } else {
            return next!(empty);
        }
    }

    if let BowlMsg::StoreRpy(StoreRpy::Bowls(num)) = msg {
        ctx.reserves += num-1;
        ctx.contents = 100;
        return next!(full);
    }

    next!(empty)
}

pub fn full(ctx: &mut Context, msg: BowlMsg) -> (StateFn<BowlTypes>, Vec<StoreReq>) {
    if let BowlMsg::CatMsg(CatMsg::Eat(pct)) = msg {
        if pct >= ctx.contents {
            ctx.contents = 0;
            return next!(empty)
        } else {
            ctx.contents -= pct;
            return next!(full)
        }
    }

    if let BowlMsg::StoreRpy(StoreRpy::Bowls(num)) = msg {
        ctx.reserves += num;
    }
    next!(full)
}
```

The code above illustrates our state functions. We can see that depending upon the current state,
the message received, and the context, the next state and any outgoing messages are determined. The
`next!` macro is what drives the transition between states so that the user doesn't manually have to
create `StateFn` values. There are two forms of this macro: one taking one parameter for transitioning when there aren't
any outgoing messages, and one with two parameters for when there are outgoing messages.

#### Using an fsm
Now that we have an fsm coded up, what can we do with it? Well, first we have to instantiate an
instance:

```Rust
let mut fsm = Fsm::<BowlTypes>::new(Context::new(), state_fn!(empty));
```

The code above shows how we create a new `Fsm` parameterized by the appropriate type. The constructor
takes a new `Context` and the initial state of the FSM as given to the ```state_fn!``` macro.

Now that our fsm is active we can send it messages and check its state. Note that in this example we
don't actually have a pet store created to send messages, so we just ignore any messages returned by the
FSM for sending.

```Rust

// Let's ensure we start off in the empty state. The extra block used here is because `get_state`
// calls return a reference to the internal context of the fsm and we want to limit it's scope so
// we can mutate the fsm with further send calls.
{
  let (name, ctx) = fsm.get_state();
  assert_eq!(name, "empty");
  assert_eq!(ctx.contents, 0);
}

// Send a message
let _ = fsm.send(BowlMsg::CatMsg(CatMsg::Meow));

// Our bowl was empty, but we have some reserves at home. When the cat meows, the bowl gets
// refilled. Let's ensure this actually happens.

let (name, ctx) = fsm.get_state();
assert_eq!(name, "full");
assert_eq!(ctx.contents, 100);
```

That's it for operating your FSM! Simple, right?

#### Invariants and testing
You have an FSM. It looks pretty good, but how do you know it's correct? Obviously you need to
test it! In this case, the recommended testing should generate messages, run them through the state
machine, and ensure that the FSM maintains certain invariants throughout the transitions between
states. `funfsm` provides a `Constraints` type to make this sort of testing easier. There are 3
types of constraints that get checked on each message processed by the FSM. `Preconditions` get
checked in a specific state before the message is sent to the fsm, `invariants` get checked in all
states, and `transitions` get checked after a message is sent to the fsm and it transitions from one
state to another. Each one of these constraints is added via macro to make adding new invariants
straightforward, and to allow test failures to report the exact constraint that fails.

```Rust
use funfsm::constraints::Constraints;

// Create a new constraints object
let mut c = Constraints::new();

// Add some preconditions. Each precondition only runs when the state machine is in that state.
// These particular preconditions simply check that when in the given state, the contents of the
// bowl is correct. Note that each state can have multiple preconditions, although only one for each
// state is shown here. In fact, complex state machines should have multiple preconditions rather
// than one large predicate function. This makes it easier to add new ones and enables better error
// reporting.
precondition!(c, "empty", |ctx: &Context| ctx.contents == 0);
precondition!(c, "full", |ctx: &Context| ctx.contents > 0 && ctx.contents <= 100);

// Add an invariant that gets checked in every state after the message is sent to the fsm and the
// transition occurs. Like preconditions, each state can have multiple invariants, and the same
// recommendations to writing them apply.
invariant!(c, |ctx: &Context| ctx.contents <= 100);

// Add some transition constraints. Transition constraints are only checked when the fsm transitions
//from one given state to another given state. Note that because transitions take so many input
//parameters (making it overly verbose to use closures), they are written differently from both
//preconditions and invariants. A single function is associated with a state transition. Inside that
//transition function, individual assertions/invariants are verified using the "check!" macro. This
//allows the same benefits of small predicates as used in both preconditions and invariants, namely
//ease of adding new checks and error reporting. The `&'static str s` passed to "check!" is there to
// indicate which transition a check failed in.
transition!(c, "empty" => "full", empty_to_full);
transition!(c, "full" => "empty", full_to_empty);

// The actual transition constraint functions. Note that these functions only illustrate some
// possibilities of error checks and don't check all invariants.

fn empty_to_full(init_ctx: &Context,
                 final_ctx: &Context,
                 msg: &BowlMsg,
                 _output: &Vec<StoreReq>) -> Result<(), String>
{
   let s = "Transition from empty to full";
   check!(s, init_ctx.contents == 0);
   check!(s, final_ctx.contents == 100);
   check!(s, match *msg {
       BowlMsg::StoreRpy(_) => true,
       BowlMsg::CatMsg(CatMsg::Meow) => true,
       _ => false
   });
   Ok(())
}

fn full_to_empty(init_ctx: &Context,
                 final_ctx: &Context,
                 msg: &BowlMsg,
                 _output: &Vec<StoreReq>) -> Result<(), String>
{
    let s = "Transition from full to empty";
    check!(s, init_ctx.contents > 0);
    check!(s, final_ctx.contents == 0);
    check!(s, { if let BowlMsg::CatMsg(CatMsg::Eat(_)) = *msg {
        true
    } else {
        false
    }});
   Ok(())
}
```
Now that we have our test constraints defined, how do we validate our FSM using these constraints? We
use a `Checker` object. The `Checker` checks preconditions, sends a message to the fsm and then
checks its invariants and transition constraints. It returns any outgoing messages in an `Ok()` on
success or an error string with a stringified version of the failing check, along with line number
and file on failure.

The recommended way to generate messages is using
[`Quickcheck`](https://github.com/BurntSushi/quickcheck), but for now we'll just use a plain ol'
static list of messages.

```Rust
use funfsm::fsm_check::Checker;

// The checker must be created with a new context and initial state of the FSM. The "c" in the third
// paramater is the `Constraints` object defined above.
let mut checker = Checker::<BowlTypes>::new(Context::new(), state_fn!(empty), c);
for msg in msgs {
  assert_matches!(checker.check(msg), Ok(_));
}
```

That's it. You now have everything you need to create and test Fun FSMs!
