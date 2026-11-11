/*
 * running a program with a time limit: std's `output()` waits as long as the
 * program does, and a stuck tailscale or ddcutil then leaves the thread that
 * asked (and the busy flag it guards) stuck until the shell restarts
 *
 * stdout and stderr are read on their own threads while the program runs, so a
 * program that writes a lot can't fill a pipe and block before it exits. on a
 * timeout the program is killed and waited for, so no zombie stays behind
 *
 * meant for programs that answer and exit (tailscale, ddcutil), not for ones
 * that start something in the background that outlives them
 */

use std::io::{self, Read, Write};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// how often a running program is checked on; short next to any time limit
const POLL: Duration = Duration::from_millis(20);

// curl's own limit for a whole transfer, in seconds; the shell only fetches small things
pub const CURL_MAX_TIME: &str = "10";

// past curl's own limit, for a curl that hangs anyway
pub const CURL_TIMEOUT: Duration = Duration::from_secs(15);

// why a program gave no output: callers word these differently (not installed vs. stuck)
#[derive(Debug)]
pub enum Failure {
    Start,
    Timeout,
}

pub fn capture(command: &mut Command, timeout: Duration) -> Result<Output, Failure> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| Failure::Start)?;

    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());

    /*
     * after a timeout the readers are left to finish on their own: a child the
     * killed program started could keep a pipe open, and joining would wait on it
     */
    let status = wait(&mut child, timeout).ok_or(Failure::Timeout)?;

    Ok(Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

// what a program printed when it ran and succeeded in time; None otherwise
pub fn read(command: &mut Command, timeout: Duration) -> Option<String> {
    let output = capture(command, timeout).ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

// the exit status, or None after killing a program that ran out of time
fn wait(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + timeout;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL),
            _ => break,
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    None
}

/*
 * hands input to a program started with a piped stdin and waits for it: stdin
 * is closed after writing so the program sees the end, and the program is
 * waited for even when writing failed, or it would stay behind as a zombie.
 * stdout is collected only when the caller piped it
 */
pub fn feed(mut child: Child, input: &[u8]) -> io::Result<Output> {
    let written = match child.stdin.take() {
        Some(mut stdin) => stdin.write_all(input),
        None => Err(io::Error::other("stdin isn't piped")),
    };

    let output = child.wait_with_output()?;

    written.map(|()| output)
}

// reads a pipe to its end on a thread; the end comes when the program exits or is killed
fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<Vec<u8>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();

        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }

        bytes
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quick_program_gives_its_output() {
        let output =
            capture(Command::new("echo").arg("hi"), Duration::from_secs(5)).expect("echo runs");

        assert!(output.status.success());
        assert_eq!(output.stdout, b"hi\n");
    }

    #[test]
    fn a_slow_program_is_stopped_at_the_limit() {
        let started = Instant::now();

        let result = capture(Command::new("sleep").arg("5"), Duration::from_millis(100));

        assert!(matches!(result, Err(Failure::Timeout)));
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn input_is_handed_over_and_the_program_waited_for() {
        let child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("cat runs");

        let output = feed(child, b"handed over").expect("fed");

        assert!(output.status.success());
        assert_eq!(output.stdout, b"handed over");
    }

    #[test]
    fn a_missing_program_is_an_error() {
        let result = capture(
            &mut Command::new("no-such-program-here"),
            Duration::from_secs(1),
        );

        assert!(matches!(result, Err(Failure::Start)));
    }
}
