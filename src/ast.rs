use std::collections::HashMap;

/// Represents the primitive types supported by Onyx.
#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    /// Boolean type (1 byte).
    Bool,
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// 32-bit floating point number.
    F32,
    /// 64-bit floating point number.
    F64,
}

impl PrimitiveType {
    /// Gets the size in bits of a primitive type.
    pub fn get_bit_width(&self) -> usize {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::I8 => 8,
            PrimitiveType::U16 | PrimitiveType::I16 => 16,
            PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32 => 32,
            PrimitiveType::U64 | PrimitiveType::I64 | PrimitiveType::F64 => 64,
        }
    }

    /// Gets the size in bytes of a primitive type.
    pub fn get_byte_size(&self) -> usize {
        match self {
            PrimitiveType::Bool | PrimitiveType::U8 | PrimitiveType::I8 => 1,
            PrimitiveType::U16 | PrimitiveType::I16 => 2,
            PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32 => 4,
            PrimitiveType::U64 | PrimitiveType::I64 | PrimitiveType::F64 => 8,
        }
    }
}

/// Represents a type in Onyx, which can be a primitive or a custom user-defined type.
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// A built-in primitive type.
    Primitive(PrimitiveType),
    /// A user-defined type (struct or enum), identified by its name.
    Custom(String), // For user-defined types (structs, enums)
}

/// Represents a field within a struct or message.
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    /// The name of the field.
    pub name: String,
    /// The type of the field.
    pub type_info: Type,
    /// Optional size for bit fields. If present, specifies the width in bits.
    pub bit_field_size: Option<usize>,
}

impl Field {
    /// Calculates the bit width of the field.
    ///
    /// If `bit_field_size` is set, it returns that value.
    /// Otherwise, it looks up the size based on the type info.
    /// For custom types, it queries the provided `module` to find the definition and its size.
    pub fn get_bit_width(&self, module: &OnyxModule) -> usize {
        match self.bit_field_size {
            Some(x) => x,
            None => match &self.type_info {
                Type::Primitive(primitive_type) => primitive_type.get_bit_width(),
                Type::Custom(s) => module.definitions.get(s).unwrap().size().unwrap(),
            },
        }
    }
}

// --- Enum Definition ---

/// Represents a variant within an enum.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    /// The name of the variant.
    pub name: String,
    /// The optional assigned constant value for the variant.
    pub value: Option<u64>,
}

/// Represents an enum definition.
#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    /// The name of the enum.
    pub name: String,
    /// The underlying primitive integer type for the enum.
    pub underlying_type: PrimitiveType,
    /// The list of variants in the enum.
    pub variants: Vec<EnumVariant>,
}

// --- Struct/Message Definitions ---

/// Represents a struct definition.
#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    /// The name of the struct.
    pub name: String,
    /// The fields contained in the struct.
    pub fields: Vec<Field>,
    /// Optional explicit size for the struct in bytes.
    pub size: Option<usize>,
}

/// Represents a message definition.
#[derive(Debug, PartialEq, Clone)]
pub struct MessageDef {
    /// The name of the message.
    pub name: String,
    /// The fields contained in the message.
    pub fields: Vec<Field>,
    /// Optional explicit size for the message in bytes.
    pub size: Option<usize>,
}

// --- Top-Level Definitions and Module ---

/// Represents a top-level definition in an Onyx module.
#[derive(Debug, PartialEq, Clone)]
pub enum Definition {
    /// A message definition.
    Message(MessageDef),
    /// A struct definition.
    Struct(StructDef),
    /// An enum definition.
    Enum(EnumDef),
}

impl Definition {
    /// Returns the name of the definition.
    pub fn name(&self) -> &str {
        match self {
            Definition::Message(m) => &m.name,
            Definition::Struct(s) => &s.name,
            Definition::Enum(e) => &e.name,
        }
    }

    /// Returns the size of the definition in bits, if available.
    ///
    /// For enums, it returns the bit width of the underlying type.
    /// For structs and messages, it returns the explicit size if set.
    pub fn size(&self) -> Option<usize> {
        match self {
            Definition::Message(message_def) => message_def.size,
            Definition::Struct(struct_def) => struct_def.size,
            Definition::Enum(enum_def) => Some(enum_def.underlying_type.get_bit_width()),
        }
    }
}

/// Specifies the endianness for wire transmission.
#[derive(Debug, Default, PartialEq, Clone)]
pub enum WireEndianness {
    /// Little-endian byte order.
    #[default]
    Little,
    /// Big-endian byte order.
    Big,
}

/// Represents a parsed Onyx module containing definitions.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct OnyxModule {
    /// A map of definitions by name.
    pub definitions: HashMap<String, Definition>,
    /// The endianness used for this module.
    pub endianness: WireEndianness,
    /// The order in which definitions appeared in the source.
    pub order: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_sizes() {
        assert_eq!(PrimitiveType::Bool.get_bit_width(), 8);
        assert_eq!(PrimitiveType::U8.get_bit_width(), 8);
        assert_eq!(PrimitiveType::U32.get_bit_width(), 32);
        assert_eq!(PrimitiveType::F64.get_bit_width(), 64);

        assert_eq!(PrimitiveType::Bool.get_byte_size(), 1);
        assert_eq!(PrimitiveType::U16.get_byte_size(), 2);
        assert_eq!(PrimitiveType::I32.get_byte_size(), 4);
        assert_eq!(PrimitiveType::F64.get_byte_size(), 8);
    }

    #[test]
    fn test_field_bit_width_primitive() {
        let module = OnyxModule::default();
        let field = Field {
            name: "test".to_string(),
            type_info: Type::Primitive(PrimitiveType::U32),
            bit_field_size: None,
        };
        assert_eq!(field.get_bit_width(&module), 32);
    }

    #[test]
    fn test_field_bit_width_bitfield() {
        let module = OnyxModule::default();
        let field = Field {
            name: "test".to_string(),
            type_info: Type::Primitive(PrimitiveType::U32),
            bit_field_size: Some(12),
        };
        assert_eq!(field.get_bit_width(&module), 12);
    }

    #[test]
    fn test_field_bit_width_custom() {
        let mut module = OnyxModule::default();
        let enum_def = EnumDef {
            name: "MyEnum".to_string(),
            underlying_type: PrimitiveType::U16,
            variants: vec![],
        };
        module
            .definitions
            .insert("MyEnum".to_string(), Definition::Enum(enum_def));

        let field = Field {
            name: "test".to_string(),
            type_info: Type::Custom("MyEnum".to_string()),
            bit_field_size: None,
        };

        // Enum underlying type is U16 (2 bytes), so size is 2 * 8 = 16 bits
        assert_eq!(field.get_bit_width(&module), 16);
    }

    #[test]
    fn test_definition_helpers() {
        let msg_def = MessageDef {
            name: "MyMsg".to_string(),
            fields: vec![],
            size: Some(100),
        };
        let def = Definition::Message(msg_def);

        assert_eq!(def.name(), "MyMsg");
        assert_eq!(def.size(), Some(100));

        let enum_def = EnumDef {
            name: "MyEnum".to_string(),
            underlying_type: PrimitiveType::U8,
            variants: vec![],
        };
        let def_enum = Definition::Enum(enum_def);
        assert_eq!(def_enum.size(), Some(8)); // 1 byte = 8 bits
    }
}
