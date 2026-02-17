// This file is part of rucat.
//
// rucat is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rucat is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rucat.  If not, see <https://www.gnu.org/licenses/>.
//
// Copyright (C) 2024 Brian 'redbeard' Harrington
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy)]
pub enum ClipboardProvider {
    Native,
    Osc52,
    Osc5522,
}

impl ClipboardProvider {
    /// Automatically detect the best clipboard provider for the current environment.
    #[must_use]
    pub fn auto_detect() -> Option<Self> {
        use std::env;

        // In tests, we can force a provider via an environment variable.
        // This is separate from the CLI flag for easier testing of auto-detection logic.
        #[cfg(test)]
        if let Ok(provider) = env::var("RUCAT_CLIPBOARD_PROVIDER_FOR_TEST") {
            return match provider.as_str() {
                "osc52" => Some(Self::Osc52),
                "osc5522" => Some(Self::Osc5522),
                "native" => Some(Self::Native),
                _ => None,
            };
        }

        // Prioritize terminal-specific protocols over native clipboard
        if env::var("TERM").is_ok_and(|t| t.contains("kitty")) {
            return Some(Self::Osc5522);
        }
        if env::var("TMUX").is_ok() || env::var("SSH_CLIENT").is_ok() {
            return Some(Self::Osc52);
        }

        // Fallback for other modern terminals (like VSCode) that might support OSC 52
        if env::var("TERM_PROGRAM").is_ok() {
            return Some(Self::Osc52);
        }

        // If no special terminal is detected, fall back to the native provider on desktop OSes.
        let is_desktop = {
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            {
                true
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok()
            }
            #[cfg(not(any(unix, windows)))]
            {
                false
            }
        };

        if is_desktop {
            return Some(Self::Native);
        }

        None
    }

    /// Copy content to the clipboard using the specified provider
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the output stream fails, or if the
    /// native clipboard provider fails.
    pub fn copy_to_clipboard(&self, content: &str, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Native => arboard::Clipboard::new().map_or_else(
                |_| {
                    Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "Failed to initialize native clipboard",
                    ))
                },
                |mut clipboard| {
                    clipboard
                        .set_text(content)
                        .map_err(|e| io::Error::other(format!("arboard error: {e}")))
                },
            ),
            Self::Osc52 => {
                let encoded = STANDARD.encode(content.as_bytes());
                write!(w, "\x1b]52;c;{encoded}\x07")
            }
            Self::Osc5522 => {
                let encoded = STANDARD.encode(content.as_bytes());
                write!(w, "\x1b]5522;{encoded}\x07")
            }
        }
    }
}
