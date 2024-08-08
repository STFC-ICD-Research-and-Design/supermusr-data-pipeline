# 7. Standardise error handling

Date: 2024-07-30

## Status

Accepted

## Context

A good amount of the pipeline contains fallible operations, this is unavoidable.

Currently these are handled in roughly one of four ways:

1. This should never happen under normal or abnormal execution: `unwrap()`
2. This might fail, we cannot recover and don't care about any state (usually in setup procedures): `expect()`
3. This might fail and the callee needs to care (i.e. in a library): `thiserror`
4. This might fail and the user might care (i.e. in a binary): `anyhow`

This comes with the following issues:

- Lack of clean up/reporting when `expect()` panics
- Lack of context when `unwrap()` is used

## Decision

The above rules will be replaced with:

1. This should never happen under normal or abnormal execution: `expect()`
2. This might fail and the callee needs to care (i.e. in a library or modules containing logic in a binary): `thiserror`
3. This might fail and the user might care (i.e. in only the setup proedure in a binary): `anyhow`

The key changes, for clarity:

- Move cases of 2 into 4
- Forbid `unwrap()` and `panic()`
- `anyhow` is only ever allowed to be the return value of `main()`

`unwrap()` is also used extensively in automated tests, for the time being the above rules will only apply to production code, not testing.
"Production code" in this case is defined as anything not inside or descending from a module named `test`.

## Consequences

Error handling throughout the codebase will need to be audited to ensure it complies with the above (there are certainly a good amount of changes that will need to be made).

Automated tooling should be considered to automatically detect uses of forbidden calls (rustc or Clippy may already have suitable lints for this which could be enabled).

A follow up task later will be to look at the use of other operators and functions which might panic, e.g. `[]` vs `get()`.
