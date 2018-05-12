use futures::{Future, Poll, Async};

/// Future for the `value` combinator.
/// It's like map, but discards the value and replaces
/// it with constant value.
///
/// This is created by the `Future::value` method.
#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct ValueState<F, V> where F: Future {
    future: F,
    value: Option<V>,
}

impl<F, V> Future for ValueState<F, V>
    where F: Future,
{
    type Item = V;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        match self.future.poll()? {
            Async::Ready(_) => {
                // Shouldn't poll future more than once
                Ok(Async::Ready(self.value.take().unwrap()))
            },
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub trait Value: Future {

    fn value<V>(self: Self, value: V) -> ValueState<Self, V>
        where Self: Future + Sized
    {
        ValueState {
            future: self,
            value: Some(value),
        }
    }
}

impl<F> Value for F where F: Future {}
