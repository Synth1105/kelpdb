use kelpdb::prelude::*;

fn main() {
    let scv = Scuver::new(DB::new("example", "hello"));
    println!("{:#?}",scv.run("GET example".to_string()).unwrap());
    println!("{:#?}",scv.run("SET example true".to_string()).unwrap());
    println!("{:#?}",scv.run("GET example".to_string()).unwrap());
}
