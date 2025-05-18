use std::{fs::File, io::Read, path::PathBuf};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    extra::Err,
    prelude::*,
    text::{inline_whitespace, whitespace},
};
use thiserror::Error;

use crate::mini::{Action, Condition, Instruction, Instructions, Operation};

/// a function which returns an instruction parser. should
/// be used as instructions().parse()
fn instructions<'a>() -> impl Parser<'a, &'a str, Instructions, Err<Rich<'a, char>>> {
    recursive(|instructions_block| {
        // parser for u8s. will give an error for ints which are not u8s.
        let byte = text::int::<_, Err<Rich<char>>>(10).try_map(|s: &str, span| {
            s.parse::<u8>()
                .map_err(|e| Rich::custom(span, format!("Invalid u8: {}", e)))
        });

        // action parser. returns an Instruction.
        let action = choice((
            just("post")
                .then(inline_whitespace())
                .then(just("register"))
                .to(Action::PostRegister),
            just("post")
                .then(inline_whitespace())
                .then(just("flare"))
                .to(Action::PostFlare),
            just("detonate").to(Action::Detonate),
            just("visit").to(Action::Visit),
        ))
        .map(Instruction::Action);

        // operation parser. returns an Instruction.
        let operation = choice((
            just("incr").to(Operation::Increment),
            just("decr").to(Operation::Decrement),
            just("set")
                .then(inline_whitespace())
                .ignore_then(byte.clone())
                .map(|n| Operation::SetValue(n)),
        ))
        .map(Instruction::Operation);

        // condition parser. returns an Instruction.
        let condition = just("if")
            .then(inline_whitespace())
            .ignore_then(choice((
                just("alive").to(Condition::VillagerIsAlive),
                just("dead").to(Condition::VillagerIsDead),
                just("eq")
                    .then(inline_whitespace())
                    .ignore_then(byte.clone())
                    .map(|n| Condition::RegisterEq(n)),
            )))
            .then_ignore(whitespace())
            .then(
                instructions_block
                    .clone()
                    .delimited_by(just('{'), just('}')),
            )
            .map(|(c, ins): (Condition, Instructions)| {
                Instruction::Condition(c, ins.into_iter().rev().collect())
            });

        let repeat = just("repeat")
            .then(inline_whitespace())
            .ignore_then(instructions_block.delimited_by(just('{'), just('}')))
            .map(|ins| Instruction::Repeat(u8::MAX, ins.into_iter().rev().collect()));

        let break_instruction = just("break").to(Instruction::Break);

        // repeat instructions
        choice((action, operation, condition, repeat, break_instruction))
            .padded()
            .repeated()
            .collect::<Vec<_>>()
    })
}

pub fn parse_instructions(path: PathBuf) -> Result<Instructions, MMParsingError> {
    let file_name = path
        .file_name()
        .expect("no file name")
        .to_str()
        .expect("should be valid unicode");
    let mut file =
        File::open(&path).map_err(|_| MMParsingError::FileDoesNotExist(file_name.to_string()))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .map_err(|_| MMParsingError::BadFile)?;

    let parse_result = instructions().parse(&buffer);
    if let Some(instructions) = parse_result.output() {
        return Ok(instructions.clone().into_iter().rev().collect());
    }

    parse_result.errors().for_each(|error| {
        let span = error.span().start()..error.span().end();
        let _ = Report::build(ReportKind::Error, (file_name, span.clone()))
            .with_message(error.to_string())
            .with_label(
                Label::new((file_name, span))
                    .with_color(Color::Red)
                    .with_message("Parsing failed here"),
            )
            .finish()
            .print((file_name, Source::from(buffer.clone())));
    });
    Err(MMParsingError::CannotParse)
}

#[derive(Error, Debug)]
pub enum MMParsingError {
    #[error("`{0}` does not exist")]
    FileDoesNotExist(String),

    #[error("file is not valid UTF-8")]
    BadFile,

    #[error("invalid code")]
    CannotParse,
}

#[cfg(test)]
mod test {
    use chumsky::Parser;

    use crate::{
        mini::{Action, Condition, Instruction},
        parser::instructions,
    };

    #[test]
    fn actions() {
        assert_eq!(
            instructions().parse("if eq 8\t{\n\tpost flare\n}").unwrap(),
            vec![Instruction::Condition(
                Condition::RegisterEq(8),
                vec![Instruction::Action(Action::PostFlare)]
            )]
        )
    }
}
