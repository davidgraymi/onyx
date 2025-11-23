use std::{collections::HashMap, fmt};

use crate::{
    ast::{
        Definition, EnumDef, EnumVariant, Field, MessageDef, OnyxModule, PrimitiveType, StructDef,
        Type, WireEndianness,
    },
    color,
    lexer::{Lexer, Token, TokenKind},
};

/// The `Parser` struct is responsible for parsing Onyx source code into an AST (`OnyxModule`).
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    module: OnyxModule,
}

impl<'a> Parser<'a> {
    /// Creates a new parser and grabs the first token.
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source);
        // Get the first token to start parsing
        let current_token = lexer
            .next()
            .ok_or_else(|| ParseError("Empty source file.".to_string()))?;

        Ok(Parser {
            lexer,
            current_token,
            module: OnyxModule::default(),
        })
    }

    /// Advances the parser to the next token.
    fn advance(&mut self) {
        // Fetch the next token from the iterator, or use EOF if none is available
        self.current_token = self.lexer.next().unwrap_or(Token {
            kind: TokenKind::Eof,
            position: self.current_token.position, // Use the last known position
        });
    }

    /// Checks if the current token matches an expected kind, consumes it, and advances.
    /// Returns an error if the current token does not match the expected kind.
    fn consume(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind == expected {
            self.advance();
            Ok(())
        } else {
            let msg = format!(
                "expected {:?}, found {:?} at position {}\n{}",
                expected,
                self.current_token.kind,
                self.current_token.position,
                self.lexer.display_token_in_context(&self.current_token)
            );
            Err(ParseError(msg))
        }
    }

    // --- Core Parsing Functions ---

    /// Parses the entire Onyx module.
    ///
    /// This is the main entry point for the parser. It parses top-level definitions
    /// (endianness, messages, structs, enums) and resolves types and sizes.
    pub fn parse_module(mut self) -> Result<OnyxModule, ParseError> {
        let mut endianness_set = false;

        while self.current_token.kind != TokenKind::Eof {
            // A top-level definition must start with a keyword
            // Check for endian keyword
            if self.current_token.kind == TokenKind::Endianness && !endianness_set {
                self.module.endianness = self.parse_endianness_directive()?;
                endianness_set = true;
            } else if self.current_token.kind == TokenKind::Endianness && endianness_set {
                return Err(ParseError(format!(
                    "expected one endianness definition, found a second at position {}\n{}",
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                )));
            }

            // Check for message, struct, or enum keywords
            let def = self.parse_definition()?;
            if !self.module.definitions.contains_key(def.name()) {
                self.module.definitions.insert(def.name().to_string(), def);
            } else {
                return Err(ParseError(format!(
                    "{} already exists, found second definition at position {}\n{}",
                    def.name(),
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                )));
            }
        }

        self.resolve_module()
    }

    /// Parses the endianness directive (e.g., `endian = big;`).
    fn parse_endianness_directive(&mut self) -> Result<WireEndianness, ParseError> {
        self.consume(TokenKind::Endianness)?;
        self.consume(TokenKind::Assign)?;

        let endianness = match &self.current_token.kind {
            TokenKind::Identifier(s) => match s.as_str() {
                "big" => WireEndianness::Big,
                "little" => WireEndianness::Little,
                _ => {
                    return Err(ParseError(format!(
                        "expected 'big' or 'little' for endianness, found '{}' at position {}\n{}",
                        s,
                        self.current_token.position,
                        self.lexer.display_token_in_context(&self.current_token)
                    )));
                }
            },
            _ => {
                return Err(ParseError(format!(
                    "expected 'big' or 'little' for endianness, found {:?} at position {}\n{}",
                    self.current_token.kind,
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                )));
            }
        };
        self.advance(); // consume Big/Little

        Ok(endianness)
    }

    /// Parses a top-level definition: message, struct, or enum.
    fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        match self.current_token.kind {
            TokenKind::Message => self.parse_message(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            _ => {
                let msg = format!(
                    "expected 'message', 'struct', or 'enum', found {:?} at posiiton {}\n{}",
                    self.current_token.kind,
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                );
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
            _ => {
                return Err(ParseError(format!(
                    "expected a type name, found {:?} at position {}\n{}",
                    self.current_token.kind,
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                )));
            }
        };
        self.advance();
        Ok(type_name)
    }

    /// Extracts a PrimitiveType from the current token kind (used for enum base type).
    fn parse_primitive_type(&mut self) -> Result<PrimitiveType, ParseError> {
        let primitive_type = match &self.current_token.kind {
            TokenKind::Primitive(val) => val.clone(),
            _ => {
                return Err(ParseError(format!(
                    "expected a numeric primitive type, found {:?} at position {}\n{}",
                    self.current_token.kind,
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                )));
            }
        };
        self.advance();
        Ok(primitive_type)
    }

    /// Parses a field definition inside a struct or message.
    /// Handles optional bit-field syntax (e.g., `name type : bits`).
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.consume_identifier()?;
        let type_info = self.parse_type()?;

        // Optional bit field size
        let bit_field_size: Option<usize> = if self.current_token.kind == TokenKind::Colon {
            self.advance();

            match &type_info {
                Type::Primitive(p) => match self.current_token.kind {
                    TokenKind::LiteralInt(size) => {
                        let max_bit_width = p
                            .get_bit_width()
                            .try_into()
                            .map_err(|e| ParseError(format!("enexpected: {e}")))?;

                        if size <= max_bit_width {
                            self.advance();
                            Some(size as usize)
                        } else {
                            return Err(ParseError(format!(
                                "bit-field size {size} exceeds type {:?}'s width of {max_bit_width} bits at position {}\n{}",
                                p,
                                self.current_token.position,
                                self.lexer.display_token_in_context(&self.current_token)
                            )));
                        }
                    }
                    _ => None,
                },
                _ => {
                    return Err(ParseError(format!(
                        "expected integer literal for bit-field size, found {:?} at position {}\n{}",
                        self.current_token.kind,
                        self.current_token.position,
                        self.lexer.display_token_in_context(&self.current_token)
                    )));
                }
            }
        } else {
            None
        };

        self.consume(TokenKind::Comma)?;

        Ok(Field {
            name,
            type_info,
            bit_field_size,
        })
    }

    /// Helper to consume an Identifier and return its string value.
    fn consume_identifier(&mut self) -> Result<String, ParseError> {
        let name = match &self.current_token.kind {
            TokenKind::Identifier(id) => id.clone(),
            _ => {
                let msg = format!(
                    "Expected an identifier, found {:?} at position {}\n{}",
                    self.current_token.kind,
                    self.current_token.position,
                    self.lexer.display_token_in_context(&self.current_token)
                );
                return Err(ParseError(msg));
            }
        };
        self.advance();
        Ok(name)
    }

    // --- Message and Struct Parsing ---

    /// Parses the body of a struct or message (the fields inside braces).
    fn parse_struct_body(&mut self) -> Result<Vec<Field>, ParseError> {
        self.consume(TokenKind::OpenBrace)?;
        let mut fields = Vec::new();

        while self.current_token.kind != TokenKind::CloseBrace
            && self.current_token.kind != TokenKind::Eof
        {
            fields.push(self.parse_field()?);
        }

        self.consume(TokenKind::CloseBrace)?;
        Ok(fields)
    }

    /// Parses a message definition.
    fn parse_message(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Message)?;
        let name = self.consume_identifier()?;
        let fields = self.parse_struct_body()?;

        Ok(Definition::Message(MessageDef {
            name,
            fields,
            size: None,
        }))
    }

    /// Parses a struct definition.
    fn parse_struct(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Struct)?;
        let name = self.consume_identifier()?;
        let fields = self.parse_struct_body()?;

        Ok(Definition::Struct(StructDef {
            name,
            fields,
            size: None,
        }))
    }

    // --- Enum Parsing ---

    /// Parses an enum definition.
    fn parse_enum(&mut self) -> Result<Definition, ParseError> {
        self.consume(TokenKind::Enum)?;
        let name = self.consume_identifier()?;

        // Underlying type: 'enum Name: u32'
        self.consume(TokenKind::Colon)?;
        let underlying_type = self.parse_primitive_type()?;

        self.consume(TokenKind::OpenBrace)?;

        let mut variants = Vec::new();
        while self.current_token.kind != TokenKind::CloseBrace
            && self.current_token.kind != TokenKind::Eof
        {
            let variant_name = self.consume_identifier()?;
            let mut value = None;

            // Optional explicit assignment: '= 10'
            if self.current_token.kind == TokenKind::Assign {
                self.advance();

                let literal_value = match self.current_token.kind {
                    TokenKind::LiteralInt(v) => v,
                    _ => {
                        return Err(ParseError(format!(
                            "expected integer literal for enum assignment, found {:?} at position {}\n{}",
                            self.current_token.kind,
                            self.current_token.position,
                            self.lexer.display_token_in_context(&self.current_token)
                        )));
                    }
                };
                self.advance();
                value = Some(literal_value);
            }

            self.consume(TokenKind::Comma)?;

            variants.push(EnumVariant {
                name: variant_name,
                value,
            });
        }

        self.consume(TokenKind::CloseBrace)?;

        Ok(Definition::Enum(EnumDef {
            name,
            underlying_type,
            variants,
        }))
    }

    // ------ Core resolving logic ------

    /// Resolves the module by calculating sizes and determining definition order.
    /// Handles circular dependency detection.
    fn resolve_module(mut self) -> Result<OnyxModule, ParseError> {
        let mut type_order: Vec<String> = Vec::new();
        let mut type_stack: Vec<String> = Vec::new();

        // Use a standard `for` loop over mutable values instead of iteration helper
        // to avoid complex lifetime issues and allow using 'id' in the error message.
        // We will store the calculated size in a temporary map.
        let mut calculated_sizes = HashMap::new();

        // Pass 1: Calculate sizes (iterate IMMUTABLY)
        for (id, def) in &self.module.definitions {
            // NOTE: The `resolve_type` must be able to calculate and return the size
            // without needing to mutate the definition, otherwise we run into the
            // same borrow issue. It looks like resolve_type is *not* supposed to
            // mutate, but rather calculate recursively.

            // The original logic required calculating size recursively and *then* // assigning it. Let's separate the calculation from the assignment.

            // 1. Clear type_stack for each top-level definition resolution
            type_stack.clear();

            // 2. Call the original resolve_type (which now just *calculates*)
            let size = self.resolve_type_calculate(&mut type_order, &mut type_stack, def)?;
            calculated_sizes.insert(id.clone(), size);
        }

        // Pass 2: Assign sizes (iterate MUTABLY)
        for (id, def) in self.module.definitions.iter_mut() {
            let size = calculated_sizes[id]; // Safe to unwrap since we just calculated them
            match def {
                Definition::Message(message_def) => message_def.size = Some(size),
                Definition::Struct(struct_def) => struct_def.size = Some(size),
                Definition::Enum(_) => {}
            }
        }

        self.module.order = type_order;
        Ok(self.module)
    }

    /// Recursively calculates the size of a definition and populates `type_order`.
    /// Detects circular dependencies using `type_stack`.
    fn resolve_type_calculate(
        &self,
        type_order: &mut Vec<String>,
        type_stack: &mut Vec<String>,
        def: &Definition,
    ) -> Result<usize, ParseError> {
        match def.size() {
            Some(size) => {
                if !type_order.contains(&def.name().to_string()) {
                    type_order.push(def.name().to_string());
                }
                Ok(size)
            }
            None => {
                if type_stack.contains(&def.name().to_string()) {
                    let cycle = type_stack
                        .iter()
                        .cloned()
                        .chain(std::iter::once(def.name().to_string()))
                        .collect::<Vec<String>>()
                        .join(" -> ");
                    return Err(ParseError(format!(
                        "circular dependency detected: '{cycle}'."
                    )));
                }

                type_stack.push(def.name().to_string());

                let calculated_size = match def {
                    Definition::Struct(s) => {
                        self.resolve_fields_calculate(type_order, type_stack, &s.fields)
                    }
                    Definition::Message(m) => {
                        self.resolve_fields_calculate(type_order, type_stack, &m.fields)
                    }
                    // Enums are resolved during field resolution, not here
                    _ => Err(ParseError("unexpected error!".to_string())),
                }?;

                let deepest_dep = match type_stack.pop() {
                    Some(id) => id,
                    None => {
                        return Err(ParseError(
                            "expected a custom type to be on the stack.".to_string(),
                        ));
                    }
                };
                if !type_order.contains(&deepest_dep) {
                    type_order.push(deepest_dep);
                }

                Ok(calculated_size)
            }
        }
    }

    /// Calculates the total size of a list of fields.
    fn resolve_fields_calculate(
        &self,
        type_order: &mut Vec<String>,
        type_stack: &mut Vec<String>,
        fields: &Vec<Field>,
    ) -> Result<usize, ParseError> {
        let mut total_size = 0;
        for field in fields {
            let field_size = match field.bit_field_size {
                Some(size) => size,
                None => match &field.type_info {
                    Type::Primitive(p) => p.get_bit_width(),
                    Type::Custom(custom_name) => {
                        if let Some(target_def) = self.module.definitions.get(custom_name) {
                            // Recursively call type resolution to understand circular dependencies
                            self.resolve_type_calculate(type_order, type_stack, target_def)?
                        } else {
                            // You had an incomplete error message here
                            return Err(ParseError(format!(
                                "custom type '{custom_name}' not defined"
                            )));
                        }
                    }
                },
            };
            total_size += field_size;
        }
        Ok(total_size)
    }
}

/// A simple error type for parsing failures.
#[derive(Debug)]
pub struct ParseError(pub String);
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}error{}: {}", color::RED, color::END, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_struct() {
        let source = "struct MyStruct { field1 u8, field2 u32, }";
        let parser = Parser::new(source).unwrap();
        let module = parser.parse_module().unwrap();

        assert!(module.definitions.contains_key("MyStruct"));
        if let Definition::Struct(s) = &module.definitions["MyStruct"] {
            assert_eq!(s.name, "MyStruct");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name, "field1");
            assert_eq!(s.fields[1].name, "field2");
            assert_eq!(s.size, Some(40)); // 8 + 32 = 40 bits
        } else {
            panic!("Expected Struct definition");
        }
    }

    #[test]
    fn test_parse_enum() {
        let source = "enum MyEnum : u8 { A = 1, B, }";
        let parser = Parser::new(source).unwrap();
        let module = parser.parse_module().unwrap();

        assert!(module.definitions.contains_key("MyEnum"));
        if let Definition::Enum(e) = &module.definitions["MyEnum"] {
            assert_eq!(e.name, "MyEnum");
            assert_eq!(e.underlying_type, PrimitiveType::U8);
            assert_eq!(e.variants.len(), 2);
            assert_eq!(e.variants[0].name, "A");
            assert_eq!(e.variants[0].value, Some(1));
            assert_eq!(e.variants[1].name, "B");
            assert_eq!(e.variants[1].value, None);
        } else {
            panic!("Expected Enum definition");
        }
    }

    #[test]
    fn test_parse_message() {
        let source = "message MyMsg { id u64, }";
        let parser = Parser::new(source).unwrap();
        let module = parser.parse_module().unwrap();

        assert!(module.definitions.contains_key("MyMsg"));
        if let Definition::Message(m) = &module.definitions["MyMsg"] {
            assert_eq!(m.name, "MyMsg");
            assert_eq!(m.fields.len(), 1);
            assert_eq!(m.size, Some(64));
        } else {
            panic!("Expected Message definition");
        }
    }

    #[test]
    fn test_endianness() {
        let source = "endian = big struct S { f u8, }";
        let parser = Parser::new(source).unwrap();
        let module = parser.parse_module().unwrap();

        assert_eq!(module.endianness, WireEndianness::Big);
    }

    #[test]
    fn test_circular_dependency() {
        let source = "struct A { b B, } struct B { a A, }";
        let parser = Parser::new(source).unwrap();
        let result = parser.parse_module();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.0.contains("circular dependency detected"));
    }

    #[test]
    fn test_duplicate_definition() {
        let source = "struct A { f u8, } struct A { f u8, }";
        let parser = Parser::new(source).unwrap();
        let result = parser.parse_module();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.0.contains("already exists"));
    }

    #[test]
    fn test_bit_field_too_large() {
        let source = "struct A { f u8 : 9, }";
        let parser = Parser::new(source).unwrap();
        let result = parser.parse_module();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.0.contains("exceeds type"));
    }
}
