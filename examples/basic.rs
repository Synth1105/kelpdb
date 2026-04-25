use kelpdb::prelude::*;

fn main() {
    let mut db = DB::new("user", String::from("John"));

    db.set("user", 25i32);
    db.set("user", 180.5f64);
    
    db.add_row("posts", String::from("asdf"));
    
    db.set("posts", String::from("sdfsdf"));

    
    println!("Posts: {:#?}", db.get("posts"));
    println!("User data count: {}", db.get_by_type::<i32>("user").len());
    println!("Height count: {}", db.get_by_type::<f64>("user").len());
}
