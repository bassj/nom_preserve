use nom::{error::ParseError, IResult, Parser};

use crate::error::{ErrorStack, PreservedError, PreservedErrorInner};

fn _preserve_error_stack<I: std::fmt::Debug, E: ParseError<I> + std::fmt::Debug>(
    base: impl Into<PreservedError<I, E>>,
    error_stack: Vec<PreservedError<I, E>>,
) -> E {
    let mut _base = base.into();
    for stack_err in error_stack {
        let stack_err: PreservedError<I, E> = stack_err.into();
        match stack_err.inner {
            PreservedErrorInner::Default(err) => _base = _base.or(err),
            PreservedErrorInner::Blamed(err) => _base = err.into(),
        }
    }

    _base.into_inner()
}

pub fn terminated<I: std::fmt::Debug, O1, O2, E: ParseError<I> + std::fmt::Debug, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, (O1, Vec<E>), E>
where
    F: Parser<I, (O1, Vec<PreservedError<I, E>>), E>,
    G: Parser<I, O2, E>,
{
    move |input: I| {
        let (input, (o1, error_stack)) = first
            .parse(input)?;

        match second.parse(input) {
            Ok((i, _)) => Ok((
                i,
                (
                    o1,
                    error_stack
                        .into_iter()
                        .map(|ipe| ipe.into_inner())
                        .collect(),
                ),
            )),
            Err(e) => match e {
                nom::Err::Error(e) => Err(nom::Err::Error(_preserve_error_stack(e, error_stack))),
                nom::Err::Failure(e) => {
                    Err(nom::Err::Failure(_preserve_error_stack(e, error_stack)))
                }

                _ => todo!(),
            },
        }
    }
}

pub fn delimited<I: std::fmt::Debug, O1, O2, O3, E: ParseError<I> + std::fmt::Debug, F, G, H>(
    mut first: F,
    mut second: G,
    mut third: H,
) -> impl FnMut(I) -> IResult<I, (O2, ErrorStack<I, E>), E>
where
    F: Parser<I, O1, E>,
    G: Parser<I, (O2, ErrorStack<I, E>), E>,
    H: Parser<I, O3, E>,
{
    move |input: I| {
        let (input, _) = first.parse(input)?;
        let (input, (o2, error_stack)) = second
            .parse(input)?;

        match third.parse(input) {
            Ok((i, _)) => Ok((i, (o2, error_stack))),
            Err(e) => match e {
                nom::Err::Error(e) => Err(nom::Err::Error(_preserve_error_stack(e, error_stack))),
                nom::Err::Failure(e) => {
                    Err(nom::Err::Failure(_preserve_error_stack(e, error_stack)))
                }
                _ => todo!(),
            },
        }
    }
}
