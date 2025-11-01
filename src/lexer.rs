use std::{iter::Peekable, str::Chars};

use crate::ast::PrimitiveType;

// A minimal struct to track location in the source file for better errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

// The core token definition for the Onyx IDL.
#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // Keywords
    Endianness,
    Import,
    Message,
    Struct,
    Enum,
    // Primitive Types
    Primitive(PrimitiveType),
    // Delimiters and Operators
    OpenBrace,  // {
    CloseBrace, // }
    Comma,      // ,
    Colon,      // :
    Semicolon,  // ;
    Assign,     // =
    /// Custom type identifier that assigns an id to something like a message or struct.
    /// my_field MyStructName
    Identifier(String),
    /// Integer literal (i.e. 123)
    LiteralInt(u64),
    /// End of File
    Eof,
    /// Error token with a message
    Error(String),
}

// A full token, including its kind and its location (span).
#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current_pos: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer from the input source string.
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.chars().peekable(),
            current_pos: 0,
        }
    }

    /// Advances the internal position and consumes the current character.
    fn advance(&mut self) -> Option<char> {
        self.current_pos += 1;
        self.chars.next()
    }

    /// Peeks at the next character without consuming it.
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Skips all whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parses an identifier or keyword.
    fn take_identifier(&mut self) -> TokenKind {
        let start = self.current_pos;
        while let Some(&c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.current_pos;
        let ident_str = &self.source[start..end];

        // Check if it's a reserved keyword or type
        match ident_str {
            "import" => TokenKind::Import,
            "endian" => TokenKind::Endianness,
            "message" => TokenKind::Message,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "bool" => TokenKind::Primitive(PrimitiveType::Bool),
            "u8" => TokenKind::Primitive(PrimitiveType::U8),
            "u16" => TokenKind::Primitive(PrimitiveType::U16),
            "u32" => TokenKind::Primitive(PrimitiveType::U32),
            "u64" => TokenKind::Primitive(PrimitiveType::U64),
            "i8" => TokenKind::Primitive(PrimitiveType::I8),
            "i16" => TokenKind::Primitive(PrimitiveType::I16),
            "i32" => TokenKind::Primitive(PrimitiveType::I32),
            "i64" => TokenKind::Primitive(PrimitiveType::I64),
            "f32" => TokenKind::Primitive(PrimitiveType::F32),
            "f64" => TokenKind::Primitive(PrimitiveType::F64),
            _ => TokenKind::Identifier(ident_str.to_string()),
        }
    }

    /// Parses an integer literal.
    fn take_number(&mut self) -> TokenKind {
        let start = self.current_pos;
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.current_pos;
        let num_str = &self.source[start..end];

        // Safely parse the number; use unwrap_or_else for a clean error token if parse fails.
        // Since we only checked for digits, this parse should typically succeed unless overflow occurs.
        match num_str.parse::<u64>() {
            Ok(val) => TokenKind::LiteralInt(val),
            Err(_) => TokenKind::Error(format!("Invalid or oversized integer literal: {num_str}")),
        }
    }
}

impl<'a> From<&'a str> for Lexer<'a> {
    fn from(source: &'a str) -> Self {
        Lexer::new(source)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // 1. Skip whitespace before processing the next token
        self.skip_whitespace();

        let start_pos = self.current_pos;
        let kind = match self.advance() {
            Some('{') => TokenKind::OpenBrace,
            Some('}') => TokenKind::CloseBrace,
            Some(',') => TokenKind::Comma,
            Some(':') => TokenKind::Colon,
            Some(';') => TokenKind::Semicolon,
            Some('=') => TokenKind::Assign,

            // Handle identifiers/keywords
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                // Must rewind one step because 'advance()' was called in the match
                self.current_pos -= c.len_utf8();
                self.chars = self.source[self.current_pos..].chars().peekable();
                self.take_identifier()
            }

            // Handle numbers
            Some(c) if c.is_ascii_digit() => {
                // Rewind one step
                self.current_pos -= c.len_utf8();
                self.chars = self.source[self.current_pos..].chars().peekable();
                self.take_number()
            }

            // End of file
            None => {
                // Return Eof token once, then None on subsequent calls
                if start_pos < self.source.len() {
                    // Should only happen if there was trailing whitespace
                    return Some(Token {
                        kind: TokenKind::Eof,
                        span: Span {
                            start: start_pos,
                            end: start_pos,
                        },
                    });
                }
                return None;
            }

            // Error token for unrecognized characters
            Some(c) => TokenKind::Error(format!("Unrecognized character: '{c}'")),
        };

        let end_pos = self.current_pos;

        Some(Token {
            kind,
            span: Span {
                start: start_pos,
                end: end_pos,
            },
        })
    }
}
