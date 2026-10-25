# Project license decision

2026-10-22: Owner explicitly approved applying MIT to original code and authored curriculum, with “DLMS Learning Lab contributors” in the notice. The applied terms are in ../../LICENSE. Third-party licenses remain separate; dependency redistribution review is required for candidates.

2026-10-25: The macOS `dispatch 0.2.0` original-notice gate was rechecked.
The published crate matches Cargo.lock and upstream revision
`82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748`; its nine original published files
match the pinned upstream tree. Neither inspected source supplies the original
copyright/permission notice. [Upstream issue18](https://github.com/SSheldon/rust-dispatch/issues/18)
was closed without supplying one; that closure does not resolve the gate.
The collector remains unchanged. No copyright year, notice or substitute license
was invented. Closing this packaging gate requires original notice provenance or
a separately reviewed dependency/distribution decision. It does not block the
independent Fedora curriculum and application work.
