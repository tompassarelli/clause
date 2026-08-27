# Clause Core Lean bootstrap

This package will contain Clause's small constitutional checker and reference
Run semantics. It starts without semantic definitions so implementation
convenience cannot seed Clause's taxonomy.

The package may model only the host-neutral calculus owned by
`docs/foundation.md`. Lean syntax, expressions, type classes, serialization,
and one-constructor-per-language-feature inductives are implementation tools,
not Clause authority.

The constitutional closure must remain safe and total, contain no `sorry`, and
follow the trust profile in `docs/architecture.md`.
