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
    // because I only have like 15 hours to spend on this, I decided against writing my own
    // parser. a little bit of research let me to the chumsky crate you see here,
    // but in retrospect, I should have used something different; this code is a bit
    // unwieldy. it's not too bad, though. all the functions correspond to something
    // that can be matched (some text, whitespace, whatever), and then things like
    // then or choice are used to combine those matches. things like map are
    // then used to convert matches to actual values. see the crate's documenation
    // for more details.

    // condition and repeat both recursively parse instructions,
    // so we have to use recursive()
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
            // the actual condition
            .then(inline_whitespace())
            .ignore_then(choice((
                just("alive").to(Condition::VillagerIsAlive),
                just("dead").to(Condition::VillagerIsDead),
                just("eq")
                    .then(inline_whitespace())
                    .ignore_then(byte.clone())
                    .map(|n| Condition::RegisterEq(n)),
            )))
            // the conditional instructions
            .then_ignore(whitespace())
            .then(
                instructions_block
                    .clone()
                    .delimited_by(just('{'), just('}')),
            )
            // construct the instruction
            .map(|(c, ins): (Condition, Instructions)| {
                Instruction::Condition(c, ins.into_iter().rev().collect())
            });

        // repeat parser. returns an Instruction.
        let repeat = just("repeat")
            .then(whitespace())
            .ignore_then(instructions_block.delimited_by(just('{'), just('}')))
            .map(|ins| Instruction::Repeat(u8::MAX, ins.into_iter().rev().collect()));

        // parses a single break
        let break_instruction = just("break").to(Instruction::Break);

        // match as many instructions of any type as possible
        choice((action, operation, condition, repeat, break_instruction))
            .padded()
            .repeated()
            .collect::<Vec<_>>()
    })
}

pub fn parse_instructions(path: PathBuf) -> Result<Instructions, MMParsingError> {
    // get the file name and contents of the provided file
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

    // parse the instructions and return on success
    let parse_result = instructions().parse(&buffer);
    if let Some(instructions) = parse_result.output() {
        return Ok(instructions.clone().into_iter().rev().collect());
    }

    // on failure, print all the errors
    parse_result.errors().for_each(|error| {
        // again, I chose crates poorly. this error report building is a bit unwieldy.
        // while it's technically a different crate that does the error reporting,
        // they're sister projects
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

/// represents anything that can go wrong with parse_instructions()
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
    use std::u8;

    // given how little time I have for this project, I'm not worried about comprehensive
    // tests here. given the declaritive nature of the parsing crate I'm using, I'm
    // not too worried about being super comprehensive with my testing
    use chumsky::Parser;

    use crate::{
        mini::{Action, Condition, Instruction, Operation},
        parser::instructions,
    };

    #[test]
    fn conditional() {
        assert_eq!(
            instructions().parse("if eq 8\t{\n\tpost flare\n}").unwrap(),
            vec![Instruction::Condition(
                Condition::RegisterEq(8),
                vec![Instruction::Action(Action::PostFlare)]
            )]
        )
    }

    #[test]
    fn repeat() {
        assert_eq!(
            instructions().parse("repeat\n{\n\t set 15}\t\n").unwrap(),
            vec![Instruction::Repeat(
                u8::MAX,
                vec![Instruction::Operation(Operation::SetValue(15))]
            )]
        )
    }

    #[test]
    fn nested() {
        assert_eq!(
            instructions()
                .parse("repeat { if alive { repeat { break } } }")
                .unwrap(),
            vec![Instruction::Repeat(
                u8::MAX,
                vec![Instruction::Condition(
                    Condition::VillagerIsAlive,
                    vec![Instruction::Repeat(u8::MAX, vec![Instruction::Break])]
                )]
            )]
        )
    }
}
