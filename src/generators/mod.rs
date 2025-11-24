use std::{error::Error, fmt, path::PathBuf};

use crate::ast::OnyxModule;

pub mod cpp;
pub mod py;
pub mod rust;

/// A trait for code generators that translate the Onyx AST into target language code.
pub trait CodeGenerator {
    /// Translates the AST module into a final, runnable code string.
    ///
    /// Returns a vector of (filename, content) tuples, allowing a single module
    /// to generate multiple files (e.g., header and source).
    fn generate(&mut self, module: &OnyxModule) -> Result<Vec<(PathBuf, String)>, CompileError>;
}

/// Minimal error type for compilation/generation failures.
#[derive(Debug)]
pub struct CompileError(pub String);

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Compile Error: {}", self.0)
    }
}

impl Error for CompileError {}
