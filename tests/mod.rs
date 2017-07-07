#[macro_use] extern crate panic_context;
#[macro_use] extern crate lazy_static;
extern crate gag;

use std::panic::{catch_unwind, UnwindSafe};
use std::sync::Mutex;
use gag::BufferRedirect;

use std::io::Read;

use panic_context::panic_context;

lazy_static! {
    // There may be only one active `gag` redirection but tests
    // are executed in parallel by default, so we have to sync them.
    static ref MUTEX: Mutex<()> = Mutex::new(());
}

fn check_output<F: FnMut() -> () + UnwindSafe>(block: F, expected_output: &str) {
    let _lock = MUTEX.lock().unwrap();
    let mut buf = BufferRedirect::stderr().unwrap();

    let result = catch_unwind(block);
    assert!(result.is_err());

    let mut output = String::new();
    buf.read_to_string(&mut output).unwrap();
    drop(buf);

    assert!(output.starts_with(expected_output));
}

#[test]
fn single_context() {
    check_output(|| {
                     panic_context!("i={}", 2);
                     panic!("boom");
                 },
                 "Panic context:\ni=2\n");
}

#[test]
fn several_contexts() {
    check_output(|| {
                     panic_context!("i={}", 2);
                     panic_context!("j={}", 4);
                     panic!("boom");
                 },
                 "Panic context:\ni=2\nj=4\n");
}

#[test]
fn update_value() {
    check_output(|| {
                     let step = panic_context("step: ");
                     step.update("initialization");
                     step.update("compilation");
                     panic!("boom");
                 },
                 "Panic context:\nstep: compilation\n");
}
