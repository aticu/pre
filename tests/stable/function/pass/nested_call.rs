use pre::pre;

#[pre("not called directly")]
unsafe fn foo() -> Result<usize, ()> {
    Ok(0)
}

#[pre]
fn main() -> Result<(), ()> {
    let array = [1, 2, 3];

    #[assure("not called directly", reason = "nested in multiple other expressions")]
    let one_ref = &unsafe { array[foo()?] };

    assert_eq!(*one_ref, 1);

    Ok(())
}
