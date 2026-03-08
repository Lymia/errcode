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
#[derive(Clone)]
struct ErrorImplInner {
    steps: Vec<ErrorSourceStep>,
    current_code: Option<&'static ErrorCodeInfo>,
}
impl ErrorImplFunctions for ErrorImpl {
    type FrameIter<'a> = ErrorImplIter<'a>;

    #[track_caller]
    #[inline(never)]
    fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl {
        ErrorImpl {
            inner: Box::new(ErrorImplInner {
                steps: vec![ErrorSourceStep {
                    static_info: source,
                    formatted_message: format_args(args),
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
    #[inline(never)]
    fn push_context(&mut self, source: &'static ErrorInfoImpl, args: Option<&Arguments<'_>>) {
        let step = ErrorSourceStep {
            static_info: ErrorOrigin::StaticOrigin(source),
            formatted_message: format_args(args),
            location: Location::caller(),
        };
        self.inner.steps.push(step);
        self.inner.current_code = source.error_code;
    }

    #[inline(always)]
    fn code(&self) -> Option<&'static ErrorCodeInfo> {
        self.inner.current_code
    }

    fn iter(&self) -> Self::FrameIter<'_> {
        ErrorImplIter {
            underlying: &self.inner,
            idx: self.inner.steps.len(),
            phase: FrameLoopPhase::Context,
        }
    }
}

fn format_args(args: Option<&Arguments>) -> Option<Cow<'static, str>> {
    if let Some(args) = args {
        if let Some(str) = args.as_str() {
            Some(Cow::Borrowed(str))
        } else {
            Some(args.to_string().into())
        }
    } else {
        None
    }
}

#[derive(Clone)]
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
    Context,
    LocationMismatchFrame,
    Ended,
}
impl Iterator for ErrorImplIter<'_> {
    type Item = ErrorFrame;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx > 0 {
            let frame = &self.underlying.steps[self.idx - 1];

            if self.phase == FrameLoopPhase::Context {
                self.phase = FrameLoopPhase::LocationMismatchFrame;

                let info = match frame.static_info {
                    ErrorOrigin::StaticOrigin(info) => Some(info),
                    ErrorOrigin::TypeOrigin(_, info) => info,
                };
                return Some(ErrorFrame {
                    data: match &frame.formatted_message {
                        None => match frame.static_info {
                            ErrorOrigin::StaticOrigin(origin) => {
                                ErrorFrameData::decode_static(Some(origin), None)
                            }
                            ErrorOrigin::TypeOrigin(ty, origin) => {
                                ErrorFrameData::TypeFrame(ty, origin.and_then(|x| x.error_code))
                            }
                        },
                        Some(Cow::Borrowed(str)) => {
                            ErrorFrameData::decode_static(info, Some(MessageContainer::Static(str)))
                        }
                        Some(Cow::Owned(str)) => ErrorFrameData::decode_static(
                            info,
                            Some(MessageContainer::Formatted(str.clone())),
                        ),
                    },
                    location: Some(frame.location.into()),
                });
            }

            if self.phase == FrameLoopPhase::LocationMismatchFrame {
                self.phase = FrameLoopPhase::Ended;

                let location = DecodedLocation::from(frame.location);
                let origin = match &frame.static_info {
                    ErrorOrigin::StaticOrigin(origin) => origin.location,
                    ErrorOrigin::TypeOrigin(_, origin) => origin.and_then(|x| x.location),
                };
                if let Some(origin) = origin {
                    if !origin.is_same(location) {
                        return Some(ErrorFrame {
                            data: ErrorFrameData::InternalContext(
                                InternalContextType::ErrorTypeConstructed,
                            ),
                            location: Some(*origin),
                        });
                    }
                }
            }

            self.idx -= 1;
            self.phase = FrameLoopPhase::Context;
        }
        None
    }
}
