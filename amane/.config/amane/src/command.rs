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

use std::io::Read;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

// how often a running program is checked on; short next to any time limit
const POLL: Duration = Duration::from_millis(20);

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
    fn a_missing_program_is_an_error() {
        let result = capture(
            &mut Command::new("no-such-program-here"),
            Duration::from_secs(1),
        );

        assert!(matches!(result, Err(Failure::Start)));
    }
}
