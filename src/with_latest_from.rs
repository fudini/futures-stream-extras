use futures::{Async, Poll};
use futures::stream::{Stream, Fuse};

/// An adapter for combining the output of two streams.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct WithLatestFromState<S1: Stream, S2: Stream> {
    stream1: Fuse<S1>,
    stream2: Fuse<S2>,
    queued1: Option<S1::Item>,
    queued2: Option<S2::Item>,
}

pub trait WithLatestFrom: Stream {

    fn with_latest_from<S2>(self: Self, stream2: S2) -> WithLatestFromState<Self, S2>
        where Self: Stream + Sized,
              S2: Stream<Error = Self::Error>,
    {
        WithLatestFromState {
            stream1: self.fuse(),
            stream2: stream2.fuse(),
            queued1: None,
            queued2: None,
        }
    }
}

impl<S1, S2> Stream for WithLatestFromState<S1, S2>
    where S1: Stream,
          S1::Item: Clone,
          S2: Stream<Error = S1::Error>,
          S2::Item: Clone,
{
    type Item = (S1::Item, S2::Item); 
    type Error = S1::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        match self.stream1.poll()? {
            Async::Ready(Some(item)) => {
                self.queued1 = Some(item);
            },
            _ => {},
        }

        match self.stream2.poll()? {
            Async::Ready(Some(item2)) => {
                self.queued2 = Some(item2)
            },
            _ => {},
        }

        if self.queued1.is_some() && self.queued2.is_some() {
            let pair = (self.queued1.take().unwrap(), self.queued2.clone().unwrap());
            return Ok(Async::Ready(Some(pair)))
        }

        if self.stream1.is_done() && self.stream2.is_done() {
            return Ok(Async::Ready(None))
        }

        Ok(Async::NotReady)
    }
}

impl<S> WithLatestFrom for S where S: Stream {}
