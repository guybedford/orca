//! Intermediate representation of sections in a wasm module.

use crate::error::Error;
use crate::ir::id::{CustomSectionID, FunctionID, GlobalID, ModuleID, TypeID};
use std::cmp::PartialEq;
use std::fmt::Formatter;
use std::fmt::{self};
use std::mem::discriminant;
use std::slice::Iter;
use wasm_encoder::reencode::Reencode;
use wasm_encoder::AbstractHeapType;
use wasmparser::{ConstExpr, Operator, RefType, ValType};

type Result<T> = std::result::Result<T, Error>;

/// Orca's Datatype. Combination of multiple [`wasmparser`] datatypes.
///
/// [ValType]: https://docs.rs/wasmparser/latest/wasmparser/enum.ValType.html
#[derive(Debug, Clone, Eq, Hash, PartialEq, Copy)]
pub enum DataType {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
    Any,
    None,
    NoExtern,
    NoFunc,
    Eq,
    Struct,
    Array,
    I31,
    Exn,
    NoExn,
    Module(ModuleID),
    RecGroup(u32),
    CoreTypeId(u32), // TODO: Look at this
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DataType::I32 => write!(f, "DataType: I32"),
            DataType::I64 => write!(f, "DataType: I64"),
            DataType::F32 => write!(f, "DataType: F32"),
            DataType::F64 => write!(f, "DataType: F64"),
            DataType::V128 => write!(f, "DataType: V128"),
            DataType::FuncRef => write!(f, "DataType: FuncRef"),
            DataType::ExternRef => write!(f, "DataType: ExternRef"),
            DataType::Any => write!(f, "DataType: Any"),
            DataType::None => write!(f, "DataType: None"),
            DataType::NoExtern => write!(f, "DataType: NoExtern"),
            DataType::NoFunc => write!(f, "DataType: NoFunc"),
            DataType::Eq => write!(f, "DataType: Eq"),
            DataType::Struct => write!(f, "DataType: Struct"),
            DataType::Array => write!(f, "DataType: Array"),
            DataType::I31 => write!(f, "DataType: I31"),
            DataType::Exn => write!(f, "DataType: Exn"),
            DataType::NoExn => write!(f, "DataType: NoExn"),
            DataType::Module(idx) => write!(f, "DataType: Module {:?}", idx),
            DataType::RecGroup(idx) => write!(f, "DataType: RecGroup {:?}", idx),
            DataType::CoreTypeId(idx) => write!(f, "DataType: CoreTypeId {:?}", idx),
        }
    }
}

impl From<ValType> for DataType {
    fn from(value: ValType) -> Self {
        match value {
            ValType::I32 => DataType::I32,
            ValType::I64 => DataType::I64,
            ValType::F32 => DataType::F32,
            ValType::F64 => DataType::F64,
            ValType::V128 => DataType::V128,
            ValType::Ref(ref_type) => match ref_type.heap_type() {
                wasmparser::HeapType::Abstract { shared: _, ty } => match ty {
                    wasmparser::AbstractHeapType::Func => DataType::FuncRef,
                    wasmparser::AbstractHeapType::Extern => DataType::ExternRef,
                    wasmparser::AbstractHeapType::Any => DataType::Any,
                    wasmparser::AbstractHeapType::None => DataType::None,
                    wasmparser::AbstractHeapType::NoExtern => DataType::NoExtern,
                    wasmparser::AbstractHeapType::NoFunc => DataType::NoFunc,
                    wasmparser::AbstractHeapType::Eq => DataType::Eq,
                    wasmparser::AbstractHeapType::Struct => DataType::Struct,
                    wasmparser::AbstractHeapType::Array => DataType::Array,
                    wasmparser::AbstractHeapType::I31 => DataType::I31,
                    wasmparser::AbstractHeapType::Exn => DataType::Exn,
                    wasmparser::AbstractHeapType::NoExn => DataType::NoExn,
                },
                wasmparser::HeapType::Concrete(u) => match u {
                    wasmparser::UnpackedIndex::Module(idx) => DataType::Module(ModuleID(idx)),
                    wasmparser::UnpackedIndex::RecGroup(idx) => DataType::RecGroup(idx),
                    wasmparser::UnpackedIndex::Id(_id) => panic!("Not supported yet!"),
                },
            },
        }
    }
}

/// Converts from Orca's DataType to [`wasm_encoder::ValType`].
///
/// [`wasm_encoder::ValType`]: https://docs.rs/wasm-encoder/0.214.0/wasm_encoder/enum.ValType.html
impl From<&DataType> for wasm_encoder::ValType {
    fn from(ty: &DataType) -> Self {
        match ty {
            DataType::I32 => wasm_encoder::ValType::I32,
            DataType::I64 => wasm_encoder::ValType::I64,
            DataType::F32 => wasm_encoder::ValType::F32,
            DataType::F64 => wasm_encoder::ValType::F64,
            DataType::V128 => wasm_encoder::ValType::V128,
            DataType::FuncRef => wasm_encoder::ValType::FUNCREF,
            DataType::ExternRef => wasm_encoder::ValType::EXTERNREF,
            DataType::Any => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Any,
                },
            }),
            DataType::None => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::None,
                },
            }),
            DataType::NoExtern => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::NoExtern,
                },
            }),
            DataType::NoFunc => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::NoFunc,
                },
            }),
            DataType::Eq => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Eq,
                },
            }),
            DataType::Struct => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Struct,
                },
            }),
            DataType::Array => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Array,
                },
            }),
            DataType::I31 => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::I31,
                },
            }),
            DataType::Exn => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Exn,
                },
            }),
            DataType::NoExn => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::NoExn,
                },
            }),
            DataType::Module(idx) => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Concrete(**idx),
            }),
            DataType::RecGroup(idx) => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Concrete(*idx),
            }),
            DataType::CoreTypeId(idx) => wasm_encoder::ValType::Ref(wasm_encoder::RefType {
                nullable: false,
                heap_type: wasm_encoder::HeapType::Concrete(*idx),
            }),
        }
    }
}

impl From<&DataType> for ValType {
    fn from(ty: &DataType) -> Self {
        match ty {
            DataType::I32 => ValType::I32,
            DataType::I64 => ValType::I64,
            DataType::F32 => ValType::F32,
            DataType::F64 => ValType::F64,
            DataType::V128 => ValType::V128,
            DataType::FuncRef => ValType::FUNCREF,
            DataType::ExternRef => ValType::EXTERNREF,
            DataType::Any => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::Any,
                    },
                )
                .unwrap(),
            ),
            DataType::None => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::None,
                    },
                )
                .unwrap(),
            ),
            DataType::NoExtern => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::NoExtern,
                    },
                )
                .unwrap(),
            ),
            DataType::NoFunc => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::NoFunc,
                    },
                )
                .unwrap(),
            ),
            DataType::Eq => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::Eq,
                    },
                )
                .unwrap(),
            ),
            DataType::Struct => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::Struct,
                    },
                )
                .unwrap(),
            ),
            DataType::Array => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::Array,
                    },
                )
                .unwrap(),
            ),
            DataType::I31 => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::I31,
                    },
                )
                .unwrap(),
            ),
            DataType::Exn => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::Exn,
                    },
                )
                .unwrap(),
            ),
            DataType::NoExn => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Abstract {
                        shared: false,
                        ty: wasmparser::AbstractHeapType::NoExn,
                    },
                )
                .unwrap(),
            ),
            DataType::Module(idx) => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Concrete(wasmparser::UnpackedIndex::Module(**idx)),
                )
                .unwrap(),
            ),
            DataType::RecGroup(idx) => ValType::Ref(
                RefType::new(
                    false,
                    wasmparser::HeapType::Concrete(wasmparser::UnpackedIndex::RecGroup(*idx)),
                )
                .unwrap(),
            ),
            DataType::CoreTypeId(_idx) => panic!("Not Supported Yet!"),
        }
    }
}

/// Converts [`ValType`] to [`wasm_encoder::ValType`].
///
/// [`wasm_encoder::ValType`]: https://docs.rs/wasm-encoder/0.214.0/wasm_encoder/enum.ValType.html
/// [`ValType`]: https://docs.rs/wasmparser/latest/wasmparser/enum.ValType.html
pub fn valtype_to_wasmencoder_type(val_type: &ValType) -> wasm_encoder::ValType {
    let mut reencoder = wasm_encoder::reencode::RoundtripReencoder;
    reencoder.val_type(*val_type).unwrap()
}

#[derive(Debug, Clone)]
/// Data Segment in a wasm module.
pub struct DataSegment {
    /// The kind of data segment.
    pub kind: DataSegmentKind,
    /// The data of the data segment.
    pub data: Vec<u8>,
}

impl DataSegment {
    pub fn from_wasmparser(data: wasmparser::Data) -> Result<DataSegment> {
        Ok(DataSegment {
            kind: DataSegmentKind::from_wasmparser(data.kind)?,
            data: data.data.to_vec(),
        })
    }
}

/// The kind of data segment.
#[derive(Debug, Clone)]
pub enum DataSegmentKind {
    /// The data segment is passive.
    Passive,
    /// The data segment is active.
    Active {
        /// The memory index for the data segment.
        memory_index: u32,
        /// The memory offset where this active data segment will be automatically
        /// initialized.
        offset_expr: InitExpr,
    },
}

impl DataSegmentKind {
    pub(crate) fn from_wasmparser(kind: wasmparser::DataKind) -> Result<DataSegmentKind> {
        Ok(match kind {
            wasmparser::DataKind::Passive => DataSegmentKind::Passive,
            wasmparser::DataKind::Active {
                memory_index,
                offset_expr,
            } => DataSegmentKind::Active {
                memory_index,
                offset_expr: InitExpr::eval(&offset_expr),
            },
        })
    }
}

#[derive(Debug, Clone)]
/// Kind of Element
pub enum ElementKind<'a> {
    Passive,
    Active {
        table_index: Option<u32>,
        offset_expr: ConstExpr<'a>,
    },
    Declared,
}

impl ElementKind<'_> {
    pub(crate) fn from_wasmparser(kind: wasmparser::ElementKind) -> Result<ElementKind> {
        match kind {
            wasmparser::ElementKind::Passive => Ok(ElementKind::Passive),
            wasmparser::ElementKind::Declared => Ok(ElementKind::Declared),
            wasmparser::ElementKind::Active {
                table_index,
                offset_expr,
            } => Ok(ElementKind::Active {
                table_index,
                offset_expr,
            }),
        }
    }
}

#[derive(Debug, Clone)]
/// Type of element
pub enum ElementItems<'a> {
    Functions(Vec<FunctionID>),
    ConstExprs {
        ty: RefType,
        exprs: Vec<ConstExpr<'a>>,
    },
}

impl ElementItems<'_> {
    pub(crate) fn from_wasmparser(items: wasmparser::ElementItems) -> Result<ElementItems> {
        match items {
            wasmparser::ElementItems::Functions(reader) => {
                let functions = reader
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                // unsure how to avoid a second iteration (cast while iterating above)
                let fids = functions.iter().map(|id| FunctionID(*id)).collect();
                Ok(ElementItems::Functions(fids))
            }
            wasmparser::ElementItems::Expressions(ref_type, reader) => {
                let exprs = reader
                    .into_iter()
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                Ok(ElementItems::ConstExprs {
                    ty: ref_type,
                    exprs,
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Mode of Function in case the function is mark as instrumented
pub enum FuncInstrMode {
    Entry,
    Exit,
}

#[derive(Default, Debug, Clone)]
/// Instrumentation Data that is stored with every function
pub struct FuncInstrFlag<'a> {
    /// boolean flag to say whether there are special instrumentation
    /// modes to resolve for this function (see InstrumentationMode variants)
    pub has_special_instr: bool,
    pub current_mode: Option<FuncInstrMode>,
    pub entry: Vec<Operator<'a>>,
    pub exit: Vec<Operator<'a>>,
}

impl fmt::Display for FuncInstrFlag<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let FuncInstrFlag {
            has_special_instr,
            entry,
            exit,
            current_mode: _,
        } = self;
        if !self.has_instr() {
            write!(f, "Not Instrumented")?;
        }
        write!(
            f,
            "Has special instrumentation: {}\n \
             Func Entry: {:?} instructions\n \
             Func Exit: {:?} instructions",
            has_special_instr,
            entry.len(),
            exit.len()
        )
    }
}

impl PartialEq for FuncInstrFlag<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Using pattern match to help identify when this function needs to be extended in the future
        let Self {
            has_special_instr,
            entry,
            exit,
            current_mode,
        } = self;
        let mut result = *has_special_instr == other.has_special_instr;
        result &= entry.eq(&other.entry);
        result &= exit.eq(&other.exit);
        result &= discriminant(current_mode) == discriminant(&other.current_mode);

        result
    }
}

impl Eq for FuncInstrFlag<'_> {}

impl<'a> FuncInstrFlag<'a> {
    pub fn has_instr(&self) -> bool {
        // Using pattern match to help identify when this function needs to be extended in the future
        let Self {
            entry,
            exit,
            has_special_instr: _,
            current_mode: _,
        } = self;
        !entry.is_empty() || !exit.is_empty()
    }

    pub fn has_special_instr(&self) -> bool {
        self.has_special_instr
    }

    /// Add an instruction to the current FuncInstrMode's list
    pub fn add_instr(&mut self, val: Operator<'a>) {
        self.has_special_instr = true;
        match self.current_mode {
            None => {
                panic!("Current mode is not set...cannot inject instructions!")
            }
            Some(FuncInstrMode::Entry) => self.entry.push(val),
            Some(FuncInstrMode::Exit) => self.exit.push(val),
        }
    }

    /// Get an instruction to the current FuncInstrMode's list
    pub fn get_instr(&self, idx: usize) -> &Operator {
        match self.current_mode {
            None => {
                panic!("Current mode is not set...cannot grab instruction without context!")
            }
            Some(FuncInstrMode::Entry) => self.entry.get(idx).unwrap(),
            Some(FuncInstrMode::Exit) => self.exit.get(idx).unwrap(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Mode of Instruction in case the instruction is marked as Instrumented
pub enum InstrumentationMode {
    Before,
    After,
    Alternate,

    // special modes
    SemanticAfter,
    BlockEntry,
    BlockExit,
    BlockAlt,
}

#[derive(Default, Debug, Clone)]
/// Instrumentation Data that is stored with every instruction
pub struct InstrumentationFlag<'a> {
    pub current_mode: Option<InstrumentationMode>,
    pub before: Vec<Operator<'a>>,
    pub after: Vec<Operator<'a>>,
    /// None means to replace with no instructions (effectively removing the original)
    /// Some(vec) means to replace with the vec of instructions
    /// Some(empty vec) means there is no alt instrumentation
    pub alternate: Option<Vec<Operator<'a>>>,

    // special modes
    pub semantic_after: Vec<Operator<'a>>,
    pub block_entry: Vec<Operator<'a>>,
    pub block_exit: Vec<Operator<'a>>,
    /// None means to replace with no instructions (effectively removing the original)
    /// Some(vec) means to replace with the vec of instructions
    /// Some(empty vec) means there is no alt instrumentation
    pub block_alt: Option<Vec<Operator<'a>>>,
}

impl fmt::Display for InstrumentationFlag<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let InstrumentationFlag {
            before,
            after,
            alternate,
            semantic_after,
            block_entry,
            block_exit,
            block_alt,
            current_mode: _,
        } = self;
        if !self.has_instr() {
            write!(f, "Not Instrumented")?;
        }
        write!(
            f,
            "Before: {:?} instructions\n \
                   After: {:?} instructions\n \
                   Alternate: {:?} instructions\n \
                   Semantic After: {:?} instructions\n \
                   Block Entry: {:?} instructions\n \
                   Block Exit: {:?} instructions\n \
                   Block Alt: {:?} instructions",
            before.len(),
            after.len(),
            alternate.as_ref().unwrap().len(),
            semantic_after.len(),
            block_entry.len(),
            block_exit.len(),
            block_alt.as_ref().unwrap().len()
        )
    }
}

impl PartialEq for InstrumentationFlag<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Using pattern match to help identify when this function needs to be extended in the future
        let Self {
            before,
            after,
            alternate,
            semantic_after,
            block_entry,
            block_exit,
            block_alt,
            current_mode,
        } = self;
        let mut result = before.eq(&other.before);
        result &= after.eq(&other.after);
        result &= *alternate == other.alternate;
        result &= semantic_after.eq(&other.semantic_after);
        result &= block_entry.eq(&other.block_entry);
        result &= block_exit.eq(&other.block_exit);
        result &= block_alt.eq(&other.block_alt);
        result &= *current_mode == other.current_mode;

        result
    }
}

impl Eq for InstrumentationFlag<'_> {}

impl<'a> InstrumentationFlag<'a> {
    pub fn has_instr(&self) -> bool {
        // Using pattern match to help identify when this function needs to be extended in the future
        let Self {
            before,
            after,
            alternate,
            semantic_after,
            block_entry,
            block_exit,
            block_alt,
            current_mode: _,
        } = self;
        !before.is_empty()
            || !after.is_empty()
            || !alternate.is_none() // Some(vec![]) means instruction removal!
            || !semantic_after.is_empty()
            || !block_entry.is_empty()
            || !block_exit.is_empty()
            || !block_alt.is_none() // Some(vec![]) means block removal!
    }

    /// Add an instruction to the current InstrumentationMode's list
    /// Returns whether the instrumentation was a 'special' mode
    pub fn add_instr(&mut self, op: &Operator, val: Operator<'a>) -> bool {
        match self.current_mode {
            None => {
                panic!("Current mode is not set...cannot inject instructions!")
            }
            Some(InstrumentationMode::Before) => {
                self.before.push(val);
                false
            }
            Some(InstrumentationMode::After) => {
                self.after.push(val);
                false
            }
            Some(InstrumentationMode::Alternate) => {
                match &mut self.alternate {
                    None => self.alternate = Some(vec![val]),
                    Some(alternate) => alternate.push(val),
                }
                false
            }
            Some(InstrumentationMode::SemanticAfter) => {
                // self.semantic_after.push(val);
                // true
                if Self::is_block_style_op(op) || Self::is_branching_op(op) {
                    self.semantic_after.push(val);
                    true
                } else {
                    // instrumentation type not applicable!
                    panic!(
                        "Cannot apply semantic after instrumentation mode to op type: {:?}",
                        op
                    );
                }
            }
            Some(InstrumentationMode::BlockEntry) => {
                if Self::is_block_style_op(op) {
                    self.block_entry.push(val);
                    true
                } else {
                    // instrumentation type not applicable!
                    panic!(
                        "Cannot apply block entry instrumentation mode to op type: {:?}",
                        op
                    );
                }
            }
            Some(InstrumentationMode::BlockExit) => {
                if Self::is_block_style_op(op) {
                    self.block_exit.push(val);
                    true
                } else {
                    // instrumentation type not applicable!
                    panic!(
                        "Cannot apply block exit instrumentation mode to op type: {:?}",
                        op
                    );
                }
            }
            Some(InstrumentationMode::BlockAlt) => {
                if Self::is_block_style_op(op) {
                    match &mut self.block_alt {
                        None => self.block_alt = Some(vec![val]),
                        Some(block_alt) => block_alt.push(val),
                    }
                    true
                } else {
                    // instrumentation type not applicable!
                    panic!(
                        "Cannot apply block alternate instrumentation mode to op type: {:?}",
                        op
                    );
                }
            }
        }
    }

    pub fn clear_instr(&mut self, mode: InstrumentationMode) {
        match mode {
            InstrumentationMode::Before => {
                self.before.clear();
            }
            InstrumentationMode::After => self.after.clear(),
            InstrumentationMode::Alternate => {
                self.alternate = None;
            }
            InstrumentationMode::SemanticAfter => self.semantic_after.clear(),
            InstrumentationMode::BlockEntry => self.block_entry.clear(),
            InstrumentationMode::BlockExit => self.block_exit.clear(),
            InstrumentationMode::BlockAlt => {
                self.block_alt = None;
            }
        }
    }

    fn is_block_style_op(op: &Operator) -> bool {
        matches!(
            op,
            Operator::Block { .. }
                | Operator::Loop { .. }
                | Operator::If { .. }
                | Operator::Else { .. }
        )
    }

    fn is_branching_op(op: &Operator) -> bool {
        matches!(
            op,
            Operator::Br { .. }
                | Operator::BrIf { .. }
                | Operator::BrTable { .. }
                | Operator::BrOnCast { .. }
                | Operator::BrOnCastFail { .. }
                | Operator::BrOnNull { .. }
                | Operator::BrOnNonNull { .. }
        )
    }

    /// Get an instruction to the current InstrumentationMode's list
    pub fn get_instr(&self, idx: usize) -> &Operator {
        match self.current_mode {
            None => {
                panic!("Current mode is not set...cannot grab instruction without context!")
            }
            Some(InstrumentationMode::Before) => self.before.get(idx).unwrap(),
            Some(InstrumentationMode::After) => self.after.get(idx).unwrap(),
            Some(InstrumentationMode::Alternate) => match &self.alternate {
                None => panic!("No alternate instructions to pull idx '{}' from", idx),
                Some(alternate) => alternate.get(idx).unwrap(),
            },
            Some(InstrumentationMode::SemanticAfter) => self.semantic_after.get(idx).unwrap(),
            Some(InstrumentationMode::BlockEntry) => self.block_entry.get(idx).unwrap(),
            Some(InstrumentationMode::BlockExit) => self.block_exit.get(idx).unwrap(),
            Some(InstrumentationMode::BlockAlt) => match &self.block_alt {
                None => panic!("No block alt instructions to pull idx '{}' from", idx),
                Some(block_alt) => block_alt.get(idx).unwrap(),
            },
        }
    }
}

/// Used to represent a unique location in a wasm component or module.
#[derive(Debug, Clone, Copy)]
pub enum Location {
    Component {
        mod_idx: ModuleID,
        func_idx: FunctionID,
        instr_idx: usize,
    },
    Module {
        func_idx: FunctionID,
        instr_idx: usize,
    },
}

#[derive(Debug, Default, Clone)]
/// Body of a function in a wasm module
pub struct Body<'a> {
    /// Local variables of the function, given as tuples of (# of locals, type).
    /// Note that these do not include the function parameters which are given
    /// indices before the locals. So if a function has 2 parameters and a local
    /// defined here then local indices 0 and 1 will refer to the parameters and
    /// index 2 will refer to the local here.
    pub locals: Vec<(u32, DataType)>,
    pub num_locals: usize,
    // accessing operators by .0 is not very clear
    pub instructions: Vec<Instruction<'a>>,
    pub num_instructions: usize,
    pub name: Option<String>,
}

// 'b should outlive 'a
impl<'a, 'b> Body<'a>
where
    'b: 'a,
{
    /// Push a new operator (instruction) to the end of the body
    pub fn push_op(&mut self, op: Operator<'b>) {
        self.instructions.push(Instruction::new(op));
        self.num_instructions += 1;
    }

    /// Get some operator (instruction) at the specified index of the body
    pub fn get_op(&self, idx: usize) -> &Operator {
        &self.instructions[idx].op
    }

    /// Get the instrumentation of some operator in the body
    pub fn get_instr_flag(&self, idx: usize) -> &InstrumentationFlag {
        &self.instructions[idx].instr_flag
    }

    /// Get the instrumentation of some operator in the body
    pub fn clear_instr(&mut self, idx: usize, mode: InstrumentationMode) {
        self.instructions[idx].instr_flag.clear_instr(mode);
    }

    /// Push an end operator (instruction) to the end of the body
    pub fn end(&mut self) {
        self.push_op(Operator::End);
    }
}

#[derive(Debug, Clone)]
pub struct Instruction<'a> {
    pub op: Operator<'a>,
    pub instr_flag: InstrumentationFlag<'a>,
}
impl<'a, 'b> Instruction<'a>
where
    'b: 'a,
{
    pub fn new(op: Operator<'b>) -> Self {
        Self {
            op,
            instr_flag: InstrumentationFlag::default(),
        }
    }

    pub fn add_instr(&mut self, val: Operator<'a>) -> bool {
        self.instr_flag.add_instr(&self.op, val)
    }
}

/// A constant which is produced in WebAssembly, typically used in global
/// initializers or element/data offsets.
#[derive(Debug, Copy, Clone)]
pub enum InitExpr {
    /// An immediate constant value
    Value(Value),
    /// A constant value referenced by the global specified
    Global(GlobalID),
    /// A null reference
    RefNull(RefType),
    /// A function initializer
    RefFunc(FunctionID),
}

impl InitExpr {
    pub(crate) fn eval(init: &ConstExpr) -> InitExpr {
        use wasmparser::Operator::*;
        let mut reader = init.get_operators_reader();
        let val = match reader.read().unwrap() {
            I32Const { value } => InitExpr::Value(Value::I32(value)),
            I64Const { value } => InitExpr::Value(Value::I64(value)),
            F32Const { value } => InitExpr::Value(Value::F32(f32::from_bits(value.bits()))),
            F64Const { value } => InitExpr::Value(Value::F64(f64::from_bits(value.bits()))),
            V128Const { value } => InitExpr::Value(Value::V128(v128_to_u128(&value))),
            GlobalGet { global_index } => InitExpr::Global(GlobalID(global_index)),
            // Marking nullable as true as it's a null reference
            RefNull { hty } => InitExpr::RefNull(RefType::new(true, hty).unwrap()),
            RefFunc { function_index } => InitExpr::RefFunc(FunctionID(function_index)),
            _ => panic!("invalid constant expression"),
        };
        match reader.read().unwrap() {
            End => {}
            _ => panic!("invalid constant expression"),
        }
        reader.ensure_end().unwrap();
        val
    }

    pub(crate) fn to_wasmencoder_type(self) -> wasm_encoder::ConstExpr {
        match self {
            InitExpr::Value(v) => match v {
                Value::I32(v) => wasm_encoder::ConstExpr::i32_const(v),
                Value::I64(v) => wasm_encoder::ConstExpr::i64_const(v),
                Value::F32(v) => wasm_encoder::ConstExpr::f32_const(v),
                Value::F64(v) => wasm_encoder::ConstExpr::f64_const(v),
                Value::V128(v) => wasm_encoder::ConstExpr::v128_const(v as i128),
            },
            InitExpr::Global(g) => wasm_encoder::ConstExpr::global_get(*g),
            InitExpr::RefNull(ty) => wasm_encoder::ConstExpr::ref_null(if ty.is_func_ref() {
                wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Func,
                }
            } else if ty.is_extern_ref() {
                wasm_encoder::HeapType::Abstract {
                    shared: false,
                    ty: AbstractHeapType::Extern,
                }
            } else {
                unreachable!()
            }),
            InitExpr::RefFunc(f) => wasm_encoder::ConstExpr::ref_func(*f),
        }
    }
}

/// Constant values that can show up in WebAssembly
#[derive(Debug, Clone, Copy)]
pub enum Value {
    /// A constant 32-bit integer
    I32(i32),
    /// A constant 64-bit integer
    I64(i64),
    /// A constant 32-bit float
    F32(f32),
    /// A constant 64-bit float
    F64(f64),
    /// A constant 128-bit vector register
    V128(u128),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::I32(i) => i.fmt(f),
            Value::I64(i) => i.fmt(f),
            Value::F32(i) => i.fmt(f),
            Value::F64(i) => i.fmt(f),
            Value::V128(i) => i.fmt(f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockType {
    /// The block produces consumes nor produces any values.
    Empty,
    /// The block produces a singular value of the given type ([] -> \[t]).
    Type(DataType),
    /// The block is described by a function type.
    ///
    /// The index is to a function type in the types section.
    FuncType(TypeID),
}

impl From<wasmparser::BlockType> for BlockType {
    fn from(value: wasmparser::BlockType) -> Self {
        match value {
            wasmparser::BlockType::Empty => BlockType::Empty,
            wasmparser::BlockType::FuncType(u) => BlockType::FuncType(TypeID(u)),
            wasmparser::BlockType::Type(val) => BlockType::Type(DataType::from(val)),
        }
    }
}

impl From<BlockType> for wasmparser::BlockType {
    fn from(ty: BlockType) -> Self {
        match ty {
            BlockType::Empty => wasmparser::BlockType::Empty,
            BlockType::FuncType(u) => wasmparser::BlockType::FuncType(*u),
            BlockType::Type(data) => wasmparser::BlockType::Type(ValType::from(&data)),
        }
    }
}

/// Intermediate Representation of Custom Sections
#[derive(Clone, Debug, Default)]
pub struct CustomSections<'a> {
    custom_sections: Vec<CustomSection<'a>>,
}

impl<'a> CustomSections<'a> {
    pub fn new(custom_sections: Vec<(&'a str, &'a [u8])>) -> Self {
        CustomSections {
            custom_sections: custom_sections
                .iter()
                .map(|cs| CustomSection::new(cs.0, cs.1))
                .collect(),
        }
    }

    /// Get a custom section ID by name
    pub fn get_id(&self, name: String) -> Option<CustomSectionID> {
        for (index, section) in self.custom_sections.iter().enumerate() {
            if section.name == name {
                return Some(CustomSectionID(index as u32));
            }
        }
        None
    }

    /// Get a custom section by its ID
    pub fn get_by_id(&self, custom_section_id: CustomSectionID) -> &CustomSection {
        if *custom_section_id < self.custom_sections.len() as u32 {
            return &self.custom_sections[*custom_section_id as usize];
        }
        panic!("Invalid custom section ID");
    }

    /// Delete a Custom Section by its ID
    pub fn delete(&mut self, id: CustomSectionID) {
        if *id < self.custom_sections.len() as u32 {
            self.custom_sections.remove(*id as usize);
        }
    }

    /// Number of custom sections
    pub fn len(&self) -> usize {
        self.custom_sections.len()
    }

    /// Check if there are any custom sections
    pub fn is_empty(&self) -> bool {
        self.custom_sections.is_empty()
    }

    /// Creates an iterable over the custom sections
    pub fn iter(&self) -> Iter<'_, CustomSection<'a>> {
        self.custom_sections.iter()
    }
}

/// Intermediate Representation of a single Custom Section
#[derive(Clone, Debug)]
pub struct CustomSection<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

impl<'a> CustomSection<'a> {
    /// Create a new custom section
    pub fn new(name: &'a str, data: &'a [u8]) -> Self {
        CustomSection { name, data }
    }
}

#[allow(clippy::identity_op)]
pub(crate) fn v128_to_u128(value: &wasmparser::V128) -> u128 {
    let n = value.bytes();
    ((n[0] as u128) << 0)
        | ((n[1] as u128) << 8)
        | ((n[2] as u128) << 16)
        | ((n[3] as u128) << 24)
        | ((n[4] as u128) << 32)
        | ((n[5] as u128) << 40)
        | ((n[6] as u128) << 48)
        | ((n[7] as u128) << 56)
        | ((n[8] as u128) << 64)
        | ((n[9] as u128) << 72)
        | ((n[10] as u128) << 80)
        | ((n[11] as u128) << 88)
        | ((n[12] as u128) << 96)
        | ((n[13] as u128) << 104)
        | ((n[14] as u128) << 112)
        | ((n[15] as u128) << 120)
}
