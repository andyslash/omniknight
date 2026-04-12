use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

pub struct TuiHarness {
    child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn Write + Send>,
    output_buf: Arc<Mutex<Vec<u8>>>,
    _reader_handle: std::thread::JoinHandle<()>,
}

impl TuiHarness {
    pub fn spawn(cols: u16, rows: u16) -> Self {
        let pty_system = NativePtySystem::default();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open PTY");

        let binary = cargo_bin_path();
        let mut cmd = CommandBuilder::new(&binary);
        cmd.env("TERM", "xterm-256color");

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .expect("failed to spawn omniknight");

        let reader = pty_pair
            .master
            .try_clone_reader()
            .expect("failed to clone reader");
        let writer = pty_pair
            .master
            .take_writer()
            .expect("failed to take writer");

        let output_buf = Arc::new(Mutex::new(Vec::new()));
        let buf_clone = Arc::clone(&output_buf);

        // Background reader thread — continuously drains PTY output
        let reader_handle = std::thread::spawn(move || {
            let mut reader = reader;
            let mut chunk = [0u8; 4096];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut buf = buf_clone.lock().unwrap();
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            child,
            writer,
            output_buf,
            _reader_handle: reader_handle,
        }
    }

    pub fn send_key(&mut self, key: &str) {
        let bytes = key_to_bytes(key);
        self.writer.write_all(&bytes).expect("failed to write key");
        self.writer.flush().expect("failed to flush");
    }

    pub fn send_char(&mut self, ch: char) {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        self.writer
            .write_all(s.as_bytes())
            .expect("failed to write char");
        self.writer.flush().expect("failed to flush");
    }

    pub fn send_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.send_char(ch);
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Read all accumulated output since last call, stripping ANSI escapes.
    pub fn capture_frame(&mut self, settle_ms: u64) -> String {
        std::thread::sleep(Duration::from_millis(settle_ms));
        let raw = {
            let mut buf = self.output_buf.lock().unwrap();
            let data = buf.clone();
            buf.clear();
            data
        };
        strip_ansi(String::from_utf8_lossy(&raw).to_string())
    }

    /// Read accumulated output WITHOUT clearing the buffer.
    pub fn peek_frame(&self) -> String {
        let buf = self.output_buf.lock().unwrap();
        strip_ansi(String::from_utf8_lossy(&buf).to_string())
    }

    /// Wait until the accumulated output contains the expected string, or timeout.
    pub fn wait_for(&mut self, needle: &str, timeout: Duration) -> Result<String, String> {
        let start = Instant::now();
        loop {
            let content = self.peek_frame();
            if content.contains(needle) {
                // Drain the buffer
                let mut buf = self.output_buf.lock().unwrap();
                let data = buf.clone();
                buf.clear();
                return Ok(strip_ansi(String::from_utf8_lossy(&data).to_string()));
            }
            if start.elapsed() >= timeout {
                return Err(format!(
                    "Timed out after {:?} waiting for {:?}.\nBuffer ({} bytes):\n{}",
                    timeout,
                    needle,
                    content.len(),
                    &content[content.len().saturating_sub(500)..]
                ));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn quit(&mut self) {
        self.send_key("q");
        std::thread::sleep(Duration::from_millis(300));
        let _ = self.child.kill();
    }
}

impl Drop for TuiHarness {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn cargo_bin_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("omniknight");
    path
}

fn key_to_bytes(key: &str) -> Vec<u8> {
    match key {
        "Enter" => vec![13],
        "Esc" => vec![27],
        "Backspace" => vec![127],
        "Tab" => vec![9],
        "Up" => vec![27, 91, 65],
        "Down" => vec![27, 91, 66],
        "Right" => vec![27, 91, 67],
        "Left" => vec![27, 91, 68],
        "Ctrl+c" => vec![3],
        "Ctrl+n" => vec![14],
        "Ctrl+w" => vec![23],
        s if s.len() == 1 => s.as_bytes().to_vec(),
        _ => panic!("Unknown key: {key}"),
    }
}

/// Strip ANSI escape sequences from terminal output.
fn strip_ansi(input: String) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c.is_ascii_alphabetic() || c == '~' || c == '@' {
                        break;
                    }
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '\x07' {
                        break;
                    }
                    if c == '\x1b' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            } else {
                chars.next();
            }
        } else {
            result.push(ch);
        }
    }
    result
}
