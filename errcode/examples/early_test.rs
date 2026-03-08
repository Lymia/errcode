use errcode::ErrorCode;
use errcode::prelude::ConvertErrorHelper;
use errcode::{Error, error_info};

#[derive(ErrorCode)]
pub enum TestCode {
    A,
    B,
    C,
}

fn io_test(f: &str) -> Result<String, Error> {
    let result = std::fs::read_to_string(f)?;
    Ok(result)
}

fn main() {
    let error = Error::from_info(error_info!("hello, world!"));
    let error = error.with_context(error_info!("test! {}", 3));
    let error = error.with_context(error_info!(TestCode::A, "test! {}", 3));
    let error = error.with_context(error_info!(TestCode::A, "something went wrong"));

    println!("{error}");
    println!();
    println!("{:?}", io_test("/etc/os-release"));
    println!("{:?}", io_test("/etc/does-not-exist"));
    println!();

    if let Err(e) = io_test("/etc/does-not-exist").with_context(error_info!("context")) {
        println!("{}", e)
    } else {
        panic!()
    }
}
