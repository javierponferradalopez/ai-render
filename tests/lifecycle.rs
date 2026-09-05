use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::json;

/// A session without a single `show`: there is no window to say goodbye to, so
/// the process exits as soon as the server thread learns about the death.
struct Session {
    process: Child,
    output: BufReader<ChildStdout>,
}

impl Session {
    /// Really initialised —with the answer read—, which is what makes sure the
    /// server thread is up and listening for the two deaths.
    fn open() -> Self {
        let mut process = Command::new(env!("CARGO_BIN_EXE_flipchart"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the flipchart binary starts");
        let output = BufReader::new(process.stdout.take().unwrap());
        let mut session = Self { process, output };
        session.initialise();
        session
    }

    fn initialise(&mut self) {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0" }
            }
        });
        let input = self.process.stdin.as_mut().unwrap();
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
        let mut answer = String::new();
        self.output
            .read_line(&mut answer)
            .expect("the server answers the initialize");
    }

    fn closes_its_input(&mut self) {
        drop(self.process.stdin.take());
    }

    fn receives_sigint(&mut self) {
        let killed = Command::new("kill")
            .args(["-INT", &self.process.id().to_string()])
            .status()
            .expect("kill -INT runs");
        assert!(killed.success());
    }

    fn exits_before(&mut self, deadline: Duration) -> ExitStatus {
        let limit = Instant::now() + deadline;
        while Instant::now() < limit {
            if let Some(status) = self
                .process
                .try_wait()
                .expect("the process can be looked at")
            {
                return status;
            }
            sleep(Duration::from_millis(20));
        }
        panic!("the process outlived its session");
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

const MARGIN: Duration = Duration::from_secs(5);

#[test]
fn the_eof_on_stdin_ends_the_process() {
    let mut session = Session::open();

    session.closes_its_input();

    assert!(session.exits_before(MARGIN).success());
}

#[test]
fn the_sigint_ends_the_process() {
    let mut session = Session::open();

    session.receives_sigint();

    assert!(session.exits_before(MARGIN).success());
}
