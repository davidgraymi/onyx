#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Custom(String), // For user-defined types (structs, enums)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub name: String,
    pub type_info: Type,
    pub bit_field_size: Option<usize>, // e.g., '4' in u8:4
}

// --- Enum Definition ---

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<u64>, // The optional assigned constant value
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    pub name: String,
    pub underlying_type: PrimitiveType, // e.g., u32, u8, etc.
    pub variants: Vec<EnumVariant>,
}

// --- Struct/Message Definitions ---

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MessageDef {
    pub name: String,
    pub fields: Vec<Field>,
}

// --- Top-Level Definitions and Module ---

#[derive(Debug, PartialEq, Clone)]
pub enum Definition {
    Message(MessageDef),
    Struct(StructDef),
    Enum(EnumDef),
}

impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Definition::Message(m) => &m.name,
            Definition::Struct(s) => &s.name,
            Definition::Enum(e) => &e.name,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OnyxModule {
    pub definitions: Vec<Definition>,
}
