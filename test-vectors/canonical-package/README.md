# Clause v0 canonical-package corpus

This directory freezes the normative Clause v0 package bytes described by
`clause:docs/canonical-package.md`.

Each `.hex` file contains lowercase hexadecimal octets plus ASCII whitespace.
Whitespace is transport formatting and is not package content. Consumers must
decode an even number of hex digits and then apply the binary grammar. The
positive directory contains the one literal bootstrap and its one authorized
successor. Each negative entry changes one named condition from that specimen;
`manifest.json` records the stage and verdict being tested.

`SHA256SUMS` is content-addressing evidence for corpus handling only. Digests
never replace exact predecessor, bootstrap, frame, claim, or package bytes in
Clause validation.
