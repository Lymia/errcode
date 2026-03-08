use errcode::prelude::*;
use errcode::{Error, ErrorCode, error_info};

#[derive(ErrorCode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestCode {
    E1,
    E2,
}

#[test]
fn option_convert() {
    let some: Option<i32> = Some(10);
    let none: Option<i32> = None;

    assert_eq!(some.convert(error_info!("error")).unwrap(), 10);
    let err: Error = none.convert(error_info!("none error")).unwrap_err();
    assert!(err.to_string().contains("none error"));
}

#[test]
fn option_convert_code() {
    let none: Option<i32> = None;
    let err = none.convert_code(TestCode::E1).unwrap_err();
    assert!(err.is(TestCode::E1));
}

#[test]
fn result_convert() {
    let ok: core::result::Result<i32, &str> = Ok(20);
    let err: core::result::Result<i32, &str> = Err("something went wrong!");

    let ok: Result<_> = ok.convert(error_info!("aaa aaa"));
    let err: Result<_> = err.convert(error_info!("aaa aaa"));

    assert_eq!(ok.unwrap(), 20);
    assert!(err.is_err());

    let err_string = err.unwrap_err().to_string();
    assert!(err_string.contains("aaa aaa"), "Line: {err_string}");
    if cfg!(feature = "repr_full") {
        assert!(err_string.contains("&str"), "Line: {err_string}");
    } else {
        assert!(err_string.contains("<original error type lost>"), "Line: {err_string}")
    }
}

#[test]
fn result_convert_code() {
    let err: core::result::Result<i32, &str> = Err("something went wrong!");

    let error = err.convert_code(TestCode::E2).unwrap_err();
    assert!(error.is(TestCode::E2));
    let err_string = error.to_string();

    assert!(err_string.contains("TestCode::E2"), "Line: {err_string}");
    if cfg!(feature = "repr_full") {
        assert!(err_string.contains("&str"), "Line: {err_string}");
    }
}

#[test]
fn with_context() {
    let res: Result<()> = Err(Error::from_info(error_info!("base")));
    let res_with_ctx = res.with_context(error_info!("added context"));

    let err = res_with_ctx.unwrap_err();
    let err_string = err.to_string();
    assert!(err_string.contains("added context"), "Line: {err_string}");
    assert!(err_string.contains("base"), "Line: {err_string}");
}

#[test]
fn with_context_code() {
    let res: Result<()> = Err(Error::from_info(error_info!("base")));
    let res_with_ctx = res.with_context_code(TestCode::E1);

    let error = res_with_ctx.unwrap_err();
    assert!(error.is(TestCode::E1));
    let err_string = error.to_string();
    assert!(err_string.contains("TestCode::E1"), "Line: {err_string}");
}
