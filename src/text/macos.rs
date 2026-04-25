//! macOS text injection via native CGEvent Unicode input (enigo crate).
//!
//! Uses enigo's `text()` which calls CGEventSetUnicodeString on macOS,
//! directly typing characters into the focused text field — no clipboard involved.
//! Requires Accessibility permission in System Settings.

use crate::{TextChangeEvent, TextObserver, TextOutput, TextSnapshot};
use anyhow::Result;
use log::info;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// Flag to suppress rdev events while we're injecting text (avoid echo).
static INJECTING: AtomicBool = AtomicBool::new(false);

/// Returns true while text injection is in progress (rdev listener should ignore events).
pub fn is_injecting() -> bool {
    INJECTING.load(Ordering::SeqCst)
}

pub struct MacDirectOutput {
    last_injected: Mutex<String>,
}

unsafe impl Send for MacDirectOutput {}
unsafe impl Sync for MacDirectOutput {}

impl MacDirectOutput {
    pub fn new() -> Result<Self> {
        Ok(Self {
            last_injected: Mutex::new(String::new()),
        })
    }

    fn inject_text(&self, text: &str) -> Result<()> {
        INJECTING.store(true, Ordering::SeqCst);

        let mut enigo = enigo::Enigo::new(&enigo::Settings::default())
            .map_err(|e| anyhow::anyhow!("Failed to create Enigo: {}", e))?;

        // enigo::text() on macOS uses CGEventSetUnicodeString for direct Unicode input.
        use enigo::Keyboard;
        enigo
            .text(text)
            .map_err(|e| anyhow::anyhow!("Failed to inject text: {}", e))?;

        INJECTING.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl TextOutput for MacDirectOutput {
    fn commit_text(&self, text: &str) -> Result<()> {
        self.inject_text(text)?;
        info!("MacDirectOutput: {} chars injected", text.len());
        *self.last_injected.lock() = text.to_string();
        Ok(())
    }

    fn method_name(&self) -> &str {
        "macos_direct"
    }
}

pub struct NopTextObserver;

impl NopTextObserver {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl TextObserver for NopTextObserver {
    fn start_monitoring(&mut self) -> Result<()> {
        Ok(())
    }
    fn stop_monitoring(&mut self) -> Result<()> {
        Ok(())
    }
    fn poll_changes(&self) -> Vec<TextChangeEvent> {
        Vec::new()
    }
    fn snapshot(&self) -> Result<TextSnapshot> {
        Ok(TextSnapshot {
            full_text: String::new(),
            cursor_position: 0,
            selection_start: 0,
            selection_end: 0,
        })
    }
}

pub fn create_platform_session() -> Result<super::PlatformTextSession> {
    let output = MacDirectOutput::new()?;
    let observer = NopTextObserver::new()?;
    Ok(super::PlatformTextSession::new(
        Box::new(output),
        Box::new(observer),
    ))
}
