use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

struct MyFuture {
    is_ready: bool,
}

impl MyFuture {
    fn new() -> MyFuture {
        MyFuture { is_ready: false }
    }

    fn make_ready(&mut self) {
        self.is_ready = true;
    }
}

impl Future for MyFuture {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.is_ready {
            Poll::Ready("Future is now ready!")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn create_waker(call_back: Arc<Mutex<bool>>) -> Waker {
    let raw_waker = RawWaker::new(
        Arc::into_raw(call_back) as *const (),
        &RawWakerVTable::new(
            clone_callback,
            wake_callback,
            wake_by_ref_callback,
            drop_callback,
        ),
    );

    unsafe { Waker::from_raw(raw_waker) }
}

unsafe fn clone_callback(ptr: *const ()) -> RawWaker {
    let arc = Arc::from_raw(ptr as *const Mutex<bool>);
    let clone = Arc::clone(&arc);
    std::mem::forget(arc);

    RawWaker::new(
        Arc::into_raw(clone) as *const (),
        &RawWakerVTable::new(
            clone_callback,
            wake_callback,
            wake_by_ref_callback,
            drop_callback,
        ),
    )
}

unsafe fn wake_callback(ptr: *const ()) {
    let arc = Arc::from_raw(ptr as *const Mutex<bool>);
    *arc.lock().unwrap() = true;
    std::mem::forget(arc);
}

unsafe fn wake_by_ref_callback(ptr: *const ()) {
    let arc = Arc::from_raw(ptr as *const Mutex<bool>);
    *arc.lock().unwrap() = true;
    std::mem::forget(arc);
}
unsafe fn drop_callback(ptr: *const ()) {
    drop(Arc::from_raw(ptr as *const Mutex<bool>));
}

#[test]
fn test_customer_waker() {
    // Shared state for the custom Waker
    let ready_state = Arc::new(Mutex::new(false));
    let waker = create_waker(ready_state.clone());
    let mut my_future = MyFuture::new();
    let mut cx = Context::from_waker(&waker);
    // Poll the future
    match Pin::new(&mut my_future).poll(&mut cx) {
        Poll::Ready(result) => println!("{}", result),
        Poll::Pending => {
            println!("Future is not ready. Waking the task...");
        }
    }
    // Simulate making the future ready
    my_future.make_ready();
    // Poll the future again
    match Pin::new(&mut my_future).poll(&mut cx) {
        Poll::Ready(result) => println!("{}", result),
        Poll::Pending => println!("Future is still not ready."),
    }
}
