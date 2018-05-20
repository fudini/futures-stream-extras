#[macro_use]
extern crate futures;
extern crate tokio_timer;
extern crate tokio_executor;

pub mod flat_map;
pub mod distinct;
pub mod scan;
pub mod scan2;
pub mod combine_latest;
pub mod with_latest_from;
pub mod fork;
pub mod delay;
pub mod value;

mod support;

#[cfg(test)]
mod tests {

    use std::rc::Rc;
    use std::cell::RefCell;
    use std::thread;
    use std::time::{Instant, Duration};

    use futures::sync::mpsc::unbounded;
    use futures::{future, Future, Stream};
    use futures::Async::*;
    use futures::stream::{self, iter_ok};

    use support::*;

    use flat_map::*;
    use distinct::*;
    use combine_latest::*;
    use scan::*;
    use scan2::*;
    use with_latest_from::*;
    use fork::*;
    use delay::*;
    use value::*;

    #[test]
    fn flat_map_test() {

        let v: Vec<u8> = vec!(1, 2, 3 ,4 ,5);
        let c: Vec<u8> = iter_ok::<_, ()>(v)
            .flat_map(|v| iter_ok::<_, ()>(vec![v, v + 10]))
            .collect().wait().unwrap();

        let r: Vec<u8> = vec![1, 11, 2, 12, 3, 13, 4, 14, 5, 15];
        assert_eq!(c, r);
    }

    #[test]
    fn distinct_test() {

        let v: Vec<u8> = vec!(1, 2, 2, 3, 3, 3, 1);
        let c: Vec<u8> = iter_ok::<_, ()>(v)
            .distinct(|a, b| a == b)
            .collect().wait().unwrap();

        let r: Vec<u8> = vec!(1, 2, 3, 1);

        assert_eq!(c, r);
    }

    #[test]
    fn fork_test() {

        let v: Vec<u8> = vec!(1, 2, 3);

        let (original, fork) = iter_ok::<_, ()>(v).fork();

        let c1: Vec<u8> = original.map(|a| a * 2)
            .collect().wait().unwrap();

        let c2: Vec<u8> = fork.map(|a| a * 3)
            .collect().wait().unwrap();

        let r1: Vec<u8> = vec!(2, 4, 6);
        let r2: Vec<u8> = vec!(3, 6, 9);

        assert_eq!(c1, r1);
        assert_eq!(c2, r2);
    }

    #[test]
    fn fork_test2() {

        let v: Vec<Rc<u8>> = vec!(Rc::new(1), Rc::new(2), Rc::new(3));

        let (original, fork) = iter_ok::<_, ()>(v).fork();

        let c1: Vec<u8> = original.map(|a| *a * 2)
            .collect().wait().unwrap();

        let c2: Vec<u8> = fork.map(|a| *a * 3)
            .collect().wait().unwrap();

        let r1: Vec<u8> = vec!(2, 4, 6);
        let r2: Vec<u8> = vec!(3, 6, 9);

        assert_eq!(c1, r1);
        assert_eq!(c2, r2);
    }

    #[test]
    fn combine_latest_test() {

        let (tx1, rx1) = unbounded::<u32>();
        let (tx2, rx2) = unbounded::<u32>();

        // tx1: 1     4 5 6
        // tx2:   2 3       7
        //  cl:   1 1 4 5 6 6
        //        2 3 3 3 3 7
        thread::spawn(move || {
            tx1.unbounded_send(1).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(2).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(3).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(4).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(5).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(6).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(7).unwrap();
            thread::sleep(Duration::from_millis(1));
        });

        let r: Vec<(u32, u32)> = rx1.combine_latest(rx2)
            .collect().wait().unwrap();

        let c = vec![(1, 2), (1, 3), (4, 3), (5, 3), (6, 3), (6, 7)];

        assert_eq!(c, r);
    }

    #[test]
    fn with_latest_from_test() {

        let (tx1, rx1) = unbounded::<u32>();
        let (tx2, rx2) = unbounded::<u32>();

        thread::spawn(move || {
            tx1.unbounded_send(1).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(2).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(3).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(4).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(5).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx1.unbounded_send(6).unwrap();
            thread::sleep(Duration::from_millis(1));
            tx2.unbounded_send(7).unwrap();
            thread::sleep(Duration::from_millis(1));
        });

        let r: Vec<(u32, u32)> = rx1.with_latest_from(rx2)
            .collect().wait().unwrap();

        let c = vec!((1, 2), (4, 3), (5, 3), (6, 3));

        assert_eq!(c, r);
    }

    #[test]
    fn scan_test() {

        let get_stream = || stream::iter_ok::<_, ()>(vec!(1, 2, 3, 4, 5, 6));

        let result: Vec<u8> = get_stream()
            .scan(0, |a, v| a + v)
            .collect().wait().unwrap();

        let expected = vec![1, 3, 6, 10, 15, 21];

        assert_eq!(result, expected);
    }

    #[test]
    fn scan_test_2() {

        let get_stream = || stream::iter_ok::<_, ()>(vec!(1, 2, 3, 4, 5, 6));

        let result = get_stream()
            .scan(
                Rc::new(RefCell::new(vec![])),
                |a, v| {
                    (*a).borrow_mut().push(v);
                    a
                }
            )
            .collect().wait().unwrap();

        let r = Rc::new(RefCell::new(vec![1, 2, 3, 4, 5, 6]));
        let expected: Vec<Rc<RefCell<Vec<u8>>>> = (0..6).map(|_| r.clone()).collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn scan2_test() {

        let get_stream = || stream::iter_ok::<_, ()>(vec!(1, 2, 3, 1, 2, 3));

        let result: Vec<usize> = get_stream()
            .scan2(
                vec![],
                |mut acc, v| {
                    acc.push(v);
                    let len = acc.len();
                    (acc, len)
                }
            )
            .collect().wait().unwrap();

        let expected = vec![1, 2, 3, 4, 5, 6];
        
        assert_eq!(result, expected);
    }

    #[test]
    fn value_future_test() {
        assert_eq!(future::ok::<u8, ()>(10).value(100).wait().unwrap(), 100);
    }

    #[test]
    fn delay_test() {

        let get_stream = || stream::iter_ok::<_, ()>(vec![1, 2, 3, 4, 5]);

        mocked(|timer, time| {

            let mut stream = get_stream()
                .flat_map(|v| {
                    stream::once(Ok(v)).delay(Duration::from_millis(100))
                });

            // TODO: tests
            
        })
    }
}

