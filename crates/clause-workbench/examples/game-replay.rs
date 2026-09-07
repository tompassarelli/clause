//! Record exact resident-session traffic; gameplay remains in the supplied source.
use clause_package::canonical_term_bytes;
use clause_runtime::*;
use clause_workbench::ResidentSourceWorkbenchV1;
use std::{error::Error, fmt::Write, path::PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Recorder {
    boundary: WasmPersistentSessionBoundaryV1,
    event: WasmSessionEventV1,
    directory: PathBuf,
    manifest: String,
    ordinal: usize,
}

impl Recorder {
    fn command(
        &mut self,
        input: usize,
        phase: &str,
        operation: WasmSessionOperationV1,
    ) -> Result<()> {
        let command = encode_wasm_session_command_v1(&WasmSessionCommandV1 {
            handle: self.event.handle,
            expected_sequence: self.event.accepted_sequence,
            operation,
        })?;
        self.event = self.boundary.command(&command)?;
        if matches!(
            self.event.kind,
            WasmSessionEventKindV1::Rejected { .. }
                | WasmSessionEventKindV1::CandidateRejected { .. }
        ) {
            return Err(format!("input {input} {phase}: {:?}", self.event.kind).into());
        }
        std::fs::write(
            self.directory.join(format!("{}.cwi1", self.ordinal)),
            command,
        )?;
        std::fs::write(
            self.directory.join(format!("{}.cse1", self.ordinal)),
            encode_wasm_session_event_v1(&self.event),
        )?;
        writeln!(self.manifest, "{}\t{input}\t{phase}", self.ordinal)?;
        self.ordinal += 1;
        Ok(())
    }

    fn admit(&mut self, input: usize) -> Result<()> {
        let (base, candidate) = match self.event.kind {
            WasmSessionEventKindV1::CandidateAccepted {
                base, candidate, ..
            } => (base, candidate),
            _ => return Err("tick did not produce a candidate".into()),
        };
        let opened =
            decode_wasm_session_event_v1(&std::fs::read(self.directory.join("opened.cse1"))?)?;
        let (package, session) = match opened.kind {
            WasmSessionEventKindV1::Opened {
                package, session, ..
            } => (package, session),
            _ => return Err("missing opened session".into()),
        };
        self.command(
            input,
            "authorize",
            WasmSessionOperationV1::IssueAdmission(WasmSessionAdmissionScopeV1 {
                package,
                session,
                base,
                candidate,
            }),
        )?;
        let authorization = match self.event.kind {
            WasmSessionEventKindV1::AdmissionAuthorizationIssued { occurrence, .. } => occurrence,
            _ => return Err("authorization was not issued".into()),
        };
        self.command(
            input,
            "admit",
            WasmSessionOperationV1::Admit(WasmSessionAdmissionV1 {
                package,
                session,
                base,
                candidate,
                authorization,
            }),
        )
    }
}

fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let [source_path, trace_path, directory] = args.as_slice() else {
        return Err("usage: game-replay SOURCE.clause INPUTS.txt NEW_OUTPUT_DIRECTORY".into());
    };
    let source = std::fs::read(source_path)?;
    let trace = std::fs::read_to_string(trace_path)?;
    let directory = PathBuf::from(directory);
    // An existing record is evidence, never an implicit overwrite destination.
    std::fs::create_dir(&directory)?;
    let workbench = ResidentSourceWorkbenchV1::open_continuous(&source)?;
    let request = decode_wasm_process_request_v1(&workbench.generation().cwr1)?;
    let physical = decode_executable_physical_plan_v1(&request.physical_plan_bytes)?;
    if let Some(metadata) = physical.source_metadata {
        std::fs::write(
            directory.join("source-metadata.term"),
            canonical_term_bytes(&metadata)?,
        )?;
    }
    let open = encode_wasm_session_open_v1(&WasmSessionOpenV1 {
        package_bytes: request.package_bytes,
        application: request.application,
        physical_plan_bytes: request.physical_plan_bytes,
        authority: request.authority,
        allocation: WasmSessionAllocationV1::Rematerialize(request.allocation),
        limits: WasmSessionLimitsV1 {
            max_commands: 4096,
            command_bytes: WASM_SESSION_COMMAND_LIMIT_V1.try_into()?,
            event_bytes: WASM_SESSION_EVENT_LIMIT_V1.try_into()?,
            trace_retention: WasmSessionTraceRetentionV1::CurrentAdmission,
        },
    })?;
    drop(workbench);
    let mut boundary = WasmPersistentSessionBoundaryV1::new();
    let event = boundary.open(&open)?;
    std::fs::write(directory.join("source.clause"), source)?;
    std::fs::write(directory.join("inputs.txt"), &trace)?;
    std::fs::write(directory.join("initial.cwi1"), open)?;
    std::fs::write(
        directory.join("opened.cse1"),
        encode_wasm_session_event_v1(&event),
    )?;
    std::fs::write(
        directory.join("initial.term"),
        boundary.current_accepted_projection_bytes(event.handle)?,
    )?;
    let mut recorder = Recorder {
        boundary,
        event,
        directory,
        manifest: String::new(),
        ordinal: 0,
    };
    let mut input_sequence = 0;
    let mut configuration_revision = 0;
    let mut count = 0;
    for (line, text) in trace.lines().enumerate() {
        let words = text.split_whitespace().collect::<Vec<_>>();
        if words.is_empty() || words[0].starts_with('#') {
            continue;
        }
        let input = count;
        count += 1;
        let operation = match words.as_slice() {
            ["key", code, phase] => {
                let phase = match *phase {
                    "down" => ExecutableKeyPhaseV1::Down,
                    "up" => ExecutableKeyPhaseV1::Up,
                    _ => return Err(format!("input {input}, line {}: expected down or up", line + 1).into()),
                };
                input_sequence += 1;
                WasmSessionOperationV1::PhysicalInput(WasmSessionPhysicalInputV1 {
                    input_sequence,
                    source: ExecutableInputSourceV1::Keyboard { code: code.as_bytes().to_vec(), phase },
                    value: None,
                })
            }
            ["scalar", channel, value] => {
                input_sequence += 1;
                WasmSessionOperationV1::PhysicalInput(WasmSessionPhysicalInputV1 {
                    input_sequence,
                    source: ExecutableInputSourceV1::Scalar { channel: channel.as_bytes().to_vec() },
                    value: Some(ExecutableValueV1::number(value.parse()?)?),
                })
            }
            ["tick", milliseconds] => {
                configuration_revision += 1;
                WasmSessionOperationV1::TickCandidate(WasmSessionTickV1 {
                    configuration_revision, fixed_tick_milliseconds: milliseconds.parse()?,
                })
            }
            _ => return Err(format!("input {input}, line {}: expected key CODE down|up, scalar CHANNEL NUMBER, or tick MILLISECONDS", line + 1).into()),
        };
        recorder.command(input, words[0], operation)?;
        if words[0] == "tick" {
            recorder.admit(input)?;
        }
    }
    recorder.command(count, "dispose", WasmSessionOperationV1::Dispose)?;
    std::fs::write(recorder.directory.join("commands.tsv"), &recorder.manifest)?;
    println!(
        "recorded {count} explicit inputs, {} exact native events in {}",
        recorder.ordinal,
        recorder.directory.display()
    );
    Ok(())
}
