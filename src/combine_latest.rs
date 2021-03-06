use futures::{Async, Poll};
use futures::stream::{Stream, Fuse};

/// An adapter for combining the output of two streams.
///
/// The combined stream produces a tuple of items from both of the underlying
/// streams as they become available. Errors, however, are not combined: you
/// get at most one error at a time.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct CombineState<S1: Stream, S2: Stream> {
    stream1: Fuse<S1>,
    stream2: Fuse<S2>,
    queued1: Option<S1::Item>,
    queued2: Option<S2::Item>
}

pub trait CombineLatest: Stream {

    fn combine_latest<S2>(self: Self, stream2: S2) -> CombineState<Self, S2>
        where Self: Stream + Sized,
              S2: Stream<Error = Self::Error>,
    {
        CombineState {
            stream1: self.fuse(),
            stream2: stream2.fuse(),
            queued1: None,
            queued2: None,
        }
    }
}

impl<S1, S2> Stream for CombineState<S1, S2>
    where S1: Stream,
          S1::Item: Clone,
          S2: Stream<Error = S1::Error>,
          S2::Item: Clone,
{
    type Item = (S1::Item, S2::Item); 
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        let mut value = false;

        match self.stream1.poll()? {
            Async::Ready(Some(item1)) => {
                self.queued1 = Some(item1);
                value = true;
            },
            _ => {},
        }

        match self.stream2.poll()? {
            Async::Ready(Some(item2)) => {
                self.queued2 = Some(item2);
                value = true;
            },
            _ => {},
        }

        if self.stream1.is_done() && self.stream2.is_done() {
            return Ok(Async::Ready(None))
        }

        if value && self.queued1.is_some() && self.queued2.is_some() {
            let pair = (self.queued1.clone().unwrap(),
                        self.queued2.clone().unwrap());
            Ok(Async::Ready(Some(pair)))
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl<S> CombineLatest for S where S: Stream {}
