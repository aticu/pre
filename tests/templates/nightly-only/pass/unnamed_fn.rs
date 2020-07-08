use pre::pre;

#[pre("precondition")]
fn foo() {}

#[pre]
fn main() {
    let fn_array = [foo];

    #[assure("precondition", reason = "precondition holds")]
    fn_array[0]();
}
