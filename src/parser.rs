use crate::{ast::{Definition, EnumDef, EnumVariant, Field, MessageDef, OnyxModule, ParseError, PrimitiveType, StructDef, Type}, lexer::{Lexer, Token, TokenKind}};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    /// Creates a new parser and grabs the first token.
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        // Get the first token to start parsing
        let current_token = lexer.next()
            .ok_or_else(|| ParseError("Empty source file.".to_string()))?;

        Ok(Parser { lexer, current_token })
    }

    /// Advances the parser to the next token.
    fn advance(&mut self) {
        // Fetch the next token from the iterator, or use EOF if none is available
        self.current_token = self.lexer.next().unwrap_or(Token {
            kind: TokenKind::Eof,
            span: self.current_token.span, // Use the last known position
        });
    }

    /// Checks if the current token matches an expected kind, consumes it, and advances.
    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind == expected {
            self.advance();
            Ok(())
        } else {
            let msg = format!(
                "Expected {:?}, found {:?} at position {}",
                expected,
                self.current_token.kind,
                self.current_token.span.start
            );
            Err(ParseError(msg))
        }
    }

    // --- Core Parsing Functions ---

    /// Parses the entire Onyx module.
    pub fn parse_module(mut self) -> Result<OnyxModule, ParseError> {
        let mut definitions = Vec::new();

        while self.current_token.kind != TokenKind::Eof {
            // A top-level definition must start with a keyword
            let def = self.parse_definition()?;
            definitions.push(def);
        }

        Ok(OnyxModule { definitions })
    }

    /// Parses a top-level definition: message, struct, or enum.
    fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        match self.current_token.kind {
            TokenKind::Message => self.parse_message(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            _ => {
                let msg = format!("Expected 'message', 'struct', or 'enum', found {:?} at posiiton {}", self.current_token.kind, self.current_token.span.start);
                Err(ParseError(msg))
            }
        }
    }

    // --- Type Parsing ---

    /// Parses a custom or primitive type name.
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let type_name = match &self.current_token.kind {
            TokenKind::Primitive(val) => Type::Primitive(val.clone()),
            TokenKind::Identifier(name) => Type::Custom(name.clone()),
            _ => return Err(ParseError(format!("Expected a type name, found {:?} at position {}", self.current_token.kind, self.current_token.span.start))),
        };
        self.advance();
        Ok(type_name)
    }

    /// Extracts a PrimitiveType from the current token kind (used for enum base type).
    fn parse_primitive_type(&mut self) -> Result<PrimitiveType, ParseError> {
        let primitive_type = match &self.current_token.kind {
            TokenKind::Primitive(val) => val.clone(),
            _ => return Err(ParseError(format!("Expected a numeric primitive type, found {:?} at position {}", self.current_token.kind, self.current_token.span.start))),
        };
        self.advance();
        Ok(primitive_type)
    }

    /// Parses a field definition inside a struct or message.
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.consume_identifier()?;
        let type_info = self.parse_type()?;

        // Optional bit field size
        let bit_field_size: Option<u64> = if self.current_token.kind == TokenKind::Colon {
            self.advance();

            match &type_info {
                Type::Primitive(p) => match self.current_token.kind {
                    TokenKind::LiteralInt(size) if size <= p.get_bit_width() => {
                        self.advance();
                        Some(size)
                    }
                    _ => None
                },
                _ => return Err(ParseError(format!("Expected integer literal for bit-field size, found {:?} at position {}", self.current_token.kind, self.current_token.span.start))),
            }
        } else {
            None
        };

        self.consume(TokenKind::Comma)?;

        Ok(Field { name, type_info, bit_field_size })
    }

    /// Helper to consume an Identifier and return its string value.
    fn consume_identifier(&mut self) -> Result<String, ParseError> {
        let name = match &self.current_token.kind {
            TokenKind::Identifier(id) => id.clone(),
            _ => {
                let msg = format!("Expected an identifier, found {:?} at position {}", self.current_token.kind, self.current_token.span.start);
                return Err(ParseError(msg));
            }
        };
        self.advance();
        Ok(name)
    }

    // --- Message and Struct Parsing ---

    fn parse_struct_body(&mut self) -> Result<Vec<Field>, ParseError> {
        self.consume(TokenKind::OpenBrace)?;
        let mut fields = Vec::new();

        while self.current_token.kind != TokenKind::CloseBrace && self.current_token.kind != TokenKind::Eof {
            fields.push(self.parse_field()?);
        }

        self.consume(TokenKind::CloseBrace)?;
        Ok(fields)
    }

    fn parse_message(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Message)?;
        let name = self.consume_identifier()?;
        let fields = self.parse_struct_body()?;

        Ok(Definition::Message(MessageDef { name, fields }))
    }

    fn parse_struct(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Struct)?;
        let name = self.consume_identifier()?;
        let fields = self.parse_struct_body()?;

        Ok(Definition::Struct(StructDef { name, fields }))
    }
    
    // --- Enum Parsing ---

    fn parse_enum(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Enum)?;
        let name = self.consume_identifier()?;
        
        // Underlying type: 'enum Name: u32'
        self.consume(TokenKind::Colon)?;
        let underlying_type = self.parse_primitive_type()?;

        self.consume(TokenKind::OpenBrace)?;
        
        let mut variants = Vec::new();
        while self.current_token.kind != TokenKind::CloseBrace && self.current_token.kind != TokenKind::Eof {
            let variant_name = self.consume_identifier()?;
            let mut value = None;

            // Optional explicit assignment: '= 10'
            if self.current_token.kind == TokenKind::Assign {
                self.advance();
                
                let literal_value = match self.current_token.kind {
                    TokenKind::LiteralInt(v) => v,
                    _ => return Err(ParseError(format!("Expected integer literal for enum assignment, found {:?} at position {}", self.current_token.kind, self.current_token.span.start))),
                };
                self.advance();
                value = Some(literal_value);
            }
            
            self.consume(TokenKind::Comma)?;

            variants.push(EnumVariant { name: variant_name, value });
        }

        self.consume(TokenKind::CloseBrace)?;

        Ok(Definition::Enum(EnumDef { name, underlying_type, variants }))
    }
}