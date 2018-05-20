use std::fmt::Debug;
use std::time::{Duration, Instant};
use futures::{Future, Stream, Async, Poll};
use self::Async::*;
use tokio_timer::Delay;
use value::*;

/// A stream combinator used to delay stream.
///
/// This structure is produced by the `Stream::delay` method.
#[must_use = "streams do nothing unless polled"]
pub struct DelayState<S> where S: Stream {
    stream: S,
    duration: Duration,
    buffer: Vec<ValueState<Delay, Result<Option<S::Item>, S::Error>>>,
    delay_future: Option<ValueState<Delay, Result<Option<S::Item>, S::Error>>>,
}

impl<S> Stream for DelayState<S>
    where S: Stream,
          S::Item: Debug,
          S::Error: Debug,
{
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        let future_value = match self.stream.poll() {
            Ok(NotReady) => None,
            Ok(Ready(v)) => Some(Ok(v)),
            Err(e) => Some(Err(e)),
        };

        future_value.into_iter()
            .for_each(|v| {
                let delay_future = Delay::new(Instant::now() + self.duration).value(v);
                if self.delay_future.is_none() {
                    self.delay_future = Some(delay_future);
                } else {
                    self.buffer.insert(0, delay_future);
                }
            });

        let result = match self.delay_future.poll() {
            Ok(Ready(Some(Ok(v)))) => Ok(Ready(v)),
            Ok(Ready(Some(Err(e)))) => Err(e),
            Ok(Ready(None)) => Ok(NotReady),
            // TODO: return some meaningful error
            Err(_) => Ok(NotReady),
            Ok(NotReady) => Ok(NotReady),
        };
        
        match result {
            Ok(Ready(Some(_))) => {
                self.delay_future = self.buffer.pop();
            },
            _ => {},
        }
        result
    }
}

pub trait DelayStream: Stream {

    fn delay(self: Self, duration: Duration) -> DelayState<Self>
        where Self: Sized
    {
        DelayState {
            stream: self,
            duration,
            buffer: vec![],
            delay_future: None,
        }
    }
}

impl<S> DelayStream for S where S: Stream {}

