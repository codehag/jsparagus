use crate::stencil::ScriptStencilIndex;
use ast::source_atom_set::SourceAtomSetIndex;

#[derive(Debug)]
pub struct FunctionFlags {
    flags: u16,
}

// WARNING
// The following section is generated by
// crates/emitter/scripts/update_opcodes.py.
// Do mot modify manually.
//
// @@@@ BEGIN TYPES @@@@
#[derive(Debug)]
pub enum FunctionKind {
    NormalFunction = 0,
    Arrow = 1,
    Method = 2,
    ClassConstructor = 3,
    Getter = 4,
    Setter = 5,
    AsmJS = 6,
    Wasm = 7,
    FunctionKindLimit = 8,
}

#[derive(Debug)]
pub enum GeneratorKind {
    NotGenerator = 0,
    Generator = 1,
}

#[derive(Debug)]
pub enum FunctionAsyncKind {
    SyncFunction = 0,
    AsyncFunction = 1,
}

#[allow(dead_code)]
const FUNCTION_KIND_SHIFT: u16 = 0;
#[allow(dead_code)]
const FUNCTION_KIND_MASK: u16 = 0x0007;
#[allow(dead_code)]
const EXTENDED: u16 = 1 << 3;
#[allow(dead_code)]
const SELF_HOSTED: u16 = 1 << 4;
#[allow(dead_code)]
const BASESCRIPT: u16 = 1 << 5;
#[allow(dead_code)]
const SELFHOSTLAZY: u16 = 1 << 6;
#[allow(dead_code)]
const CONSTRUCTOR: u16 = 1 << 7;
#[allow(dead_code)]
const BOUND_FUN: u16 = 1 << 8;
#[allow(dead_code)]
const LAMBDA: u16 = 1 << 9;
#[allow(dead_code)]
const WASM_JIT_ENTRY: u16 = 1 << 10;
#[allow(dead_code)]
const HAS_INFERRED_NAME: u16 = 1 << 11;
#[allow(dead_code)]
const ATOM_EXTRA_FLAG: u16 = 1 << 12;
#[allow(dead_code)]
const HAS_GUESSED_ATOM: u16 = ATOM_EXTRA_FLAG;
#[allow(dead_code)]
const HAS_BOUND_FUNCTION_NAME_PREFIX: u16 = ATOM_EXTRA_FLAG;
#[allow(dead_code)]
const RESOLVED_NAME: u16 = 1 << 13;
#[allow(dead_code)]
const RESOLVED_LENGTH: u16 = 1 << 14;
#[allow(dead_code)]
const NEW_SCRIPT_CLEARED: u16 = 1 << 15;
#[allow(dead_code)]
const NORMAL_KIND: u16 = (FunctionKind::NormalFunction as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const ASMJS_KIND: u16 = (FunctionKind::AsmJS as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const WASM_KIND: u16 = (FunctionKind::Wasm as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const ARROW_KIND: u16 = (FunctionKind::Arrow as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const METHOD_KIND: u16 = (FunctionKind::Method as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const CLASSCONSTRUCTOR_KIND: u16 = (FunctionKind::ClassConstructor as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const GETTER_KIND: u16 = (FunctionKind::Getter as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const SETTER_KIND: u16 = (FunctionKind::Setter as u16) << FUNCTION_KIND_SHIFT;
#[allow(dead_code)]
const NATIVE_FUN: u16 = NORMAL_KIND;
#[allow(dead_code)]
const NATIVE_CTOR: u16 = CONSTRUCTOR | NORMAL_KIND;
#[allow(dead_code)]
const ASMJS_CTOR: u16 = CONSTRUCTOR | ASMJS_KIND;
#[allow(dead_code)]
const ASMJS_LAMBDA_CTOR: u16 = CONSTRUCTOR | LAMBDA | ASMJS_KIND;
#[allow(dead_code)]
const WASM: u16 = WASM_KIND;
#[allow(dead_code)]
const INTERPRETED_NORMAL: u16 = BASESCRIPT | CONSTRUCTOR | NORMAL_KIND;
#[allow(dead_code)]
const INTERPRETED_CLASS_CTOR: u16 = BASESCRIPT | CONSTRUCTOR | CLASSCONSTRUCTOR_KIND;
#[allow(dead_code)]
const INTERPRETED_GENERATOR_OR_ASYNC: u16 = BASESCRIPT | NORMAL_KIND;
#[allow(dead_code)]
const INTERPRETED_LAMBDA: u16 = BASESCRIPT | LAMBDA | CONSTRUCTOR | NORMAL_KIND;
#[allow(dead_code)]
const INTERPRETED_LAMBDA_ARROW: u16 = BASESCRIPT | LAMBDA | ARROW_KIND;
#[allow(dead_code)]
const INTERPRETED_LAMBDA_GENERATOR_OR_ASYNC: u16 = BASESCRIPT | LAMBDA | NORMAL_KIND;
#[allow(dead_code)]
const INTERPRETED_GETTER: u16 = BASESCRIPT | GETTER_KIND;
#[allow(dead_code)]
const INTERPRETED_SETTER: u16 = BASESCRIPT | SETTER_KIND;
#[allow(dead_code)]
const INTERPRETED_METHOD: u16 = BASESCRIPT | METHOD_KIND;
#[allow(dead_code)]
const MUTABLE_FLAGS: u16 = RESOLVED_NAME | RESOLVED_LENGTH | NEW_SCRIPT_CLEARED;
#[allow(dead_code)]
const STABLE_ACROSS_CLONES: u16 = CONSTRUCTOR | LAMBDA | SELF_HOSTED | FUNCTION_KIND_MASK;
// @@@@ END TYPES @@@@

impl FunctionFlags {
    pub fn new(flags: u16) -> Self {
        debug_assert!(
            (((FunctionKind::FunctionKindLimit as u16) - 1) << FUNCTION_KIND_SHIFT)
                <= FUNCTION_KIND_MASK
        );

        Self { flags }
    }
}

#[derive(Debug)]
pub struct NonLazyFunctionScript {
    script: ScriptStencilIndex,
}

#[derive(Debug)]
pub struct LazyFunctionScript {
    closed_over_bindings: Vec<SourceAtomSetIndex>,
    inner_functions: Vec<FunctionCreationDataIndex>,
    force_strict: bool,
    strict: bool,
}

#[derive(Debug)]
pub enum FunctionScript {
    NonLazy(NonLazyFunctionScript),
    Lazy(LazyFunctionScript),
}

/// Partially maps to FunctionCreationData in m-c/js/src/frontend/Stencil.h
#[derive(Debug)]
pub struct FunctionCreationData {
    name: Option<SourceAtomSetIndex>,
    script: FunctionScript,
    generator_kind: GeneratorKind,
    async_kind: FunctionAsyncKind,
    flags: FunctionFlags,
    // FIXME: add more fields
}

impl FunctionCreationData {
    #[allow(dead_code)]
    pub fn non_lazy(
        name: Option<SourceAtomSetIndex>,
        script: ScriptStencilIndex,
        generator_kind: GeneratorKind,
        async_kind: FunctionAsyncKind,
        flags: FunctionFlags,
    ) -> Self {
        Self {
            name,
            script: FunctionScript::NonLazy(NonLazyFunctionScript { script }),
            generator_kind,
            async_kind,
            flags,
        }
    }

    pub fn lazy(
        name: Option<SourceAtomSetIndex>,
        generator_kind: GeneratorKind,
        async_kind: FunctionAsyncKind,
        flags: FunctionFlags,
    ) -> Self {
        Self {
            name,
            script: FunctionScript::Lazy(LazyFunctionScript {
                closed_over_bindings: Vec::new(),
                inner_functions: Vec::new(),
                force_strict: false,
                strict: false,
            }),
            generator_kind,
            async_kind,
            flags,
        }
    }
}

/// Index into FunctionCreationDataList.items.
#[derive(Debug, Clone, Copy)]
pub struct FunctionCreationDataIndex {
    index: usize,
}

impl FunctionCreationDataIndex {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

impl From<FunctionCreationDataIndex> for usize {
    fn from(index: FunctionCreationDataIndex) -> usize {
        index.index
    }
}

/// List of FunctionCreationData.
#[derive(Debug)]
pub struct FunctionCreationDataList {
    items: Vec<FunctionCreationData>,
}

impl FunctionCreationDataList {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, fun_data: FunctionCreationData) -> FunctionCreationDataIndex {
        let index = self.items.len();
        self.items.push(fun_data);
        FunctionCreationDataIndex::new(index)
    }
}

impl From<FunctionCreationDataList> for Vec<FunctionCreationData> {
    fn from(list: FunctionCreationDataList) -> Vec<FunctionCreationData> {
        list.items
    }
}