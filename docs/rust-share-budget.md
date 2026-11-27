# Rust source-share budget

The final clean tracked source must be at least 50% genuine Rust bytes. The budget protects implementation planning; it never substitutes for meaningful code.

| Source class | Planning range |
| --- | ---: |
| Rust production across 20–25 meaningful crates | 280–340 KiB |
| Rust behavior/failure/property/integration tests | 100–140 KiB |
| React/TypeScript frontend and typed client | 70–95 KiB |
| JavaScript/shell gate, evidence, launch, and test tooling | 90–120 KiB |
| Other recognized source | <= 20 KiB |

Target at least 58% measured Rust before final evidence to leave margin. Run the pinned Rust-share checker throughout implementation and inspect its included/excluded paths.

Forbidden tactics: padded comments, generated/copied Rust, duplicated behavior, ceremonial crates, hidden required frontend/tooling, language overrides, tracked dependency/build trees, or deleting product behavior to raise the percentage.

The 25-crate map is semantic. No byte or line threshold makes a crate meaningful.
