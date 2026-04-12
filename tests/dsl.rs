#[path = "harness.rs"]
mod harness;

use std::time::Duration;

use harness::TuiHarness;

/// LLM-friendly DSL for driving Omniknight TUI tests.
pub struct TuiTest {
    harness: TuiHarness,
    accumulated: String,
}

impl TuiTest {
    pub fn launch() -> Self {
        Self::launch_with_size(100, 24)
    }

    pub fn launch_with_size(cols: u16, rows: u16) -> Self {
        let harness = TuiHarness::spawn(cols, rows);
        let mut t = Self {
            harness,
            accumulated: String::new(),
        };
        t.settle(500);
        t
    }

    /// Focus a pane by pressing h/l to cycle.
    pub fn focus_pane(&mut self, pane: &str) {
        // Press 'l' up to 5 times to find the pane
        for _ in 0..5 {
            self.settle(100);
            if self.accumulated.contains(&format!("▸ {pane}")) {
                return;
            }
            self.press("l");
        }
    }

    pub fn press(&mut self, key: &str) {
        self.harness.send_key(key);
        self.settle(150);
    }

    pub fn type_str(&mut self, s: &str) {
        self.harness.send_str(s);
        self.settle(200);
    }

    pub fn settle(&mut self, ms: u64) {
        let frame = self.harness.capture_frame(ms);
        self.accumulated.push_str(&frame);
    }

    pub fn assert_contains(&self, needle: &str) {
        assert!(
            self.accumulated.contains(needle),
            "Expected to find {:?} in terminal output.\nTail:\n{}",
            needle,
            tail_str(&self.accumulated, 1000),
        );
    }

    pub fn assert_not_contains(&self, needle: &str) {
        assert!(
            !self.accumulated.contains(needle),
            "Did NOT expect to find {:?} in terminal output",
            needle,
        );
    }

    pub fn wait_for(&mut self, needle: &str) {
        self.wait_for_timeout(needle, Duration::from_secs(5));
    }

    pub fn wait_for_timeout(&mut self, needle: &str, timeout: Duration) {
        if self.accumulated.contains(needle) {
            return;
        }
        match self.harness.wait_for(needle, timeout) {
            Ok(content) => self.accumulated.push_str(&content),
            Err(msg) => panic!("{msg}"),
        }
    }

    pub fn clear_frame(&mut self) {
        self.accumulated.clear();
    }

    pub fn quit(&mut self) {
        self.harness.quit();
    }
}

fn tail_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut start = s.len() - max_bytes;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    &s[start..]
}
