use dices_man::ManItem;

fn main() {
    let index = ManItem::root();
    println!("{}", index.content())
}
