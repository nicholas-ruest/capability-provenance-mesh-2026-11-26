# Planned integration matrix

Exact revisions, licenses, bindings, and source URLs must be resolved and probed before selection. This plan is not a selection receipt.

| Canonical upstream | Category | Product responsibility | Intended binding | Success operation | Failure assertion |
| --- | --- | --- | --- | --- | --- |
| `ruvnet/rvm` | Ruvnet | Package identity, capability mapping, witnessed refusal | direct pinned Rust crates | verify fixture and map classes | altered header/unsupported class refuses before allocation |
| `ruvnet/rune` | Ruvnet | Frozen build-plan and artifact-lineage verification | pinned executable or direct crate API | replay plan and match digest | source/lock drift causes stale-plan refusal |
| `ruvnet/LatentMesh` | Ruvnet | Signed, fragmented, replay-defended trial observations | direct pinned Rust crate | encode/round-trip/reassemble envelope | corrupt/replayed fragment rejected |
| `ruvnet/worldgraph` | Ruvnet | Typed capability/dependency/effect provenance graph | direct pinned Rust crate | add/query typed reachability graph | invalid/untrusted relation cannot authorize |
| `ruvnet/APx` | Ruvnet | Task usefulness/eligibility against frozen reference pack | pinned executed local service/process | reproduce accepted score | incomplete/unsafe/unmetered evidence ineligible |
| `bytecodealliance/wasmtime` | Enterprise | Actual bounded WASM execution | direct pinned Rust crates | permitted host-call fixture | undeclared call or fuel limit traps with no effect |
| `cedar-policy/cedar` | Enterprise | Tenant/action/resource/context policy | direct pinned Rust crates | parse/evaluate compliant request | denied principal/context prevents activation |
| `apache/opendal` | Enterprise | Durable object/event/snapshot boundary | direct pinned Rust crate | write/read/restart reconstruction | conflict/missing object leaves no partial pointer |

## Counting notes

- RVM inspection is not represented as WASM execution; Wasmtime owns execution.
- Required coordination tools do not count unless they perform a demonstrated runtime product responsibility.
- Packages, subcrates, aliases, or forks from one canonical upstream count once.
- If any responsibility cannot be executed through the pinned source, replace the ingredient and redo probes/selection.
