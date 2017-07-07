# Panic context

This library allows to print manually-maintained messages on panic.

When your program panics, it prints backtrace. However, if panic
occurs inside loop, it is not clear which iteration was the cause.
It is possible to use log, but printing slows down execution considerably
and you get lots of entries, while only last ones are required.

Panic context lets you set value which is remembered, but not printed anywhere
until panic occurs. It is also automatically forgotten at the end of scope.

## Example

```rust
#[macro_use] extern crate panic_context;

use panic_context::panic_context;

static ITEMS: &[&str] = &["foo", "bar", "yo", "nope"];

fn get_len(item: &str) -> usize { item.len() }
fn calc_sig(item: &str) -> &str { &item[3..] }

fn main() {
    let step = panic_context("step: ");

    step.update("calculate lengths");
    for item in ITEMS {
        panic_context!("item: {}", item);
        get_len(item);
    }

    step.update("calculate signatures");
    for item in ITEMS {
        panic_context!("item: {}", item);
        calc_sig(item);
    }

    panic!("boom!");
}
```

When this code panics inside `calc_sig`, you will see:

```text
Panic context:
step: calculate signatures
item: yo
thread 'main' panicked at '...', src/libcore/str/mod.rs:2162
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```
