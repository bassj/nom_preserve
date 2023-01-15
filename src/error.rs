use std::{cell::RefCell, rc::Rc};

use nom::{IResult, Parser, Err, error::ParseError};

pub type ErrorStack<I, E> = Vec<PreservedError<I, E>>;

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
        let inner = match self.inner {
            PreservedErrorInner::Default(e) => PreservedErrorInner::Default(e.or(other)),
            PreservedErrorInner::Blamed(e) => PreservedErrorInner::Blamed(e.or(other)),
        };

        Self {
            _marker: std::marker::PhantomData,
            inner,
        }
    }
}

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


pub fn blame<I: std::fmt::Debug, O, E: ParseError<I> + std::fmt::Debug, F>(
    mut parser: F,
) -> impl FnMut(I) -> IResult<I, (O, ErrorStack<I, E>), E>
where
    F: Parser<I, (O, ErrorStack<I, E>), E>,
{
    move |input: I| match parser.parse(input) {
        Ok((input, (output, mut error_stack))) => {
            if let Some(last) = error_stack.pop() {
                let inner = match last.inner {
                    PreservedErrorInner::Default(err) => PreservedErrorInner::Blamed(err),
                    e => e,
                };

                error_stack.push(PreservedError {
                    _marker: std::marker::PhantomData,
                    inner,
                });
            }

            Ok((input, (output, error_stack)))
        },
        e => e,
        // Err(err) => match err {
        //     nom::Err::Error(err) => Err(nom::Err::Error(PreservedError {
        //         _marker: std::marker::PhantomData,
        //         inner: PreservedErrorInner::Blamed(err),
        //     })),
        //     nom::Err::Failure(err) => Err(nom::Err::Failure(PreservedError {
        //         _marker: std::marker::PhantomData,
        //         inner: PreservedErrorInner::Blamed(err),
        //     })),
        //     nom::Err::Incomplete(_) => unimplemented!(),
        // },
        // Ok(o) => Ok(o),
    }
}

pub fn wrap<I, O, E: ParseError<I> + std::fmt::Debug, F>(
    mut parser: F,
) -> impl FnMut(I) -> IResult<I, (O, Vec<PreservedError<I, E>>), PreservedError<I, E>>
where
    F: Parser<I, (O, Vec<PreservedError<I, E>>), E>,
{
    move |input: I| match parser.parse(input) {
        Err(err) => match err {
            nom::Err::Error(err) => Err(nom::Err::Error(PreservedError {
                _marker: std::marker::PhantomData,
                inner: PreservedErrorInner::Default(err),
            })),
            nom::Err::Failure(err) => Err(nom::Err::Failure(PreservedError {
                _marker: std::marker::PhantomData,
                inner: PreservedErrorInner::Default(err),
            })),
            nom::Err::Incomplete(_) => unimplemented!(),
        },
        Ok(o) => Ok(o),
    }
}