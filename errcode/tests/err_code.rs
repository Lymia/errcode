use errcode::{Error, ErrorCode, error_info};

#[derive(ErrorCode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Code1 {
    A,
    B,
}

#[derive(ErrorCode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Code2 {
    X,
    Y,
}

#[test]
fn has_code_functions() {
    let err = Error::from_info(error_info!("no code"));
    assert!(!err.has_code());

    let err = Error::from_code(Code1::A);
    assert!(err.has_code());

    let err = Error::from_info(error_info!(Code2::X, "has code"));
    assert!(err.has_code());

    let err = Error::from_info(error_info!("no code"))
        .with_context_code(Code2::X)
        .with_context(error_info!("also no code"));
    assert!(err.has_code());
}

#[test]
fn test_is_type() {
    let err = Error::from_code(Code1::A);
    assert!(err.is_type::<Code1>());
    assert!(!err.is_type::<Code2>());
}

#[test]
fn test_is_value() {
    let err = Error::from_code(Code1::A);
    assert!(err.is(Code1::A));
    assert!(!err.is(Code1::B));
    assert!(!err.is(Code2::X));
    assert!(!err.is(Code2::Y));
}

#[test]
fn test_downcast_code() {
    let err = Error::from_code(Code1::B);

    let code1: Option<Code1> = err.downcast_code::<Code1>();
    assert_eq!(code1, Some(Code1::B));

    let code2: Option<Code2> = err.downcast_code::<Code2>();
    assert_eq!(code2, None);
}

#[test]
fn context_code_overwriting() {
    let err = Error::from_code(Code1::A).with_context(error_info!("some context"));
    assert!(err.is(Code1::A));

    let err = err.with_context(error_info!(Code2::X, "more context"));
    assert!(err.is(Code2::X));
    assert!(!err.is(Code1::A));
}
