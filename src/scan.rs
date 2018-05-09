use futures::{Poll, Async};
use futures::stream::Stream;

/// It's like fold but yields aggregated  values over time
///
/// This stream is returned by the `Stream::scan method.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct ScanState<S, F, T> {
    stream: S,
    f: F,
    // This could be initialized to None if scan
    // doesn't have initial value (would have to be different operator)
    state: Option<T>, 
}

impl<S, F, T> Stream for ScanState<S, F, T>
    where S: Stream,
          F: FnMut(T, S::Item) -> T,
          T: Clone
{
    type Item = T;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.stream.poll()? {
            Async::Ready(Some(v)) => {
                let state = self.state.take();
                let new_state = (self.f)(state.unwrap(), v);
                self.state = Some(new_state.clone());
                Ok(Async::Ready(Some(new_state.clone())))
            }
            Async::Ready(None) => Ok(Async::Ready(None)),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub trait Scan: Stream {

    fn scan<F, T>(self: Self, t: T, f: F) -> ScanState<Self, F, T>
    where F: FnMut(T, Self::Item) -> T,
          Self: Sized,
          T: Clone,
    { 
        ScanState {
            stream: self,
            f: f,
            state: Some(t),
        }
    }
}

impl<S> Scan for S where S: Stream {}

