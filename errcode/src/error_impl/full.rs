//! Error type implementation for when `alloc` is enabled.
//!
//! TODO: Document

use super::*;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

#[repr(transparent)]
pub struct ErrorImpl {
    inner: Box<ErrorImplInner>,
}
struct ErrorImplInner {
    original: ErrorSourceStep,
    steps: Vec<ErrorSourceStep>,
    current_code: Option<&'static ErrorCodeInfo>,
}
impl ErrorImplFunctions for ErrorImpl {
    type FrameIter = ();

    #[track_caller]
    fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl {
        ErrorImpl {
            inner: Box::new(ErrorImplInner {
                original: ErrorSourceStep {
                    static_info: source,
                    formatted_message: args.map(|x| x.to_string().into()),
                    location: Location::caller(),
                },
                steps: Vec::new(),
                current_code: match source {
                    ErrorOrigin::StaticOrigin(o) => o.error_code,
                    ErrorOrigin::TypeOrigin(_) => None,
                },
            }),
        }
    }

    #[track_caller]
    fn push_context(
        &mut self,
        source: &'static ErrorSourceStatic,
        args: Option<&Arguments<'_>>,
    ) {
        let step = ErrorSourceStep {
            static_info: ErrorOrigin::StaticOrigin(source),
            formatted_message: args.map(|x| x.to_string().into()),
            location: Location::caller(),
        };
        self.inner.steps.push(step);
        self.inner.current_code = source.error_code;
    }

    fn code(&self) -> Option<&'static ErrorCodeInfo> {
        self.inner.current_code
    }

    fn iter(&self) -> Self::FrameIter {
        todo!()
    }
}

struct ErrorSourceStep {
    static_info: ErrorOrigin,
    location: &'static Location<'static>,
    formatted_message: Option<Cow<'static, str>>,
}
