use errcode::{error_info, Error};

fn main() {
    let info = error_info!("hello, world! {}", 3);
    let error = Error::from_info(info);

    println!("{error}");
}
