use futures::Stream;
use futures::stream::{Flatten, Map};

/// Basically map + flatten
///
/// This stream is returned by the `Stream::flat_map method.
pub trait FlatMap: Stream {

    fn flat_map<S, F>(self: Self, f: F) -> Flatten<Map<Self, F>>
        where S: Stream<Error = Self::Error>,
              Self: Stream + Sized,
              F: FnMut(Self::Item) -> S,
    {
        self.map(f).flatten()
    }
}

impl<S> FlatMap for S where S: Stream {}

