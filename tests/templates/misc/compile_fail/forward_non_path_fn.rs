use pre::pre;

fn foo() {}

#[pre]
fn main() {
    let array = [foo];

    #[forward(foo)]
    array[0]();
}
