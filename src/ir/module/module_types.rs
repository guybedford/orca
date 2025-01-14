//!  Intermediate representation of Module Types

use crate::ir::id::TypeID;
use crate::DataType;
use std::collections::HashMap;
use std::hash::Hash;

/// Orca's representation of function types, shortened from [Walrus' Representation].
///
/// [Walrus' Representation]: https://docs.rs/walrus/latest/walrus/struct.Type.html
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FuncType {
    pub params: Box<[DataType]>,
    pub results: Box<[DataType]>,
}
impl FuncType {
    /// Create a new Function Type
    pub fn new(params: Box<[DataType]>, results: Box<[DataType]>) -> Self {
        Self { params, results }
    }
}

/// The Module Types Section
#[derive(Clone, Debug, Default)]
pub struct ModuleTypes {
    pub types: Vec<FuncType>,
    /// This enables us to quickly do a lookup to determine if a type has already been added
    pub types_map: HashMap<FuncType, TypeID>,
}

impl ModuleTypes {
    /// Create a new Module Types section
    pub fn new(types: Vec<FuncType>) -> Self {
        let mut types_map = HashMap::default();
        for (id, ty) in types.iter().enumerate() {
            types_map.insert(ty.clone(), TypeID(id as u32));
        }
        ModuleTypes { types, types_map }
    }

    /// Check if there are any types in this module
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Add a new type to the module, returns the index of the new type.
    pub fn add(&mut self, param: &[DataType], ret: &[DataType]) -> TypeID {
        let index = self.types.len();
        let ty = FuncType::new(
            param.to_vec().into_boxed_slice(),
            ret.to_vec().into_boxed_slice(),
        );

        if !self.types_map.contains_key(&ty) {
            self.types.push(ty.clone());
        }
        *self
            .types_map
            .entry(ty.clone())
            .or_insert(TypeID(index as u32))
    }

    /// Number of types in this module
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Create an iterable over the Type Section
    pub fn iter(&self) -> std::slice::Iter<'_, FuncType> {
        self.types.iter()
    }

    /// Get type from index of the type section
    pub fn get(&self, index: TypeID) -> Option<&FuncType> {
        self.types.get(*index as usize)
    }
}
