use std::io::{self, Write};
use std::path::Path;

pub trait Formatter {
    fn write(&self, path: &Path, content: &str, w: &mut dyn Write) -> io::Result<()>;
}

pub mod ansi;
pub mod xml;
pub mod markdown;
