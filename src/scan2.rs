use std::marker::PhantomData;
use futures2::{Poll, Async};
use futures2::task::Context;
use futures2::stream::Stream;

/// It's like fold but yields a aggregated value over time
///
/// This stream is returned by the `Stream::scan2 method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct Scan2State<S, F, T, R> {
    stream: S,
    f: F,
    // This could be initialized to None if scan
    // doesn't have initial value
    state: Option<T>, 
    _marker: PhantomData<R>,
}

impl<S, F, T, R> Stream for Scan2State<S, F, T, R>
    where S: Stream,
          F: FnMut(T, S::Item) -> (T, R),
{
    type Item = R;
    type Error = S::Error;

    fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<Self::Item>, S::Error> {
        match self.stream.poll_next(cx)? {
            Async::Ready(Some(v)) => {
                let state = self.state.take();
                let (new_state, next_value) = (self.f)(state.unwrap(), v);
                self.state = Some(new_state);
                Ok(Async::Ready(Some(next_value)))
            }
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::Pending => Ok(Async::Pending),
        }
    }
}

pub trait Scan2: Stream {

    fn scan2<F, T, R>(self: Self, t: T, f: F) -> Scan2State<Self, F, T, R>
    where F: FnMut(T, Self::Item) -> (T, R),
          Self: Sized,
    { 
        Scan2State {
            stream: self,
            f: f,
            state: Some(t),
            _marker: PhantomData,
        }
    }
}

impl<S> Scan2 for S where S: Stream {}

