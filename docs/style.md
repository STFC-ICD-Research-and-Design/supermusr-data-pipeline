# Style

## Architecture

Individual data processing components should typically only do one thing: taking a single input stream, transforming it and producing an output stream.
Should a data format boundary need to be crossed (e.g. file loading or saving) this should be done by a dedicated utility.

## Code

In general whatever `treefmt` (and it's downstream formatters) dictates the code style to use.

In cases where the formatter does not care, the following rules apply:

### :crab: No empty lines in `use` statements

```rust
use crate::Something;
use super::SomethingElse
use std::time::Duration
use supermusr_common::Time
use tokio::task::JoinHandle;
```

instead of

```rust
use crate::Something;

use tokio::task::JoinHandle;
use supermusr_common::Time

use std::time::Duration
use super::SomethingElse
```

### :crab: One empty line between `fn`, `impl`, and `mod` block

```rust
fn one() -> i32 {
  1
}

fn two() -> i32 {
  1
}
```

instead of

```rust
fn one() -> i32 {
  1
}
fn two() -> i32 {
  1
}
```
