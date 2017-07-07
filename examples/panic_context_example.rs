#![doc(hidden)]

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
