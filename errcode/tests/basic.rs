use errcode::{Error, ErrorCode, error_info};

#[derive(ErrorCode, Debug, PartialEq, Eq, Clone, Copy)]
pub enum TestCode {
    A,
    B,
    C,
}

#[test]
fn error_info() {
    let info = error_info!("hello, world!");
    let error = Error::from_info(info);
    assert!(error.to_string().starts_with("hello, world!"));
}

#[test]
fn error_with_format_args() {
    let a = 6;
    let error = Error::from_info(error_info!("test! {{ }} {a} {}", 7));
    let line = error.to_string();

    #[cfg(feature = "repr_full")]
    {
        assert!(line.starts_with("test! { } 6 7"), "Line: {line}");
    }

    #[cfg(not(feature = "repr_full"))]
    {
        assert!(line.starts_with("<unformatted:> \"test! {{ }} {a} {}\""), "Line: {line}");
    }
}

#[test]
#[cfg(not(feature = "repr_full"))]
fn formatted_and_unformatted() {
    let error = Error::from_info(error_info!("test! {{ }}"));
    assert!(error.to_string().starts_with("test! {{ }}"), "Line: {error}");

    let error = Error::from_info(error_info!("test! {{ }} {}", 3));
    assert!(
        error
            .to_string()
            .starts_with("<unformatted:> \"test! {{ }} {}\""),
        "Line: {error}"
    );
}

#[test]
fn error_with_code() {
    let error = Error::from_code(TestCode::A);
    assert!(error.is(TestCode::A));
    assert!(!error.is(TestCode::B));
    assert!(!error.is(TestCode::C));
}

#[test]
#[cfg(feature = "repr_full")]
fn error_with_context() {
    let error = Error::from_info(error_info!("root cause"))
        .with_context(error_info!(TestCode::B, "intermediate 1"))
        .with_context(error_info!(TestCode::A, "intermediate 2"))
        .with_context(error_info!("top level"));

    assert!(error.is(TestCode::A));

    let lines = error.to_string();
    let lines: Vec<_> = lines.lines().collect();

    let expected_lines = &[
        "top level",
        "caused by: intermediate 2 (TestCode::A)",
        "caused by: intermediate 1 (TestCode::B)",
        "caused by: root cause",
    ];
    for (i, line) in lines.iter().enumerate() {
        assert!(lines[i].trim().starts_with(expected_lines[i]), "Line {}: {}", i, line);
    }
}
