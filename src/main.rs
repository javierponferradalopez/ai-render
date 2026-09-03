use std::thread;

use flipchart::{
    keep_awake_while_the_session_lasts, open_at_the_first_show, serve, stay_out_of_the_dock, wire,
};
use objc2::MainThreadMarker;

fn main() {
    let _activity = keep_awake_while_the_session_lasts();
    stay_out_of_the_dock(MainThreadMarker::new().expect("main() runs on the main thread"));

    let (viewer, commands) = wire();
    thread::spawn(move || serve(viewer));

    open_at_the_first_show(commands);
}
