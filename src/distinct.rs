use std::marker::Sized;
use futures::{Stream, Async, Poll};

/// A stream combinator used to filter out duplicates.
///
/// This structure is produced by the `Stream::distinct` method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct DistinctState<S, F> where S: Stream {
    stream: S,
    f: F,
    value: Option<S::Item>,
}

impl<S, F> Stream for DistinctState<S, F>
    where S: Stream,
          F: FnMut(&S::Item, &S::Item) -> bool,
          S::Item: Clone
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match try_ready!(self.stream.poll()) {
                Some(e) => {
                    let mut distinct = false;
                    match self.value {
                        None => {
                            distinct = true;
                        }
                        Some(ref v) => {
                            if !(self.f)(&e, &v) {
                                distinct = true;
                            }
                        }
                    }
                    if distinct {
                        self.value = Some(e.clone());
                        return Ok(Async::Ready(Some(e)))
                    }
                }
                None => return Ok(Async::Ready(None)),
            }
        }
    }
}

pub trait Distinct: Stream {

    fn distinct<F>(self: Self, f: F) -> DistinctState<Self, F>
        where F: FnMut(&Self::Item, &Self::Item) -> bool,
              Self: Sized
    {
        DistinctState {
            stream: self,
            f: f,
            value: None
        }
    }
}

impl<S> Distinct for S where S: Stream {}

