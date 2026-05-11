// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

use derive_builder::Builder;

#[derive(Builder, Default)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

pub trait Foo {
    /// Does a foo thing
    ///
    /// Args:
    /// * `a` - The value for a
    fn foo(&self, a: usize);
}

impl Foo for Command {
    fn foo(&self, a: usize) {
        println!("Hello, foo!")
    }
}

fn main() {
    let mut builder = Command::builder();
    // builder.executable("cargo".to_owned());
    // builder.args(vec!["build".to_owned(), "--release".to_owned()]);
    // builder.env(vec![]);
    // builder.current_dir("..".to_owned());
    // let c = Command::default();
}
