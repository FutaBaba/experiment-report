use std::sync::Arc;

fn main() {
    let five = Arc::new(String::from("Hello"));
    let five_2 = Arc::new(String::from("Hello"));

    assert!(five == five_2);
}
