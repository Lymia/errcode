use errcode::{Error, error_info};

fn main() {
    let error = Error::from_info(error_info!("hello, world!"));
    let error = error.with_context(error_info!("test! {}", 3));

    println!("{error}");
}
