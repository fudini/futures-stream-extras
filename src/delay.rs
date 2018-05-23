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
pub struct DelayState<S, F>
where S: Stream,
      F: Fn() -> Instant,
{
    stream: S,
    duration: Duration,
    buffer: Vec<ValueState<Delay, Result<Option<S::Item>, S::Error>>>,
    delay_future: Option<ValueState<Delay, Result<Option<S::Item>, S::Error>>>,
    get_now: F,
}

impl<S, F> Stream for DelayState<S, F>
    where S: Stream,
          S::Item: Debug,
          S::Error: Debug,
          F: Fn() -> Instant,
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
                let now = (self.get_now)();
                let delay_future = Delay::new(now + self.duration).value(v);
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

    fn delay<F>(self: Self, duration: Duration, get_now: F) -> DelayState<Self, F>
        where Self: Sized,
              F: Fn() -> Instant,
    {
        DelayState {
            stream: self,
            duration,
            buffer: vec![],
            delay_future: None,
            get_now,
        }
    }
}

impl<S> DelayStream for S where S: Stream {}

