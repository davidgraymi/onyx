use std::{fmt::Display, iter::Peekable, str::Chars};

use crate::{ast::PrimitiveType, color};

/// A minimal struct to track location in the source file for better errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Tracks token location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub span: Span,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.span.start + 1)
    }
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
    pub position: Position,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    start_line_indices: Vec<usize>,
    absolute_pos: usize,
    current_line: usize,
    current_col: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer from the input source string.
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            chars: source.chars().peekable(),
            start_line_indices: vec![0],
            absolute_pos: 0,
            current_line: 0,
            current_col: 0,
        }
    }

    pub fn display_token_in_context(&self, token: &Token) -> String {
        let left_index = if token.position.line > 3 {
            self.start_line_indices[token.position.line - 4]
        } else {
            0
        };
        let right_index = self.start_line_indices[token.position.line] + token.position.span.end;

        let mut result = self.source[left_index..right_index].to_string();
        let token_size = token.position.span.end - token.position.span.start;
        let point_str = '^'.to_string().repeat(token_size);
        let space_str = ' '.to_string().repeat(token.position.span.start);

        let right_slice = &self.source[right_index..];
        for (index, slice) in right_slice.lines().enumerate() {
            if index == 0 {
                result.push_str(slice);
                result.push_str(&format!(
                    "\n{space_str}{}{point_str}{}",
                    color::RED,
                    color::END
                ));
            } else if index <= 4 {
                result.push('\n');
                result.push_str(slice);
            } else {
                break;
            }
        }

        result
    }

    /// Advances the internal position and consumes the current character.
    fn advance(&mut self) -> Option<char> {
        self.absolute_pos += 1;
        self.current_col += 1;
        self.chars.next()
    }

    /// Advances the internal position and consumes the current character.
    fn advance_new_line(&mut self) -> Option<char> {
        self.current_col = 0;
        self.absolute_pos += 1;
        self.current_line += 1;
        self.start_line_indices.push(self.absolute_pos);
        self.chars.next()
    }

    /// Peeks at the next character without consuming it.
    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    /// Skips all whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c == '\n' {
                self.advance_new_line();
            } else if c.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Parses an identifier or keyword.
    fn take_identifier(&mut self) -> TokenKind {
        let start = self.absolute_pos;
        while let Some(&c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.absolute_pos;
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
        let start = self.absolute_pos;
        while let Some(&c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.absolute_pos;
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

        let start_pos = self.absolute_pos;
        let start_col = self.current_col;
        let kind = match self.peek() {
            Some('{') => {
                self.advance();
                TokenKind::OpenBrace
            }
            Some('}') => {
                self.advance();
                TokenKind::CloseBrace
            }
            Some(',') => {
                self.advance();
                TokenKind::Comma
            }
            Some(':') => {
                self.advance();
                TokenKind::Colon
            }
            Some(';') => {
                self.advance();
                TokenKind::Semicolon
            }
            Some('=') => {
                self.advance();
                TokenKind::Assign
            }

            // Handle identifiers/keywords
            Some(c) if c.is_ascii_alphabetic() || *c == '_' => self.take_identifier(),

            // Handle numbers
            Some(c) if c.is_ascii_digit() => self.take_number(),

            // End of file
            None => {
                // Return Eof token once, then None on subsequent calls
                if start_pos < self.source.len() {
                    // Should only happen if there was trailing whitespace
                    return Some(Token {
                        kind: TokenKind::Eof,
                        position: Position {
                            line: self.current_line,
                            span: Span {
                                start: start_pos,
                                end: start_pos,
                            },
                        },
                    });
                }
                return None;
            }

            // Error token for unrecognized characters
            Some(c) => TokenKind::Error(format!("Unrecognized character: '{c}'")),
        };

        let end_col = self.current_col;

        Some(Token {
            kind,
            position: Position {
                line: self.current_line,
                span: Span {
                    start: start_col,
                    end: end_col,
                },
            },
        })
    }
}
