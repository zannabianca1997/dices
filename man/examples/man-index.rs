use dices_man::Manual;
use itertools::Itertools;

fn main() {
    let manual = Manual::new();
    for page in manual.first().descendant().sorted() {
        if !page.path().is_empty() {
            print!("{}. ", page.path().iter().format("."))
        }
        println!("{}", page.title())
    }
}
