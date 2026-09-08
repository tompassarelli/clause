//! Independently derived pure numeric query code; no module imports or host evaluator.
use super::*;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, ExportKind, ExportSection, Function, FunctionSection,
    GlobalSection, GlobalType, Instruction as W, MemArg, MemorySection, MemoryType, Module,
    TypeSection, ValType,
};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Number,
    Boolean,
    EncodedValue,
}
impl Kind {
    fn wasm(self) -> ValType {
        match self {
            Self::Number => ValType::F64,
            Self::Boolean => ValType::I32,
            Self::EncodedValue => ValType::I64,
        }
    }
}
#[derive(Clone, Copy)]
enum Leaf {
    Slot(u16),
    Argument(u16),
    Binding(u16),
}
struct Input {
    leaf: Leaf,
    kind: Kind,
    offset: u32,
}

pub(super) struct NumericQuery {
    code: Arc<[u8]>,
    inputs: Vec<Input>,
    fixed_bytes: usize,
    row_bytes: usize,
    packed_rows: Mutex<Option<PackedRows>>,
    engine: engine::Retained,
}

// Weak identities cannot keep retired bindings or their values alive. Their
// allocation remains distinct until replacement, so pointer reuse is impossible.
// The fixed payload length preserves encoded offsets and lazy error statuses.
struct PackedRows {
    bindings: Vec<std::sync::Weak<relational::Bindings>>,
    fixed_payload_bytes: usize,
    data: Vec<u8>,
    payload: Vec<u8>,
}

fn known(index: usize, nodes: &[(Node, bool, bool)]) -> Option<Kind> {
    match nodes[index].0 {
        Node::Number(_)
        | Node::Add(..)
        | Node::Subtract(..)
        | Node::Multiply(..)
        | Node::Divide(..)
        | Node::SquareRoot(_)
        | Node::Clamp(..) => Some(Kind::Number),
        Node::Boolean(_)
        | Node::GreaterThan(..)
        | Node::LessThanOrEqual(..)
        | Node::Equal(..)
        | Node::And(..)
        | Node::Not(_) => Some(Kind::Boolean),
        Node::Conditional(_, yes, no) => {
            let yes = known(yes, nodes)?;
            (known(no, nodes) == Some(yes)).then_some(yes)
        }
        _ => None,
    }
}

fn constrain(
    index: usize,
    kind: Kind,
    nodes: &[(Node, bool, bool)],
    kinds: &mut [Option<Kind>],
) -> Option<()> {
    if let Some(previous) = kinds[index] {
        return (previous == kind).then_some(());
    }
    if known(index, nodes).is_some_and(|known| known != kind) {
        return None;
    }
    kinds[index] = Some(kind);
    let mut child = |index, kind| constrain(index, kind, nodes, kinds);
    use Kind::{Boolean as B, Number as N};
    match nodes[index].0 {
        Node::Number(_)
        | Node::Boolean(_)
        | Node::Slot(_)
        | Node::Argument(_)
        | Node::Binding(_) => {}
        Node::Add(a, b)
        | Node::Subtract(a, b)
        | Node::Multiply(a, b)
        | Node::Divide(a, b)
        | Node::GreaterThan(a, b)
        | Node::LessThanOrEqual(a, b) => {
            child(a, N)?;
            child(b, N)?;
        }
        Node::And(a, b) => {
            child(a, B)?;
            child(b, B)?;
        }
        Node::Not(a) => child(a, B)?,
        Node::SquareRoot(a) => child(a, N)?,
        Node::Clamp(a, b, c) => {
            child(a, N)?;
            child(b, N)?;
            child(c, N)?;
        }
        Node::Conditional(a, b, c) => {
            child(a, B)?;
            child(b, kind)?;
            child(c, kind)?;
        }
        Node::Equal(a, b) => {
            let operand = match (known(a, nodes), known(b, nodes)) {
                (Some(a), Some(b)) if a == b => a,
                (None, None) => Kind::EncodedValue,
                _ => return None,
            };
            child(a, operand)?;
            child(b, operand)?;
        }
        _ => return None,
    }
    Some(())
}

struct Emitter<'a> {
    plan: &'a ScalarPlan,
    kinds: Vec<Kind>,
    leaves: Vec<Option<usize>>,
    inputs: Vec<Input>,
    function: Function,
}
impl Emitter<'_> {
    fn op(&mut self, op: W<'_>) {
        self.function.instruction(&op);
    }
    fn value(&self, node: usize) -> u32 {
        4 + node as u32
    }
    fn flag(&self, node: usize) -> u32 {
        4 + self.plan.nodes.len() as u32 + node as u32
    }
    fn failure(&mut self, code: i32) {
        self.op(W::I32Const(code));
        self.op(W::GlobalSet(0));
        self.op(W::Unreachable);
    }
    fn guard(&mut self, code: i32) {
        self.op(W::If(BlockType::Empty));
        self.failure(code);
        self.op(W::End);
    }
    fn finite(&mut self, local: u32) {
        self.op(W::LocalTee(local));
        self.op(W::F64Abs);
        self.op(W::F64Const(f64::MAX.into()));
        self.op(W::F64Le);
        self.op(W::I32Eqz);
        self.guard(1);
        self.op(W::LocalGet(local));
        self.op(W::F64Const(0.0.into()));
        self.op(W::F64Eq);
        self.op(W::If(BlockType::Result(ValType::F64)));
        self.op(W::F64Const(0.0.into()));
        self.op(W::Else);
        self.op(W::LocalGet(local));
        self.op(W::End);
    }
    fn number(&mut self, node: usize) {
        self.node(node);
    }
    fn node(&mut self, node: usize) {
        let cached = self.plan.nodes[node].1;
        if cached {
            self.op(W::LocalGet(self.flag(node)));
            self.op(W::If(BlockType::Result(self.kinds[node].wasm())));
            self.op(W::LocalGet(self.value(node)));
            self.op(W::Else);
        }
        self.fresh(node);
        self.op(W::LocalTee(self.value(node)));
        if cached {
            self.op(W::I32Const(1));
            self.op(W::LocalSet(self.flag(node)));
            self.op(W::End);
        }
    }
    fn address(&mut self, leaf: usize) {
        let input = &self.inputs[leaf];
        let row = matches!(input.leaf, Leaf::Binding(_));
        let offset = input.offset;
        if row {
            self.op(W::LocalGet(2));
        } else {
            self.op(W::I32Const(0));
        }
        self.op(W::I32Const(offset as i32));
        self.op(W::I32Add);
    }
    fn fresh(&mut self, node: usize) {
        let memory = |offset, align| MemArg {
            offset,
            align,
            memory_index: 0,
        };
        match self.plan.nodes[node].0 {
            Node::Number(bits) => self.op(W::F64Const(f64::from_bits(bits).into())),
            Node::Boolean(value) => self.op(W::I32Const(i32::from(value))),
            Node::Slot(_) | Node::Argument(_) | Node::Binding(_) => {
                let leaf = self.leaves[node].expect("typed leaf");
                self.address(leaf);
                self.op(W::I32Load(memory(0, 2)));
                self.op(W::If(BlockType::Empty));
                self.address(leaf);
                self.op(W::I32Load(memory(0, 2)));
                self.op(W::GlobalSet(0));
                self.op(W::I32Const(leaf as i32));
                self.op(W::GlobalSet(1));
                self.op(W::Unreachable);
                self.op(W::End);
                self.address(leaf);
                self.op(match self.kinds[node] {
                    Kind::Number => W::F64Load(memory(8, 3)),
                    Kind::Boolean => W::I32Load(memory(8, 2)),
                    Kind::EncodedValue => W::I64Load(memory(8, 3)),
                });
            }
            Node::Add(a, b) | Node::Subtract(a, b) | Node::Multiply(a, b) => {
                self.number(a);
                self.number(b);
                self.op(match self.plan.nodes[node].0 {
                    Node::Add(..) => W::F64Add,
                    Node::Subtract(..) => W::F64Sub,
                    _ => W::F64Mul,
                });
                self.finite(self.value(node));
            }
            Node::Divide(a, b) => {
                self.number(b);
                self.op(W::Drop);
                self.op(W::LocalGet(self.value(b)));
                self.op(W::F64Const(0.0.into()));
                self.op(W::F64Eq);
                self.guard(1);
                self.number(a);
                self.op(W::LocalGet(self.value(b)));
                self.op(W::F64Div);
                self.finite(self.value(node));
            }
            Node::GreaterThan(a, b) | Node::LessThanOrEqual(a, b) => {
                self.number(a);
                self.number(b);
                self.op(
                    if matches!(self.plan.nodes[node].0, Node::GreaterThan(..)) {
                        W::F64Gt
                    } else {
                        W::F64Le
                    },
                );
            }
            Node::Equal(a, b) => {
                self.node(a);
                if self.kinds[a] == Kind::Number {
                    self.op(W::I64ReinterpretF64);
                }
                self.node(b);
                if self.kinds[b] == Kind::Number {
                    self.op(W::I64ReinterpretF64);
                }
                self.op(match self.kinds[a] {
                    Kind::Number => W::I64Eq,
                    Kind::Boolean => W::I32Eq,
                    Kind::EncodedValue => W::Call(1),
                });
            }
            Node::Conditional(a, b, c) => {
                self.node(a);
                self.op(W::If(BlockType::Result(self.kinds[node].wasm())));
                self.node(b);
                self.op(W::Else);
                self.node(c);
                self.op(W::End);
            }
            Node::And(a, b) => {
                self.node(a);
                self.op(W::If(BlockType::Result(ValType::I32)));
                self.node(b);
                self.op(W::Else);
                self.op(W::I32Const(0));
                self.op(W::End);
            }
            Node::Not(a) => {
                self.node(a);
                self.op(W::I32Eqz);
            }
            Node::SquareRoot(a) => {
                self.number(a);
                self.op(W::Drop);
                self.op(W::LocalGet(self.value(a)));
                self.op(W::F64Const(0.0.into()));
                self.op(W::F64Lt);
                self.guard(1);
                self.op(W::LocalGet(self.value(a)));
                self.op(W::F64Sqrt);
                self.finite(self.value(node));
            }
            Node::Clamp(a, b, c) => {
                self.number(a);
                self.op(W::Drop);
                self.number(b);
                self.op(W::Drop);
                self.number(c);
                self.op(W::Drop);
                self.op(W::LocalGet(self.value(b)));
                self.op(W::LocalGet(self.value(c)));
                self.op(W::F64Gt);
                self.guard(1);
                self.op(W::LocalGet(self.value(a)));
                self.op(W::LocalGet(self.value(b)));
                self.op(W::F64Lt);
                self.op(W::If(BlockType::Result(ValType::F64)));
                self.op(W::LocalGet(self.value(b)));
                self.op(W::Else);
                self.op(W::LocalGet(self.value(a)));
                self.op(W::End);
                self.op(W::LocalSet(self.value(node)));
                self.op(W::LocalGet(self.value(node)));
                self.op(W::LocalGet(self.value(c)));
                self.op(W::F64Gt);
                self.op(W::If(BlockType::Result(ValType::F64)));
                self.op(W::LocalGet(self.value(c)));
                self.op(W::Else);
                self.op(W::LocalGet(self.value(node)));
                self.op(W::End);
                self.finite(self.value(node));
            }
            _ => unreachable!("only fully typed pure numeric plans are emitted"),
        }
    }
}

// Input packing preserves the existing exact value encoding. The compiled
// function owns equality; the host neither compares nor interns values.
fn encoded_equality() -> Function {
    let mut function = Function::new([(4, ValType::I32)]);
    let memory = MemArg {
        offset: 0,
        align: 0,
        memory_index: 0,
    };
    for op in [
        W::LocalGet(0),
        W::I32WrapI64,
        W::LocalSet(2),
        W::LocalGet(1),
        W::I32WrapI64,
        W::LocalSet(3),
        W::LocalGet(0),
        W::I64Const(32),
        W::I64ShrU,
        W::I32WrapI64,
        W::LocalSet(4),
        W::LocalGet(4),
        W::LocalGet(1),
        W::I64Const(32),
        W::I64ShrU,
        W::I32WrapI64,
        W::I32Ne,
        W::If(BlockType::Empty),
        W::I32Const(0),
        W::Return,
        W::End,
        W::Block(BlockType::Empty),
        W::Loop(BlockType::Empty),
        W::LocalGet(5),
        W::LocalGet(4),
        W::I32GeU,
        W::BrIf(1),
        W::LocalGet(2),
        W::LocalGet(5),
        W::I32Add,
        W::I32Load8U(memory),
        W::LocalGet(3),
        W::LocalGet(5),
        W::I32Add,
        W::I32Load8U(memory),
        W::I32Ne,
        W::If(BlockType::Empty),
        W::I32Const(0),
        W::Return,
        W::End,
        W::LocalGet(5),
        W::I32Const(1),
        W::I32Add,
        W::LocalSet(5),
        W::Br(0),
        W::End,
        W::End,
        W::I32Const(1),
        W::End,
    ] {
        function.instruction(&op);
    }
    function
}

impl NumericQuery {
    pub(super) fn compile(plan: &ScalarPlan) -> Option<Arc<Self>> {
        let mut kinds = vec![None; plan.nodes.len()];
        constrain(plan.root, Kind::Number, &plan.nodes, &mut kinds)?;
        let kinds = kinds.into_iter().collect::<Option<Vec<_>>>()?;
        let mut leaves = vec![None; plan.nodes.len()];
        let mut inputs = Vec::new();
        let mut fixed_bytes = 0;
        let mut row_bytes = 0;
        for (index, (node, _, _)) in plan.nodes.iter().enumerate() {
            let leaf = match node {
                Node::Slot(i) => Leaf::Slot(*i),
                Node::Argument(i) => Leaf::Argument(*i),
                Node::Binding(i) => Leaf::Binding(*i),
                _ => continue,
            };
            let offset = if matches!(leaf, Leaf::Binding(_)) {
                let offset = row_bytes;
                row_bytes += 16;
                offset
            } else {
                let offset = fixed_bytes;
                fixed_bytes += 16;
                offset
            };
            leaves[index] = Some(inputs.len());
            inputs.push(Input {
                leaf,
                kind: kinds[index],
                offset,
            });
        }
        let mut locals = vec![(1, ValType::I32), (1, ValType::I32), (1, ValType::F64)];
        locals.extend(kinds.iter().map(|kind| (1, kind.wasm())));
        locals.push((plan.nodes.len() as u32, ValType::I32));
        let mut emitter = Emitter {
            plan,
            kinds,
            leaves,
            inputs,
            function: Function::new(locals),
        };
        emitter.op(W::I32Const(0));
        emitter.op(W::GlobalSet(0));
        emitter.op(W::Block(BlockType::Empty));
        emitter.op(W::Loop(BlockType::Empty));
        emitter.op(W::LocalGet(1));
        emitter.op(W::LocalGet(0));
        emitter.op(W::I32GeU);
        emitter.op(W::BrIf(1));
        emitter.op(W::LocalGet(1));
        emitter.op(W::I32Const(row_bytes as i32));
        emitter.op(W::I32Mul);
        emitter.op(W::I32Const(fixed_bytes as i32));
        emitter.op(W::I32Add);
        emitter.op(W::LocalSet(2));
        for (index, (_, reused, fixed)) in plan.nodes.iter().enumerate() {
            if *reused && !fixed {
                emitter.op(W::I32Const(0));
                emitter.op(W::LocalSet(emitter.flag(index)));
            }
        }
        emitter.op(W::LocalGet(3));
        emitter.node(plan.root);
        emitter.op(W::F64Add);
        emitter.finite(3);
        emitter.op(W::LocalSet(3));
        emitter.op(W::LocalGet(1));
        emitter.op(W::I32Const(1));
        emitter.op(W::I32Add);
        emitter.op(W::LocalSet(1));
        emitter.op(W::Br(0));
        emitter.op(W::End);
        emitter.op(W::End);
        emitter.op(W::LocalGet(3));
        emitter.op(W::End);
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32], [ValType::F64]);
        types
            .ty()
            .function([ValType::I64, ValType::I64], [ValType::I32]);
        module.section(&types);
        let mut functions = FunctionSection::new();
        functions.function(0).function(1);
        module.section(&functions);
        let mut memory = MemorySection::new();
        memory.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memory);
        let mut globals = GlobalSection::new();
        for _ in 0..2 {
            globals.global(
                GlobalType {
                    val_type: ValType::I32,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i32_const(0),
            );
        }
        module.section(&globals);
        let mut exports = ExportSection::new();
        exports
            .export("memory", ExportKind::Memory, 0)
            .export("sum", ExportKind::Func, 0)
            .export("error", ExportKind::Global, 0)
            .export("leaf", ExportKind::Global, 1);
        module.section(&exports);
        let mut code = CodeSection::new();
        code.function(&emitter.function)
            .function(&encoded_equality());
        module.section(&code);
        Some(Arc::new(Self {
            code: module.finish().into(),
            inputs: emitter.inputs,
            fixed_bytes: fixed_bytes as usize,
            row_bytes: row_bytes as usize,
            packed_rows: Mutex::new(None),
            engine: engine::Retained::default(),
        }))
    }
}

fn pack_input(data: &mut [u8], input: &Input, value: Result<&ExecutableValueV1, u32>, payload: &mut Vec<u8>, length: usize) {
    let (status, bits) = match (value, input.kind) {
        (Ok(ExecutableValueV1::Number(bits)), Kind::Number) => (0, *bits),
        (Ok(ExecutableValueV1::Boolean(value)), Kind::Boolean) => (0, u64::from(*value)),
        (Ok(value), Kind::EncodedValue) => {
            let begin = payload.len();
            match encode_value(payload, value) {
                Ok(()) => {
                    let size = payload.len() - begin;
                    let end = length.checked_add(payload.len());
                    if end.is_none_or(|end| end > i32::MAX as usize) {
                        (7, 0)
                    } else {
                        (0, ((size as u64) << 32) | (length + begin) as u64)
                    }
                }
                Err(ExecutableErrorV1::ResourceLimit) => (7, 0),
                Err(_) => (6, 0),
            }
        }
        (Ok(_), _) => (2, 0),
        (Err(code), _) => (code, 0),
    };
    let offset = input.offset as usize;
    data[offset..offset + 4].copy_from_slice(&status.to_le_bytes());
    data[offset + 8..offset + 16].copy_from_slice(&bits.to_le_bytes());
}

impl NumericQuery {
    fn pack(
        &self,
        configuration: &[ExecutableSlotV1],
        arguments: &[ExecutableValueV1],
        matches: &[(relational::Matched, bool)],
    ) -> Result<(Vec<u8>, i32), ExecutableErrorV1> {
        let rows = matches.iter().filter(|(_, accepted)| *accepted).count();
        let length = rows
            .checked_mul(self.row_bytes)
            .and_then(|n| n.checked_add(self.fixed_bytes))
            .filter(|n| *n <= i32::MAX as usize)
            .ok_or(ExecutableErrorV1::ResourceLimit)?;
        let rows_i32 = i32::try_from(rows).map_err(|_| ExecutableErrorV1::ResourceLimit)?;
        let mut data = vec![0; length];
        let mut payload = Vec::new();
        for input in &self.inputs {
            let value = match input.leaf {
                Leaf::Slot(index) => configuration
                    .get(usize::from(index))
                    .ok_or(3)
                    .and_then(|slot| slot.value().ok_or(4)),
                Leaf::Argument(index) => arguments.get(usize::from(index)).ok_or(5),
                Leaf::Binding(_) => continue,
            };
            pack_input(&mut data, input, value, &mut payload, length);
        }
        let fixed_payload_bytes = payload.len();
        let mut retained = self.packed_rows.lock().map_err(|_| ExecutableErrorV1::CarrierRejected)?;
        if let Some(previous) = retained.as_ref().filter(|previous|
            previous.fixed_payload_bytes == fixed_payload_bytes
                && previous.bindings.len() == rows
                && previous.bindings.iter().zip(matches.iter().filter(|(_, accepted)| *accepted))
                    .all(|(previous, (matched, _))| std::ptr::eq(previous.as_ptr(), Arc::as_ptr(&matched.bindings)))) {
            data[self.fixed_bytes..].copy_from_slice(&previous.data);
            data.extend_from_slice(&payload);
            data.extend_from_slice(&previous.payload);
            return Ok((data, rows_i32));
        }
        let mut offset = self.fixed_bytes;
        for (matched, accepted) in matches {
            if !accepted {
                continue;
            }
            for input in &self.inputs {
                if let Leaf::Binding(index) = input.leaf {
                    pack_input(
                        &mut data[offset..], input, matched.bindings.get(&index).ok_or(6),
                        &mut payload, length,
                    );
                }
            }
            offset += self.row_bytes;
        }
        *retained = Some(PackedRows {
            bindings: matches.iter().filter(|(_, accepted)| *accepted)
                .map(|(matched, _)| Arc::downgrade(&matched.bindings)).collect(),
            fixed_payload_bytes,
            data: data[self.fixed_bytes..].to_vec(),
            payload: payload[fixed_payload_bytes..].to_vec(),
        });
        data.extend_from_slice(&payload);
        Ok((data, rows_i32))
    }

    fn error(&self, code: i32, leaf: i32) -> ExecutableErrorV1 {
        match code {
            1 => ExecutableErrorV1::NumericDomain,
            2 => ExecutableErrorV1::TypeMismatch,
            3 => match self.inputs.get(leaf as usize).map(|input| input.leaf) {
                Some(Leaf::Slot(index)) => ExecutableErrorV1::UnknownSlot(index),
                _ => ExecutableErrorV1::MalformedProgram,
            },
            4 => ExecutableErrorV1::MissingState,
            5 => match self.inputs.get(leaf as usize).map(|input| input.leaf) {
                Some(Leaf::Argument(index)) => ExecutableErrorV1::UnknownArgument(index),
                _ => ExecutableErrorV1::MalformedProgram,
            },
            6 => ExecutableErrorV1::MalformedProgram,
            7 => ExecutableErrorV1::ResourceLimit,
            _ => ExecutableErrorV1::CarrierRejected,
        }
    }

    pub(super) fn sum(
        &self,
        configuration: &[ExecutableSlotV1],
        arguments: &[ExecutableValueV1],
        matches: &[(relational::Matched, bool)],
    ) -> Result<f64, ExecutableErrorV1> {
        let (data, rows) = self.pack(configuration, arguments, matches)?;
        self.engine.run(self, &data, rows)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod engine {
    use super::*;
    use wasmtime::{Engine, Global, Instance, Memory, Module, Store, TypedFunc};
    struct Compiled {
        store: Store<()>,
        memory: Memory,
        sum: TypedFunc<i32, f64>,
        error: Global,
        leaf: Global,
    }
    #[derive(Default)]
    pub(super) struct Retained(Mutex<Option<Compiled>>);
    impl Retained {
        pub(super) fn run(
            &self,
            query: &NumericQuery,
            data: &[u8],
            rows: i32,
        ) -> Result<f64, ExecutableErrorV1> {
            let mut retained = self
                .0
                .lock()
                .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
            if retained.is_none() {
                let _profile = source_profile_scope_v1(SourceProfilePhaseV1::ScalarPlanBuild);
                static ENGINE: std::sync::OnceLock<Engine> = std::sync::OnceLock::new();
                let engine = ENGINE.get_or_init(Engine::default);
                let module = Module::new(engine, &query.code)
                    .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
                let mut store = Store::new(engine, ());
                let instance = Instance::new(&mut store, &module, &[])
                    .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
                let memory = instance
                    .get_memory(&mut store, "memory")
                    .ok_or(ExecutableErrorV1::CarrierRejected)?;
                let sum = instance
                    .get_typed_func::<i32, f64>(&mut store, "sum")
                    .map_err(|_| ExecutableErrorV1::CarrierRejected)?;
                let error = instance
                    .get_global(&mut store, "error")
                    .ok_or(ExecutableErrorV1::CarrierRejected)?;
                let leaf = instance
                    .get_global(&mut store, "leaf")
                    .ok_or(ExecutableErrorV1::CarrierRejected)?;
                *retained = Some(Compiled {
                    store,
                    memory,
                    sum,
                    error,
                    leaf,
                });
            }
            let compiled = retained
                .as_mut()
                .ok_or(ExecutableErrorV1::CarrierRejected)?;
            let Compiled {
                store,
                memory,
                sum,
                error,
                leaf,
            } = compiled;
            let pages = data.len().div_ceil(65536) as u64;
            let current = memory.size(&*store);
            if pages > current {
                memory
                    .grow(&mut *store, pages - current)
                    .map_err(|_| ExecutableErrorV1::ResourceLimit)?;
            }
            memory
                .write(&mut *store, 0, data)
                .map_err(|_| ExecutableErrorV1::ResourceLimit)?;
            sum.call(&mut *store, rows).map_err(|_| {
                query.error(
                    error.get(&mut *store).i32().unwrap_or(0),
                    leaf.get(&mut *store).i32().unwrap_or(-1),
                )
            })
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod engine {
    use super::*;
    use js_sys::{Function, Object, Reflect, Uint8Array, WebAssembly as Wasm};
    use wasm_bindgen::{JsCast, JsValue};
    struct Compiled {
        memory: Wasm::Memory,
        sum: Function,
        error: Wasm::Global,
        leaf: Wasm::Global,
    }
    thread_local! { static INSTANCES:std::cell::RefCell<BTreeMap<usize,Compiled>>=std::cell::RefCell::new(BTreeMap::new()); }
    pub(super) struct Retained(usize);
    impl Default for Retained {
        fn default() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
            Self(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        }
    }
    impl Drop for Retained {
        fn drop(&mut self) {
            let _ = INSTANCES.try_with(|instances| {
                instances.borrow_mut().remove(&self.0);
            });
        }
    }
    impl Retained {
        pub(super) fn run(
            &self,
            query: &NumericQuery,
            data: &[u8],
            rows: i32,
        ) -> Result<f64, ExecutableErrorV1> {
            INSTANCES.with(|instances| {
                let rejected = |_| ExecutableErrorV1::CarrierRejected;
                let mut instances = instances.borrow_mut();
                if let std::collections::btree_map::Entry::Vacant(entry) = instances.entry(self.0) {
                    let bytes = Uint8Array::from(query.code.as_ref());
                    let module = Wasm::Module::new(&bytes).map_err(rejected)?;
                    let instance =
                        Wasm::Instance::new(&module, &Object::new()).map_err(rejected)?;
                    let exports = instance.exports();
                    let get = |name: &str| {
                        Reflect::get(&exports, &JsValue::from_str(name)).map_err(rejected)
                    };
                    entry.insert(Compiled {
                        memory: get("memory")?.dyn_into().map_err(rejected)?,
                        sum: get("sum")?.dyn_into().map_err(rejected)?,
                        error: get("error")?.dyn_into().map_err(rejected)?,
                        leaf: get("leaf")?.dyn_into().map_err(rejected)?,
                    });
                }
                let compiled = instances
                    .get(&self.0)
                    .ok_or(ExecutableErrorV1::CarrierRejected)?;
                let existing = Uint8Array::new(&compiled.memory.buffer()).length() as usize;
                if data.len() > existing {
                    let grow = Reflect::get(&compiled.memory, &JsValue::from_str("grow"))
                        .map_err(rejected)?
                        .dyn_into::<Function>()
                        .map_err(rejected)?;
                    grow.call1(
                        &compiled.memory,
                        &JsValue::from((data.len() - existing).div_ceil(65536) as u32),
                    )
                    .map_err(|_| ExecutableErrorV1::ResourceLimit)?;
                }
                Uint8Array::new(&compiled.memory.buffer())
                    .subarray(0, data.len() as u32)
                    .copy_from(data);
                compiled
                    .sum
                    .call1(&JsValue::UNDEFINED, &JsValue::from(rows))
                    .map_err(|_| {
                        query.error(
                            compiled.error.value().as_f64().unwrap_or(0.0) as i32,
                            compiled.leaf.value().as_f64().unwrap_or(-1.0) as i32,
                        )
                    })?
                    .as_f64()
                    .ok_or(ExecutableErrorV1::CarrierRejected)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExecutableExpressionV1 as E;
    fn number(n: f64) -> E {
        E::Constant(ExecutableValueV1::number(n).unwrap())
    }
    fn boolean(b: bool) -> E {
        E::Constant(ExecutableValueV1::Boolean(b))
    }
    fn interpreted(
        plan: &ScalarPlan,
        arguments: &[ExecutableValueV1],
        rows: &[(relational::Matched, bool)],
    ) -> Result<u64, ExecutableErrorV1> {
        let memo = plan.memo();
        let mut sum = 0.0;
        for (row, accepted) in rows {
            if !accepted {
                continue;
            }
            let context = EvaluationContextV1 {
                allocation_root: [0; IDENTITY_BYTES],
                step_ordinal: 0,
                reads: None,
                sum_queries: None,
                scalar_memo: None,
                bindings: Some(&row.bindings),
                relational_occurrence: None,
            };
            sum += memo
                .evaluate(&plan.expression, &[], arguments, context)?
                .as_number()
                .ok_or(ExecutableErrorV1::TypeMismatch)?;
            if !sum.is_finite() {
                return Err(ExecutableErrorV1::NumericDomain);
            }
        }
        Ok(canonical_number_bits(sum))
    }
    fn check(expression: E, arguments: &[ExecutableValueV1], rows: &[(relational::Matched, bool)]) {
        let plan = ScalarPlan::new(&expression).unwrap();
        let actual = plan
            .compiled_sum(&[], arguments, rows)
            .expect("pure numeric plan selected")
            .map(canonical_number_bits);
        assert_eq!(
            actual,
            interpreted(&plan, arguments, rows),
            "{expression:?}"
        );
    }
    #[test]
    fn compiled_equality_preserves_exact_values_and_lazy_missing_inputs() {
        let expression = E::Conditional(
            Box::new(E::Equal(Box::new(E::Binding(0)), Box::new(E::Argument(0)))),
            Box::new(number(1.0)),
            Box::new(number(0.0)),
        );
        let plan = ScalarPlan::new(&expression).unwrap();
        let values = [
            ExecutableValueV1::Referent(ExecutableReferentV1::declared(3, 7)),
            ExecutableValueV1::Referent(ExecutableReferentV1::declared(4, 7)),
            ExecutableValueV1::Referent(ExecutableReferentV1::declared(3, 8)),
            ExecutableValueV1::Referent(ExecutableReferentV1::created(3, [7; IDENTITY_BYTES])),
            ExecutableValueV1::Referent(ExecutableReferentV1::created(4, [7; IDENTITY_BYTES])),
            ExecutableValueV1::Referent(ExecutableReferentV1::created(3, [8; IDENTITY_BYTES])),
            ExecutableValueV1::Number(0),
            ExecutableValueV1::Number((-0.0f64).to_bits()),
            ExecutableValueV1::Boolean(false),
            ExecutableValueV1::Boolean(true),
            ExecutableValueV1::symbol(b"same").unwrap(),
            ExecutableValueV1::Text(ExecutableTextV1::new("same").unwrap()),
            ExecutableValueV1::Text(ExecutableTextV1::new("same\0").unwrap()),
        ];
        for argument in &values {
            let rows: Vec<_> = values
                .iter()
                .map(|value| {
                    (
                        relational::Matched {
                            bindings: Arc::new(relational::Bindings::from([(0, value.clone())])),
                            predicates: Vec::new(),
                        },
                        true,
                    )
                })
                .collect();
            assert_eq!(
                plan.compiled_sum(&[], std::slice::from_ref(argument), &rows)
                    .expect("exact equality selected")
                    .map(canonical_number_bits),
                interpreted(&plan, std::slice::from_ref(argument), &rows)
            );
        }
        let missing = [(relational::Matched::default(), true)];
        check(expression.clone(), &[], &missing);
        check(
            expression.clone(),
            &[],
            &[(relational::Matched::default(), false)],
        );
        check(
            E::Conditional(
                Box::new(boolean(false)),
                Box::new(expression),
                Box::new(number(3.0)),
            ),
            &[],
            &missing,
        );
    }

    #[test]
    fn retained_packed_rows_preserve_changed_inputs_rows_and_lazy_errors() {
        let expression = E::Conditional(
            Box::new(E::Equal(Box::new(E::Binding(0)), Box::new(E::Argument(0)))),
            Box::new(number(7.0)),
            Box::new(E::Binding(1)),
        );
        let plan = ScalarPlan::new(&expression).unwrap();
        let text = |value: &str| ExecutableValueV1::text(value).unwrap();
        let mut rows = vec![(relational::Matched {
            bindings: Arc::new(relational::Bindings::from([
                (0, text("alpha")), (1, ExecutableValueV1::number(3.0).unwrap()),
            ])), predicates: Vec::new(),
        }, true)];
        let arguments = [text("alpha"), text("omega"), text("longer encoded argument"), text("alpha")];
        for argument in &arguments {
            let args = std::slice::from_ref(argument);
            assert_eq!(plan.compiled_sum(&[], args, &rows).unwrap().map(canonical_number_bits),
                interpreted(&plan, args, &rows));
            assert_eq!(Arc::strong_count(&rows[0].0.bindings), 1);
        }
        // A copy-on-write mutation cannot match the retained immutable bindings.
        Arc::make_mut(&mut rows[0].0.bindings).insert(1, ExecutableValueV1::Boolean(false));
        for argument in &arguments {
            let args = std::slice::from_ref(argument);
            assert_eq!(plan.compiled_sum(&[], args, &rows).unwrap().map(canonical_number_bits),
                interpreted(&plan, args, &rows));
        }
        rows[0].1 = false;
        assert_eq!(plan.compiled_sum(&[], &[], &rows).unwrap(), Ok(0.0));
    }

    #[test]
    fn compiled_arithmetic_preserves_overflow_and_canonical_zero_checks() {
        let rows = [(relational::Matched::default(), true)];
        for overflow in [
            E::Add(Box::new(number(f64::MAX)), Box::new(number(f64::MAX))),
            E::Subtract(Box::new(number(-f64::MAX)), Box::new(number(f64::MAX))),
            E::Multiply(Box::new(number(f64::MAX)), Box::new(number(2.0))),
            E::Divide(Box::new(number(f64::MAX)), Box::new(number(0.5))),
        ] {
            check(overflow.clone(), &[], &rows);
            check(
                E::Conditional(
                    Box::new(boolean(false)),
                    Box::new(overflow),
                    Box::new(number(7.0)),
                ),
                &[],
                &rows,
            );
        }
        check(number(f64::MAX), &[], &[
            (relational::Matched::default(), true),
            (relational::Matched::default(), true),
        ]);
        let negative_zero = || E::Constant(ExecutableValueV1::Number((-0.0f64).to_bits()));
        for zero in [
            E::Multiply(Box::new(number(0.0)), Box::new(number(-1.0))),
            E::SquareRoot(Box::new(negative_zero())),
            E::Clamp(Box::new(negative_zero()), Box::new(number(-1.0)), Box::new(number(1.0))),
        ] {
            let expression = E::Conditional(
                Box::new(E::Equal(Box::new(zero), Box::new(number(0.0)))),
                Box::new(number(1.0)),
                Box::new(number(0.0)),
            );
            check(expression.clone(), &[], &rows);
            assert_eq!(ScalarPlan::new(&expression).unwrap().compiled_sum(&[], &[], &rows).unwrap(), Ok(1.0));
        }
    }

    #[test]
    fn compiled_query_preserves_selected_errors_and_ordered_arithmetic() {
        let rows: Vec<_> = [1e16, 1.0, -1e16, -0.0, 3.0]
            .into_iter()
            .map(|n| {
                (
                    relational::Matched {
                        bindings: Arc::new(relational::Bindings::from([(
                            0,
                            ExecutableValueV1::number(n).unwrap(),
                        )])),
                        predicates: Vec::new(),
                    },
                    true,
                )
            })
            .collect();
        let expressions = [
            E::Binding(0),
            E::Add(Box::new(E::Binding(0)), Box::new(E::Argument(0))),
            E::Conditional(
                Box::new(boolean(false)),
                Box::new(E::Argument(9)),
                Box::new(E::Binding(0)),
            ),
            E::Divide(Box::new(E::Argument(9)), Box::new(number(0.0))),
            E::Divide(Box::new(E::Binding(0)), Box::new(E::Argument(0))),
            E::SquareRoot(Box::new(E::Binding(0))),
            E::Clamp(
                Box::new(E::Binding(0)),
                Box::new(number(-0.0)),
                Box::new(number(9.0)),
            ),
            E::Clamp(
                Box::new(E::Binding(0)),
                Box::new(number(9.0)),
                Box::new(number(1.0)),
            ),
            E::Conditional(
                Box::new(E::And(
                    Box::new(E::LessThanOrEqual(
                        Box::new(E::Binding(0)),
                        Box::new(number(1.0)),
                    )),
                    Box::new(E::Not(Box::new(boolean(false)))),
                )),
                Box::new(number(3.0)),
                Box::new(E::Slot(9)),
            ),
            E::Add(Box::new(E::Argument(9)), Box::new(E::Slot(9))),
        ];
        for expression in expressions {
            for argument in [
                ExecutableValueV1::Boolean(false),
                ExecutableValueV1::number(0.0).unwrap(),
                ExecutableValueV1::number(2.0).unwrap(),
            ] {
                check(expression.clone(), &[argument], &rows);
            }
            check(expression.clone(), &[], &[]);
            let rejected = vec![(relational::Matched::default(), false)];
            check(expression, &[], &rejected);
        }
        let expression = E::Conditional(
            Box::new(E::Equal(Box::new(E::Argument(0)), Box::new(number(1.0)))),
            Box::new(number(2.0)),
            Box::new(number(3.0)),
        );
        assert!(
            NumericQuery::compile(&ScalarPlan::new(&expression).unwrap()).is_none(),
            "untyped equality remains interpreted"
        );
    }
}
