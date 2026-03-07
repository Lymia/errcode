use errcode::{Error, error_info};

fn io_test(f: &str) -> Result<String, Error> {
    let result = std::fs::read_to_string(f)?;
    Ok(result)
}

fn main() {
    let error = Error::from_info(error_info!("hello, world!"));
    let error = error.with_context(error_info!("test! {}", 3));

    println!("{error}");
    println!("{:?}", io_test("/etc/os-release"));
    println!("{:?}", io_test("/etc/does-not-exist"));

    if let Err(e) = io_test("/etc/does-not-exist") {
        println!("{}", e)
    } else {
        panic!()
    }
}
