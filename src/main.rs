use std::sync::mpsc::channel;
use std::thread;

use flipchart::{keep_awake_while_the_session_lasts, serve, stay_out_of_the_dock};
use objc2::MainThreadMarker;

fn main() {
    let _activity = keep_awake_while_the_session_lasts();
    stay_out_of_the_dock(MainThreadMarker::new().expect("main() runs on the main thread"));

    let (viewer, commands) = channel();
    thread::spawn(move || serve(viewer));

    while commands.recv().is_ok() {}
}
