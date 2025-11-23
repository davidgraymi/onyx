//! # Onyx IDL Compiler Library
//!
//! Onyx is a zero-copy Interface Definition Language (IDL) designed for high-performance,
//! low-latency applications. It allows you to define data structures (messages, structs, enums)
//! in a language-agnostic way and compile them into C++ (and potentially other languages)
//! with zero-copy serialization and deserialization.
//!
//! ## Core Components
//!
//! This library is composed of several modules that work together to compile Onyx source files:
//!
//! - **Lexer** (`lexer`): Tokenizes the input Onyx source code into a stream of tokens.
//! - **Parser** (`parser`): Consumes tokens to build an Abstract Syntax Tree (AST), validating syntax and resolving types.
//! - **AST** (`ast`): Defines the internal representation of the parsed code (Definitions, Fields, Types).
//! - **Generators** (`generators`): Takes the AST and generates code for target languages (e.g., C++).
//!
//! ## Usage
//!
//! The typical compilation flow is:
//! 1.  **Lexing**: `Lexer::new(source)` creates a token stream.
//! 2.  **Parsing**: `Parser::new(source)?.parse_module()?` creates a resolved `OnyxModule`.
//! 3.  **Generation**: A `CodeGenerator` (like `CppGenerator`) takes the `OnyxModule` and produces output files.
//!
//! ## Example
//!
//! ```rust
//! use onyx::parser::Parser;
//! use onyx::generators::cpp::CppGenerator;
//! use onyx::generators::CodeGenerator;
//! use std::path::PathBuf;
//!
//! let source = "
//!     endian = big
//!     message MyMsg {
//!         id u64,
//!     }
//! ";
//!
//! // 1. Parse
//! let parser = Parser::new(source).unwrap();
//! let module = parser.parse_module().unwrap();
//!
//! // 2. Generate
//! let mut generator = CppGenerator::default();
//! generator.add_file_path(PathBuf::from("my_msg.onyx")).unwrap();
//! let outputs = generator.generate(&module).unwrap();
//!
//! for (path, content) in outputs {
//!     println!("Generated file: {:?}", path);
//!     // std::fs::write(path, content).unwrap();
//! }
//! ```

pub mod ast;
pub mod generators;
pub mod lexer;
pub mod parser;

mod color;
