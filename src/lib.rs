mod diagram;
mod flipchart;
mod mac;
mod server;

pub use flipchart::{Flipchart, ViewerCommand};
pub use mac::{keep_awake_while_the_session_lasts, stay_out_of_the_dock};
pub use server::serve;
