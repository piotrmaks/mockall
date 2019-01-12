// vim: tw=80

use mockall::*;

#[test]
#[should_panic(expected = "No matching expectation found")]
fn missing_expectation() {
    let e = Expectations::default();
    e.called::<i32, u32>(&"foo", 5);
}

/// A MockObject with a method that takes &mut self like:
/// fn foo(&mut self, x: i32) -> u32
#[test]
fn mutable_self() {
    let mut e = Expectations::default();
    let mut count = 0;
    e.expect::<i32, i32>(&"foo")
        .returning(move |x| {
            count += x;
            count
        });
    assert_eq!(5, e.called::<i32, i32>(&"foo", 5));
    assert_eq!(10, e.called::<i32, i32>(&"foo", 5));
}

/// A MockObject with a method that has no arguments or returns
/// fn foo(&self)
#[test]
fn no_args_or_returns() {
    let mut e = Expectations::default();
    e.expect::<(), ()>(&"foo")
        .returning(|_| ());
    e.called::<(), ()>(&"foo", ());
}

/// A MockObject with a simple method like:
/// fn foo(&self, x: i32) -> u32
#[test]
fn simple_method() {
    let mut e = Expectations::default();
    e.expect::<i32, u32>(&"foo")
        .returning(|_| 42);
    let r = e.called::<i32, u32>(&"foo", 5);
    assert_eq!(42, r);
}