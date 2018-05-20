use futures::{Poll, Async, Stream};
use futures::sync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};

pub struct ForkState<S> where S: Stream {
    stream: S,
    tx: UnboundedSender<S::Item>,
}

impl<S> Stream for ForkState<S>
    where S: Stream,
          S::Item: Clone {

    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        match self.stream.poll() {
            Ok(Async::Ready(Some(i))) => {
                match self.tx.unbounded_send(i.clone()) {
                    Ok(_) => Ok(Async::Ready(Some(i))),
                    Err(_) => {
                        panic!("forked stream must be polled");
                    }
                }
            },
            a => a,
        }
    }
}

pub trait Fork: Stream {

    fn fork(self: Self) -> (ForkState<Self>, UnboundedReceiver<Self::Item>) where Self: Sized {

        let (tx, fork) = unbounded();

        (ForkState {
            stream: self,
            tx,
        }, fork)
    }
}

impl<S> Fork for S where S: Stream {}

