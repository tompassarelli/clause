//! Opt-in, bounded wall-clock observations of the existing checked source path.
//! No profile observation participates in source identity, checking or execution.
use std::cell::RefCell;

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum SourceProfilePhaseV1 {
    Transfer,
    WitnessCheck,
    SourceRead,
    Allocation,
    OfferedEdit,
    OldElaboration,
    NewElaboration,
    Lowering,
    SnapshotMetadata,
    RowProjection,
    Cpp1Decode,
    CompareAndMap,
    Instantiate,
    Migration,
    NativeEdit,
    SourceEditBulk,
    ClearIo,
    InstallEvent,
    EventExport,
}
const NAMES: [&str; 19] = [
    "transfer",
    "witness-check",
    "source-read",
    "allocation",
    "offered-edit",
    "old-elaboration",
    "new-elaboration",
    "lowering",
    "snapshot-metadata",
    "row-projection",
    "cpp1-decode",
    "compare-and-map",
    "instantiate",
    "migration",
    "native-edit",
    "source-edit-bulk",
    "clear-io",
    "install-event",
    "event-export",
];

#[derive(Clone, Copy, Debug, Default)]
pub struct SourceProfileMeasurementV1 {
    pub calls: u64,
    pub inclusive_milliseconds: f64,
    pub exclusive_milliseconds: f64,
}
#[derive(Clone, Debug)]
pub struct ExecutableSourceProfileV1 {
    pub wall_milliseconds: f64,
    pub truncated: bool,
    pub phases: [SourceProfileMeasurementV1; 19],
}
impl ExecutableSourceProfileV1 {
    /// Fixed field names and finite numeric measurements; no source or secrets.
    pub fn to_json(&self) -> String {
        let fields = self
            .phases
            .iter()
            .zip(NAMES)
            .map(|(phase, name)| {
                format!(
                    "\"{name}\":{{\"calls\":{},\"inclusiveMs\":{},\"exclusiveMs\":{}}}",
                    phase.calls, phase.inclusive_milliseconds, phase.exclusive_milliseconds
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"clock\":\"monotonic-wall-ms\",\"wallMs\":{},\"truncated\":{},\"phases\":{{{fields}}}}}",
            self.wall_milliseconds, self.truncated
        )
    }
}
struct Frame {
    phase: usize,
    started: f64,
    child_milliseconds: f64,
}
struct Active {
    started: f64,
    frames: Vec<Frame>,
    report: ExecutableSourceProfileV1,
}
thread_local! { static ACTIVE: RefCell<Option<Active>> = const { RefCell::new(None) }; }

#[cfg(not(target_arch = "wasm32"))]
fn now() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f64()
        * 1000.0
}
#[cfg(target_arch = "wasm32")]
fn now() -> f64 {
    use wasm_bindgen::prelude::wasm_bindgen;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = performance, js_name = now)]
        fn performance_now() -> f64;
    }
    performance_now()
}

/// One active profile per execution thread. Re-entry refuses, never resets a
/// caller's active observation. No clock reads occur while profiling is off.
pub fn begin_executable_source_profile_v1() -> bool {
    ACTIVE.with_borrow_mut(|active| {
        if active.is_some() {
            return false;
        }
        *active = Some(Active {
            started: now(),
            frames: Vec::with_capacity(16),
            report: ExecutableSourceProfileV1 {
                wall_milliseconds: 0.0,
                truncated: false,
                phases: [SourceProfileMeasurementV1::default(); 19],
            },
        });
        true
    })
}
pub fn finish_executable_source_profile_v1() -> Option<ExecutableSourceProfileV1> {
    ACTIVE.with_borrow_mut(|active| {
        if active.as_ref()?.frames.len() != 0 {
            return None;
        }
        let mut result = active.take()?;
        result.report.wall_milliseconds = (now() - result.started).max(0.0);
        Some(result.report)
    })
}

#[must_use]
pub struct SourceProfileScopeV1 {
    depth: Option<usize>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}
pub fn source_profile_scope_v1(phase: SourceProfilePhaseV1) -> SourceProfileScopeV1 {
    let depth = ACTIVE.with_borrow_mut(|active| {
        let active = active.as_mut()?;
        if active.frames.len() >= 64 {
            active.report.truncated = true;
            return None;
        }
        let depth = active.frames.len();
        active.frames.push(Frame {
            phase: phase as usize,
            started: now(),
            child_milliseconds: 0.0,
        });
        Some(depth)
    });
    SourceProfileScopeV1 {
        depth,
        _thread: std::marker::PhantomData,
    }
}
impl Drop for SourceProfileScopeV1 {
    fn drop(&mut self) {
        let Some(depth) = self.depth else {
            return;
        };
        ACTIVE.with_borrow_mut(|active| {
            let Some(active) = active else {
                return;
            };
            if active.frames.len() != depth + 1 {
                active.report.truncated = true;
                return;
            }
            let frame = active.frames.pop().unwrap();
            let elapsed = (now() - frame.started).max(0.0);
            let measurement = &mut active.report.phases[frame.phase];
            measurement.calls = measurement.calls.saturating_add(1);
            measurement.inclusive_milliseconds += elapsed;
            measurement.exclusive_milliseconds += (elapsed - frame.child_milliseconds).max(0.0);
            if let Some(parent) = active.frames.last_mut() {
                parent.child_milliseconds += elapsed;
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn clause_source_profile_v1_begin() -> bool {
    begin_executable_source_profile_v1()
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn clause_source_profile_v1_finish() -> String {
    finish_executable_source_profile_v1()
        .map(|report| report.to_json())
        .unwrap_or_else(|| "null".into())
}
