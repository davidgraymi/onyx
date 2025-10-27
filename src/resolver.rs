use std::{collections::HashMap, fmt};

use crate::{
    ast::{Definition, MessageDef, OnyxModule, StructDef, Type},
    generators::CompileError,
};

/// The main resolution function that populates the packed_sizes map.
pub fn resolve_module(module: &OnyxModule) -> Result<HashMap<String, usize>, CompileError> {
    // Create a quick lookup map of all definitions
    let def_map: HashMap<&str, &Definition> = module
        .definitions
        .iter()
        .map(|def| (def.name(), def))
        .collect();

    let mut packed_sizes = HashMap::new();
    let mut resolving_types = Vec::new();

    // Iterate through all definitions and ensure their size is resolved,
    // relying on the recursive function to handle dependencies.
    for def in &module.definitions {
        resolve_packed_size_recursive(
            def.name(),
            &def_map,
            &mut packed_sizes,
            &mut resolving_types,
        )?;
    }

    Ok(packed_sizes)
}

/// Recursively calculates the packed size of a type, resolving dependencies as it goes.
fn resolve_packed_size_recursive(
    def_name: &str,
    def_map: &HashMap<&str, &Definition>,
    packed_sizes: &mut HashMap<String, usize>,
    type_stack: &mut Vec<String>,
) -> Result<usize, CompileError> {
    // 1. Check if size is already resolved
    if let Some(size) = packed_sizes.get(def_name) {
        return Ok(*size);
    }

    // 2. Check for circular dependency
    if type_stack.contains(&def_name.to_string()) {
        let cycle = type_stack
            .iter()
            .cloned()
            .chain(std::iter::once(def_name.to_string()))
            .collect::<Vec<String>>()
            .join(" -> ");
        return Err(CompileError(format!(
            "Circular dependency detected: '{cycle}'."
        )));
    }

    // 3. Mark the type as currently being resolved (enter recursion)
    type_stack.push(def_name.to_string());

    // 4. Find definition
    let def = def_map
        .get(def_name)
        .ok_or_else(|| CompileError(format!("Type '{def_name}' is used but not defined.")))?;

    let size_in_bits = match def {
        Definition::Enum(e) => e.underlying_type.get_bit_width(),
        Definition::Struct(s) => {
            get_struct_packed_size_bytes(s, def_map, packed_sizes, type_stack)?
        }
        Definition::Message(m) => {
            get_message_packed_size_bytes(m, def_map, packed_sizes, type_stack)?
        }
    };

    let size_in_bytes = (size_in_bits + 7) / 8; // Ceiling division

    // 5. Unmark and store resolved size (exit recursion)
    type_stack.pop();
    packed_sizes.insert(def_name.to_string(), size_in_bytes);

    Ok(size_in_bytes)
}

/// Calculates the size of a MessageDef by summing up its fields' resolved sizes.
fn get_message_packed_size_bytes(
    m: &MessageDef,
    def_map: &HashMap<&str, &Definition>,
    packed_sizes: &mut HashMap<String, usize>,
    type_stack: &mut Vec<String>,
) -> Result<usize, CompileError> {
    let mut total_size = 0;
    for field in &m.fields {
        let field_size = match field.bit_field_size {
            Some(size) => size,
            None => match &field.type_info {
                Type::Primitive(p) => p.get_bit_width(),
                Type::Custom(custom_name) => {
                    resolve_packed_size_recursive(custom_name, def_map, packed_sizes, type_stack)?
                }
            },
        };
        total_size += field_size;
    }
    Ok(total_size)
}

/// Calculates the size of a MessageDef by summing up its fields' resolved sizes.
fn get_struct_packed_size_bytes(
    s: &StructDef,
    def_map: &HashMap<&str, &Definition>,
    packed_sizes: &mut HashMap<String, usize>,
    type_stack: &mut Vec<String>,
) -> Result<usize, CompileError> {
    let mut total_size = 0;
    for field in &s.fields {
        let field_size = match field.bit_field_size {
            Some(size) => size,
            None => match &field.type_info {
                Type::Primitive(p) => p.get_bit_width(),
                Type::Custom(custom_name) => {
                    resolve_packed_size_recursive(custom_name, def_map, packed_sizes, type_stack)?
                }
            },
        };
        total_size += field_size;
    }
    Ok(total_size)
}

// A simple error type for parsing failures
#[derive(Debug)]
pub struct ResolveError(pub String);
impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Resolve Error: {}", self.0)
    }
}
