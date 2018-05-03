use futures2::{Stream, StreamExt};
use futures2::stream::{Flatten, Map};

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

