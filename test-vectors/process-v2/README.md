# Clause process-v2 experimental Rust carrier corpus

This directory freezes `CLPV` version 2 experiment packages. The current Rust
encoder produces every accepted fixture; the one deliberately noncanonical
support-slot mutation starts from that exact encoder output. `SHA256SUMS` binds
the transport files and `manifest.json` names each expected decode, check,
authority, or replay verdict. This is not yet a canonical Clause wire format:
its byte contract has no host-neutral ratification or independent decoder.

`positive/process-v2-core.hex` is the compact proving package. It contains two
nominal Applications with one structural shape, distinct Activations of one
Application, multi-Step progress and suspension, externally entered resumption
after carrier rematerialization, a deliberately live Activation without a fake
result, causally independent sibling Steps, explicit true/false/absent
observations, repeated occurrence-exact support slots, one admitted State
delta, and one governed rejection. Only the admitted decision creates a
successor `StateRevision`.

The eight named negative files isolate:

- candidate bytes attempting to supply their own admission grant;
- substitution of otherwise canonical package bytes;
- a produced binding where a closed Application shape is required;
- an external trigger with no constituted boundary or evidence;
- resumption without fresh entered provenance;
- a second takeup of one linear continuation;
- collapse of two required support slots; and
- a second decision for one candidate delta.

`exact-byte-substitution.hex` is intentionally well-formed. Its verdict is an
exact `ProcessPackageId` change relative to the positive package, not a decode
failure. `support-collapse.hex` is intentionally noncanonical and is rejected
at strict decode. All other stage-specific verdicts are recorded in the
manifest and exercised by `crates/clause-package/tests/process_v2.rs`.

There is no live process-v1 compatibility decoder. Historical rejected v1
bytes remain only in Git history and have no semantic standing.
