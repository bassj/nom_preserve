use std::{cell::RefCell, rc::Rc};

use nom::{IResult, Parser, Err, error::ParseError};

#[derive(Debug)]
pub enum PreservedErrorInner<E> {
    Default(E),
    Blamed(E),
}

#[derive(Debug)]
pub struct PreservedError<I, E: ParseError<I>> {
    pub _marker: std::marker::PhantomData<I>,
    pub inner: PreservedErrorInner<E>,
}

impl<I, E:ParseError<I>> PreservedError<I, E> {
    pub fn into_inner(self) -> E {
        match self.inner {
            PreservedErrorInner::Blamed(e) => e,
            PreservedErrorInner::Default(e) => e,
        }
    }

    pub fn or(self, other: E) -> Self {
        todo!()
    }
}

// impl<I, E: ParseError<I>> ParseError<I> for PreservedError<I, E> {
//     fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
//         todo!()
//     }

//     fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
//         todo!()
//     }

//     fn from_char(input: I, c: char) -> Self {
//         todo!()
//     }

//     fn or(self, other: Self) -> Self {
//         todo!()
//     }
// }

impl<I, E: ParseError<I>> From<E> for PreservedError<I, E> {
    fn from(value: E) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            inner: PreservedErrorInner::Default(value),
        }
    }
}

/// If the result of the provided parser is an Error, preserve it in the stack for later reference.
pub fn preserve<'p, I: 'p, E: ParseError<I> + Clone + 'p, F, O>(
    stack: Rc<RefCell<Vec<PreservedError<I, E>>>>,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, O, E> + 'p
where
    F: Parser<I, O, E> + 'p,
{
    move |i: I| match f.parse(i) {
        Ok(o) => Ok(o),
        Err(Err::Error(e)) => {
            stack.borrow_mut().push(e.clone().into());
            Err(Err::Error(e))
        }
        Err(e) => Err(e),
    }
}

pub fn blame<I, O, E: ParseError<I> + std::fmt::Debug, F>(
    mut parser: F,
) -> impl FnMut(I) -> IResult<I, (O, Vec<PreservedError<I, E>>), PreservedError<I, E>>
where
    F: Parser<I, (O, Vec<PreservedError<I, E>>), E>,
{
    move |input: I| match parser.parse(input) {
        Err(err) => match err {
            nom::Err::Error(err) => Err(nom::Err::Error(PreservedError {
                _marker: std::marker::PhantomData,
                inner: PreservedErrorInner::Blamed(err),
            })),
            nom::Err::Failure(err) => Err(nom::Err::Failure(PreservedError {
                _marker: std::marker::PhantomData,
                inner: PreservedErrorInner::Blamed(err),
            })),
            nom::Err::Incomplete(_) => unimplemented!(),
        },
        Ok(o) => Ok(o),
    }
}