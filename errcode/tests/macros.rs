use errcode::{ErrorCode, error_info, error, bail, ensure, prelude::*};

#[derive(ErrorCode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestCode {
    A,
    B,
}

#[test]
fn test_error_info_no_args() {
    let info = error_info!();
    // Default is "error encountered"
    // We can't easily check ErrorInfo directly as its fields are private or opaque
    // but we can check the resulting Error's string representation.
    let err: errcode::Error = errcode::Error::from_info(info);
    assert!(err.to_string().contains("error encountered"));
}

#[test]
fn test_error_info_format_literal() {
    let info = error_info!("test message");
    let err = errcode::Error::from_info(info);
    assert!(err.to_string().contains("test message"));
}

#[test]
fn test_error_info_format_args() {
    let info = error_info!("test message: {}", 42);
    let err = errcode::Error::from_info(info);
    
    #[cfg(feature = "repr_full")]
    {
        assert!(err.to_string().contains("test message: 42"));
    }
    #[cfg(not(feature = "repr_full"))]
    {
        assert!(err.to_string().contains("test message: {}"));
    }
}

#[test]
fn test_error_info_code_only() {
    let info = error_info!(TestCode::A);
    let err = errcode::Error::from_info(info);
    assert!(err.is(TestCode::A));
}

#[test]
fn test_error_info_code_and_format() {
    let info = error_info!(TestCode::B, "with message");
    let err = errcode::Error::from_info(info);
    assert!(err.is(TestCode::B));
    assert!(err.to_string().contains("with message"));
}

#[test]
fn test_error_info_code_and_format_args() {
    let info = error_info!(TestCode::A, "with message: {}", "val");
    let err = errcode::Error::from_info(info);
    assert!(err.is(TestCode::A));
    #[cfg(feature = "repr_full")]
    {
        assert!(err.to_string().contains("with message: val"));
    }
    #[cfg(not(feature = "repr_full"))]
    {
        assert!(err.to_string().contains("with message: {}"));
    }
}

#[test]
fn test_error_macro() {
    let err = error!("error macro test");
    assert!(err.to_string().contains("error macro test"));
    
    let err = error!(TestCode::B);
    assert!(err.is(TestCode::B));
}

#[test]
fn test_bail_macro() {
    fn produces_error() -> Result<()> {
        bail!("bailed out");
    }
    
    let res = produces_error();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.to_string().contains("bailed out"));
}

#[test]
fn test_bail_macro_with_code() {
    fn produces_error() -> Result<()> {
        bail!(TestCode::A, "bailed with code");
    }
    
    let res = produces_error();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.is(TestCode::A));
    assert!(err.to_string().contains("bailed with code"));
}

#[test]
fn test_ensure_macro() {
    fn check_condition(cond: bool) -> Result<()> {
        ensure!(cond, "condition failed");
        Ok(())
    }
    
    assert!(check_condition(true).is_ok());
    let res = check_condition(false);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("condition failed"));
}

#[test]
fn test_ensure_macro_no_args() {
    fn check_condition(cond: bool) -> Result<()> {
        ensure!(cond);
        Ok(())
    }
    
    assert!(check_condition(true).is_ok());
    let res = check_condition(false);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("error encountered"));
}

#[test]
fn test_ensure_macro_with_code() {
    fn check_condition(cond: bool) -> Result<()> {
        ensure!(cond, TestCode::B, "failed with code");
        Ok(())
    }
    
    assert!(check_condition(true).is_ok());
    let res = check_condition(false);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.is(TestCode::B));
    assert!(err.to_string().contains("failed with code"));
}
