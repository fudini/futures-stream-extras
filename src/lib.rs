extern crate futures;

//pub mod flat_map;
//pub mod distinct;
//pub mod scan2;
//pub mod scan3;
pub mod combine_latest;
pub mod with_latest_from;
pub mod fork;

#[cfg(test)]
mod tests {

    use std::rc::Rc;
    use std::thread;
    use std::time::Duration;
    use futures::sync::mpsc::{channel, unbounded};
    use futures::{Future, Sink, Stream};
    use futures::stream::iter_ok;
    //use futures2::StreamExt;
    //use futures2::stream::{self, iter_ok};
    //use futures2::executor::block_on;

    //use flat_map::*;
    //use distinct::*;
    //use combine_latest::*;
    //use scan3::*;
    use with_latest_from::*;
    use fork::*;

    //#[test]
    //fn flat_map_test() {

        //let v: Vec<u8> = vec!(1, 2, 3 ,4 ,5);
        //let c: Vec<u8> = block_on(iter_ok::<_, ()>(v)
            //.flat_map(|v| iter_ok::<_, ()>(vec![v, v + 10]))
            //.collect()).unwrap();

        //let r: Vec<u8> = vec![1, 11, 2, 12, 3, 13, 4, 14, 5, 15];
        //assert_eq!(c, r);
    //}

    //#[test]
    //fn distinct_test() {

        //let v: Vec<u8> = vec!(1, 2, 2, 3, 3, 3, 1);
        //let c: Vec<u8> = block_on(iter_ok::<_, ()>(v)
            //.distinct(|a, b| a == b)
            //.collect()).unwrap();

        //let r: Vec<u8> = vec!(1, 2, 3, 1);

        //assert_eq!(c, r);
    //}

    //#[test]
    //fn fork_test() {

        //let v: Vec<u8> = vec!(1, 2, 3);

        //let (original, fork) = iter_ok::<_, ()>(v).fork();

        //let c1: Vec<u8> = block_on(original.map(|a| a * 2)
            //.collect()).unwrap();

        //let c2: Vec<u8> = block_on(fork.map(|a| a * 3)
            //.collect()).unwrap();

        //let r1: Vec<u8> = vec!(2, 4, 6);
        //let r2: Vec<u8> = vec!(3, 6, 9);

        //assert_eq!(c1, r1);
        //assert_eq!(c2, r2);
    //}

    //#[test]
    //fn fork_test2() {

        //let v: Vec<Rc<u8>> = vec!(Rc::new(1), Rc::new(2), Rc::new(3));

        //let (original, fork) = iter_ok::<_, ()>(v).fork();

        //let c1: Vec<u8> = block_on(original.map(|a| *a * 2)
            //.collect()).unwrap();

        //let c2: Vec<u8> = block_on(fork.map(|a| *a * 3)
            //.collect()).unwrap();

        //let r1: Vec<u8> = vec!(2, 4, 6);
        //let r2: Vec<u8> = vec!(3, 6, 9);

        //assert_eq!(c1, r1);
        //assert_eq!(c2, r2);
    //}

    //// this test is a bit rubbish
    //// need to find a way to test it with correct order
    //#[test]
    //fn combine_latest() {

        //let v1: Vec<u8> = vec!(1, 3, 5, 6, 7, 9);
        //let v2: Vec<u8> = vec!(2, 4, 8);

        //let s1 = iter_ok::<_, ()>(v1);
        //let s2 = iter_ok::<_, ()>(v2);

        //let r: Vec<(u8, u8)> = block_on(s1.combine_latest(s2).collect()).unwrap();
        //let c = vec!((1, 2), (3, 4), (5, 8), (6, 8), (7, 8), (9, 8));

        //assert_eq!(c, r);
    //}

    #[test]
    fn with_latest_from() {

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

    //#[test]
    //fn scan() {

        //let get_stream = || stream::iter_ok::<_, ()>(vec!(1, 2, 3, 4, 5, 6));

        //let r1: Vec<u8> = block_on(get_stream()
            //.scan3(0, |a, v| a + v)
            //.collect()).unwrap();

        //let c1 = vec![1, 3, 6, 10, 15, 21];

        //assert_eq!(r1, c1);

        ////let r6: Vec<u8> = block_on(get_stream()
            ////.scan3(
                ////Rc::new(RefCell::new(vec![])),
                ////|a, v| {
                    ////(*a).borrow_mut().push(v);
                    ////a
                ////}
            ////)
            ////.collect()).unwrap();

        ////println!("{:?}", r6);
    //}
}

