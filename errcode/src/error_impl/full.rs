//! Error type implementation for when `alloc` is enabled.
//!
//! TODO: Document

use super::*;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

#[repr(transparent)]
#[derive(Clone)]
pub struct ErrorImpl {
    inner: Box<ErrorImplInner>,
}
struct ErrorImplInner {
    steps: Vec<ErrorSourceStep>,
    current_code: Option<&'static ErrorCodeInfo>,
}
impl ErrorImplFunctions for ErrorImpl {
    type FrameIter<'a> = ErrorImplIter<'a>;

    #[track_caller]
    fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl {
        ErrorImpl {
            inner: Box::new(ErrorImplInner {
                steps: vec![ErrorSourceStep {
                    static_info: source,
                    formatted_message: args.map(|x| x.to_string().into()),
                    location: Location::caller(),
                }],
                current_code: match source {
                    ErrorOrigin::StaticOrigin(o) => o.error_code,
                    ErrorOrigin::TypeOrigin(_, Some(code)) => code.error_code,
                    _ => None,
                },
            }),
        }
    }

    #[track_caller]
    fn push_context(&mut self, source: &'static ErrorSourceStatic, args: Option<&Arguments<'_>>) {
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

    fn iter(&self) -> Self::FrameIter<'_> {
        ErrorImplIter {
            underlying: &self.inner,
            idx: 0,
            phase: FrameLoopPhase::LocationMismatchFrame,
        }
    }
}

struct ErrorSourceStep {
    static_info: ErrorOrigin,
    location: &'static Location<'static>,
    formatted_message: Option<Cow<'static, str>>,
}

pub struct ErrorImplIter<'a> {
    underlying: &'a ErrorImplInner,
    idx: usize,
    phase: FrameLoopPhase,
}
#[derive(Copy, Clone, Eq, PartialEq)]
enum FrameLoopPhase {
    LocationMismatchFrame,
    Context,
    Ended,
}
impl Iterator for ErrorImplIter<'_> {
    type Item = ErrorFrame;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.underlying.steps.len() {
            let frame = &self.underlying.steps[self.idx];

            if self.phase == FrameLoopPhase::LocationMismatchFrame {
                self.phase = FrameLoopPhase::Context;

                // TODO: Location mismatch
            }

            if self.phase == FrameLoopPhase::Context {
                self.phase = FrameLoopPhase::Ended;

                // TODO: Context
            }

            self.idx += 1;
            self.phase = FrameLoopPhase::Context;
        }
        None
    }
}
