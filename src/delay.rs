use std::time::{Duration, Instant};
use futures::{Future, Stream};
use futures::stream::{Flatten, Map};
use tokio_timer::Delay;

/// Delay stream by Duration
///
/// This stream is returned by the `Stream::delay method.
pub trait DelayStream: Stream {

    fn delay<F>(self: Self, duration: Duration) -> Flatten<Map<Self, F>>
        where Self: Stream + Sized,
              F: FnMut(Self::Item) -> Delay,
    {

        let delay_future = Delay::new(Instant::now() + duration);

        self.map(|v| delay_future.map(move |_| v).into_stream())
            .flatten()
    }
}

impl<S> DelayStream for S where S: Stream {}


