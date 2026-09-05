//! The lifecycle: the MCP session rules, not the conversation. `/clear` ends the
//! conversation and leaves the session alive, so the flipchart outlives it.
//!
//! The two death signals —`SIGINT` first, EOF on stdin after— are handled on the
//! server thread, and it is that thread which decides when the process exits:
//! with the window fully covered macOS does not slow the event loop down, it
//! **stops** it, and covered is the normal case —the user is in their
//! terminal—. The event loop is not a clock.

use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use crate::wire::Wire;

/// The goodbye: just enough for whoever was watching on the second monitor to
/// learn why the window is disappearing. Leaving it on screen would turn the
/// ephemeral into a broken promise.
const FAREWELL: Duration = Duration::from_millis(2500);

pub fn the_session_is_over(viewer: &Wire) -> ! {
    if viewer.say_goodbye() {
        sleep(FAREWELL);
    }
    exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::wire;

    #[test]
    fn the_goodbye_lasts_between_two_and_three_seconds() {
        assert!((Duration::from_secs(2)..=Duration::from_secs(3)).contains(&FAREWELL));
    }

    #[test]
    fn a_session_that_never_opened_a_window_has_nobody_to_say_goodbye_to() {
        let (viewer, _commands) = wire();

        assert!(!viewer.say_goodbye());
    }
}
