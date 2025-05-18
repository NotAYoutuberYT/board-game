use chumsky::{prelude::*, text::inline_whitespace};

use crate::mini::{Action, Instructions, Operation};

fn action<'a>() -> impl Parser<'a, &'a str, Action> {
    choice((
        just(Action::PostRegister.as_ref()).to(Action::PostRegister),
        just(Action::PostFlare.as_ref()).to(Action::PostRegister),
        just(Action::Detonate.as_ref()).to(Action::Detonate),
        just(Action::Visit.as_ref()).to(Action::Visit)
    ))
}

fn operation<'a>() -> impl Parser<'a, &'a str, Operation> {
    choice((
        just(Operation::Increment.as_ref()).or(just("incr")).to(Operation::Increment),
        just(Operation::Decrement.as_ref()).or(just("decr")).to(Operation::Decrement),
        just(Operation::SetValue(0).as_ref()).or(just("set")).then(text::int(10).map(|s: &str| s.parse().unwrap()).padded()) // somehow extract the op here,
    ))
}

fn instruction_parser<'a>() -> impl Parser<'a, &'a str, Instructions> {
    end()
}