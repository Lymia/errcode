//! This module contains the internal guts of the error type.

use crate::error_code::ErrorCodeInfo;
use core::fmt::{Arguments, Display, Formatter};
use core::panic::Location;

/// Common trait for [`ErrorImpl`] variants.
pub trait ErrorImplFunctions {
    /// The iterator type used to iterate frames.
    type FrameIter: Iterator<Item = ErrorFrame>;

    /// Creates a new error type.
    fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl;

    /// Pushes a new context frame onto this type.
    fn push_context(&mut self, source: &'static ErrorSourceStatic, args: Option<&Arguments<'_>>);

    /// Gets the current error code of this type.
    fn code(&self) -> Option<&'static ErrorCodeInfo>;

    /// Returns an iterator of the frames in this error type.
    fn iter(&self) -> Self::FrameIter;
}

/// Error type implementation for when `alloc` is enabled.
///
/// TODO: Document
#[cfg(feature = "repr_full")]
mod implementation {
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
}

/// Implementation for `repr_unboxed` and `repr_unboxed_location`.
///
/// TODO: Document
#[cfg(any(
    feature = "repr_unboxed",
    feature = "repr_unboxed_location",
    not(any(feature = "repr_full"))
))]
mod implementation {
    use super::*;
    use core::hint::unreachable_unchecked;
    use core::num::NonZeroUsize;

    pub struct ErrorImpl {
        origin_info: PackedOriginInfo,
        #[cfg(feature = "repr_unboxed_location")]
        original_location: &'static Location<'static>,
    }
    impl ErrorImplFunctions for ErrorImpl {
        type FrameIter = ErrorImplIter;

        #[cfg_attr(feature = "repr_unboxed_location", track_caller)]
        fn new(source: ErrorOrigin, _args: Option<&Arguments<'_>>) -> ErrorImpl {
            ErrorImpl {
                origin_info: PackedOriginInfo::for_origin(source),
                #[cfg(feature = "repr_unboxed_location")]
                original_location: Location::caller(),
            }
        }

        fn push_context(
            &mut self,
            source: &'static ErrorSourceStatic,
            _args: Option<&Arguments<'_>>,
        ) {
            self.origin_info = self.origin_info.with_context(source);
        }

        fn code(&self) -> Option<&'static ErrorCodeInfo> {
            self.origin_info.code()
        }

        fn iter(&self) -> Self::FrameIter {
            ErrorImplIter {
                phase: ErrorIterPhase::TypeContext,
                origin_info: self.origin_info,
                #[cfg(feature = "repr_unboxed_location")]
                original_location: self.original_location,
                #[cfg(not(feature = "repr_unboxed_location"))]
                original_location: None,
            }
        }
    }

    const TAG_STATIC_ORIGINAL: usize = 0;
    const TAG_STATIC_TYPE_ONLY: usize = 1;
    const TAG_STATIC_CONTEXT_ONLY: usize = 2;
    const TAG_MASK: usize = 0b11;

    const MAX_TYPE_LEN: usize = (usize::MAX >> 2) + 1;
    const OMITTED_BIT_MASK: usize = 0b1;

    #[derive(Copy, Clone)]
    struct PackedOriginInfo {
        /// A pointer to something that also stores a tag in the lower 2 bits. The fact that this
        /// is valid (due to alignment) is enforced by the const check in this module.
        ///
        /// This is nonzero to allow for niche optimization on the error type.
        ///
        /// For `TAG_STATIC_ORIGINAL` and `TAG_STATIC_CONTEXT_ONLY`, this is a pointer to an
        /// `ErrorSourceStatic`. It is enforced nonzero because pointers cannot be zero.
        ///
        /// For `TAG_STATIC_TYPE_ONLY`, this is the length of the type string, with the pointer
        /// itself stored in `additional`. It is enforced nonzero because the tag is nonzero.
        tag: NonZeroUsize,

        /// Additional tag information.
        ///
        /// For `TAG_STATIC_ORIGINAL` and `TAG_STATIC_CONTEXT_ONLY` this is a pointer to an
        /// `ErrorSourceStatic`, or zero (for no latest context). The lowest bit is used to store
        /// a flag for whether frames have been omitted from this context.
        ///
        /// For `TAG_STATIC_TYPE_ONLY`, this is the pointer to the string.
        additional: usize,
    }
    impl PackedOriginInfo {
        fn for_origin(e: ErrorOrigin) -> Self {
            unsafe {
                match e {
                    ErrorOrigin::StaticOrigin(ptr) | ErrorOrigin::TypeOrigin(_, Some(ptr)) => {
                        PackedOriginInfo {
                            tag: NonZeroUsize::new_unchecked(
                                (ptr as *const _ as usize) | TAG_STATIC_ORIGINAL,
                            ),
                            additional: 0,
                        }
                    }
                    ErrorOrigin::TypeOrigin(ptr, None) => {
                        assert!(ptr.len() < MAX_TYPE_LEN);
                        PackedOriginInfo {
                            tag: NonZeroUsize::new_unchecked(
                                (ptr.len() << 2) | TAG_STATIC_TYPE_ONLY,
                            ),
                            additional: ptr.as_ptr() as usize,
                        }
                    }
                }
            }
        }

        fn tag(&self) -> usize {
            self.tag.get() & TAG_MASK
        }

        fn with_context(mut self, source: &'static ErrorSourceStatic) -> Self {
            unsafe {
                match self.tag() {
                    TAG_STATIC_ORIGINAL | TAG_STATIC_CONTEXT_ONLY => {
                        if self.additional == 0 {
                            self.additional = source as *const _ as usize;
                            self
                        } else {
                            let original = &*(self.additional as *const ErrorSourceStatic);
                            if original.error_code.is_none() || source.error_code.is_some() {
                                self.additional = source as *const _ as usize;
                                self.additional |= OMITTED_BIT_MASK;
                                self
                            } else {
                                self.additional |= OMITTED_BIT_MASK;
                                self
                            }
                        }
                    }
                    TAG_STATIC_TYPE_ONLY => PackedOriginInfo {
                        tag: NonZeroUsize::new_unchecked(
                            (source as *const _ as usize) | TAG_STATIC_CONTEXT_ONLY,
                        ),
                        additional: 0,
                    },
                    _ => unreachable_unchecked(),
                }
            }
        }

        fn ty_name(&self) -> &'static str {
            unsafe {
                assert_eq!(self.tag(), TAG_STATIC_TYPE_ONLY);
                let ptr = self.additional as *const u8;
                let len = self.tag.get() >> 2;
                let slice = core::slice::from_raw_parts(ptr, len);
                core::str::from_utf8_unchecked(slice)
            }
        }

        fn context_first(&self) -> &'static ErrorSourceStatic {
            unsafe {
                assert!(self.tag() == TAG_STATIC_ORIGINAL || self.tag() == TAG_STATIC_CONTEXT_ONLY);
                &*((self.tag.get() & !TAG_MASK) as *const ErrorSourceStatic)
            }
        }

        fn context_second(&self) -> Option<&'static ErrorSourceStatic> {
            unsafe {
                assert!(self.tag() == TAG_STATIC_ORIGINAL || self.tag() == TAG_STATIC_CONTEXT_ONLY);
                if (self.additional & !OMITTED_BIT_MASK) == 0 {
                    None
                } else {
                    Some(&*((self.additional & !OMITTED_BIT_MASK) as *const ErrorSourceStatic))
                }
            }
        }

        fn has_omitted_context(self) -> bool {
            if self.tag() == TAG_STATIC_CONTEXT_ONLY || self.tag() == TAG_STATIC_ORIGINAL {
                self.additional & OMITTED_BIT_MASK == OMITTED_BIT_MASK
            } else {
                false
            }
        }

        fn code(&self) -> Option<&'static ErrorCodeInfo> {
            if self.tag() == TAG_STATIC_TYPE_ONLY {
                None
            } else {
                if let Some(context_second) = self.context_second() {
                    if context_second.error_code.is_some() {
                        return context_second.error_code;
                    }
                }
                self.context_first().error_code
            }
        }
    }

    pub struct ErrorImplIter {
        phase: ErrorIterPhase,
        origin_info: PackedOriginInfo,
        original_location: Option<&'static Location<'static>>,
    }
    #[derive(Copy, Clone, Eq, PartialEq)]
    enum ErrorIterPhase {
        TypeContext,
        LocationMismatchFrame,
        FirstContext,
        LastContext,
        FramesOmitted,
        Ended,
    }
    impl Iterator for ErrorImplIter {
        type Item = ErrorFrame;
        fn next(&mut self) -> Option<Self::Item> {
            let tag = self.origin_info.tag();

            // emits a type context frame if we are a static type node.
            if self.phase == ErrorIterPhase::TypeContext {
                self.phase = ErrorIterPhase::LocationMismatchFrame;

                if tag == TAG_STATIC_TYPE_ONLY {
                    // we have a static type node!
                    // we know it's ended at this point, save some time
                    self.phase = ErrorIterPhase::Ended;
                    return Some(ErrorFrame {
                        data: ErrorFrameData::TypeFrame(self.origin_info.ty_name(), None),
                        location: self.original_location.map(DecodedLocation::from),
                    });
                } else if tag == TAG_STATIC_CONTEXT_ONLY {
                    // we have a former type node that we appended context to
                    return Some(ErrorFrame {
                        data: ErrorFrameData::InternalContext(
                            InternalContextType::OriginalTypeLost,
                        ),
                        location: self.original_location.map(DecodedLocation::from),
                    });
                }
            }

            // emits a "location mismatch" frame if the error construction is far from the first
            // context's error frame
            if self.phase == ErrorIterPhase::LocationMismatchFrame {
                self.phase = ErrorIterPhase::FirstContext;
                if tag == TAG_STATIC_ORIGINAL {
                    let context_first = self.origin_info.context_first();
                    if let Some(location_a) = context_first.location
                        && let Some(location_b) = self.original_location
                    {
                        if !location_a.is_same(location_b.into()) {
                            return Some(ErrorFrame {
                                data: ErrorFrameData::InternalContext(
                                    InternalContextType::ErrorTypeConstructed,
                                ),
                                location: Some(*location_a),
                            });
                        }
                    }
                }
            }

            // returns the first context frame
            if self.phase == ErrorIterPhase::FirstContext {
                self.phase = ErrorIterPhase::LastContext;
                if tag == TAG_STATIC_ORIGINAL || tag == TAG_STATIC_CONTEXT_ONLY {
                    let context_first = self.origin_info.context_first();
                    let location = if tag == TAG_STATIC_ORIGINAL {
                        self.original_location.map(DecodedLocation::from)
                    } else {
                        context_first.location.map(|x| *x)
                    };
                    return Some(ErrorFrame {
                        data: ErrorFrameData::decode_static(context_first, None),
                        location,
                    });
                }
            }

            // returns the last context frame
            if self.phase == ErrorIterPhase::LastContext {
                self.phase = ErrorIterPhase::FramesOmitted;
                if tag == TAG_STATIC_ORIGINAL || tag == TAG_STATIC_CONTEXT_ONLY {
                    if let Some(context_second) = self.origin_info.context_second() {
                        return Some(ErrorFrame {
                            data: ErrorFrameData::decode_static(context_second, None),
                            location: context_second.location.map(|x| *x),
                        });
                    }
                }
            }

            // returns the frames ommitted message, if needed
            if self.phase == ErrorIterPhase::FramesOmitted {
                self.phase = ErrorIterPhase::Ended;
                if tag == TAG_STATIC_ORIGINAL || tag == TAG_STATIC_CONTEXT_ONLY {
                    if self.origin_info.has_omitted_context() {
                        return Some(ErrorFrame {
                            data: ErrorFrameData::InternalContext(
                                InternalContextType::FurtherFramesOmitted,
                            ),
                            location: None,
                        });
                    }
                }
            }

            None
        }
    }

    const _CHECK_REQUIRED_ALIGNMENT: () = {
        let required_alignment = 4;
        assert!(align_of::<&'static ErrorSourceStatic>() >= required_alignment);
        assert!(align_of::<&'static str>() >= required_alignment);
    };
}

#[cfg(any(
    all(feature = "repr_full", feature = "repr_unboxed_location"),
    all(feature = "repr_full", feature = "repr_unboxed"),
    all(feature = "repr_unboxed_location", feature = "repr_unboxed"),
))]
const _: () = {
    compile_error!(
        "You may only use one of `repr_full`, `repr_unboxed` or `repr_unboxed_location`."
    );
};

pub use implementation::ErrorImpl;

pub struct ErrorSourceStatic {
    pub error_code: Option<&'static ErrorCodeInfo>,
    pub message_static: Option<&'static str>,
    pub is_static_message_incomplete: bool,
    pub location: Option<&'static DecodedLocation>,
}
impl ErrorSourceStatic {
    /// Returns `true` if the only information in this object is the error code itself.
    pub fn is_code_only(&self) -> bool {
        self.location.is_none()
    }
}

#[derive(Copy, Clone)]
pub struct DecodedLocation {
    pub module: &'static str,
    pub line: u32,
    pub column: u32,
}
impl From<&'static Location<'static>> for DecodedLocation {
    fn from(value: &'static Location<'static>) -> Self {
        DecodedLocation { module: value.file(), line: value.line(), column: value.column() }
    }
}
impl DecodedLocation {
    fn is_same(&self, other: DecodedLocation) -> bool {
        self.module == other.module && self.line == other.line
    }
}

#[derive(Copy, Clone)]
pub enum ErrorOrigin {
    StaticOrigin(&'static ErrorSourceStatic),
    TypeOrigin(&'static str, Option<&'static ErrorSourceStatic>),
}

/// A decoded frame of error information, retrieved from an [`ErrorImpl`].
pub struct ErrorFrame {
    data: ErrorFrameData,
    location: Option<DecodedLocation>,
}
impl Display for ErrorFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self.data {
            ErrorFrameData::InternalContext(ctx) => write!(f, "{}", ctx.message())?,
            ErrorFrameData::TypeFrame(ty, info) => match info {
                Some(info) if info.message.is_some() => write!(
                    f,
                    "{} ({}::{})",
                    info.message.unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) => write!(
                    f,
                    "<converted from type: {}> ({}::{})",
                    ty, info.type_name, info.variant_name
                )?,
                None => write!(f, "<converted from type: {}>", ty)?,
            },
            ErrorFrameData::NormalFrame(msg, info) => match info {
                Some(info) if info.message.is_some() && msg.is_none() => write!(
                    f,
                    "{} ({}::{})",
                    info.message.unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) if msg.is_some() => write!(
                    f,
                    "{} ({}::{})",
                    msg.as_ref().unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) => {
                    write!(f, "<no message given> ({}::{})", info.type_name, info.variant_name)?
                }
                None if msg.is_some() => write!(f, "{}", msg.as_ref().unwrap())?,
                None => write!(f, "<no message or code given???>")?,
            },
        }

        Ok(())
    }
}

/// The data represented by an error frame.
enum ErrorFrameData {
    /// Used to represent a frame of context that doesn't "really" exist, but should be reported
    /// to the user anyway.
    InternalContext(InternalContextType),

    /// Used to represent a frame where the only information known is the type of a converted
    /// error.
    TypeFrame(&'static str, Option<&'static ErrorCodeInfo>),

    /// A normal frame that contains a message, an error code or both.
    NormalFrame(Option<MessageContainer>, Option<&'static ErrorCodeInfo>),
}
impl ErrorFrameData {
    fn decode_static(
        data: &'static ErrorSourceStatic,
        formatted: Option<MessageContainer>,
    ) -> ErrorFrameData {
        ErrorFrameData::NormalFrame(
            formatted.or_else(|| {
                data.message_static.map(|msg| {
                    if data.is_static_message_incomplete {
                        MessageContainer::IncompleteStatic(msg)
                    } else {
                        MessageContainer::Static(msg)
                    }
                })
            }),
            data.error_code,
        )
    }
}

enum MessageContainer {
    /// Used to represent a static message given by the user.
    Static(&'static str),

    /// Used to represent a static message that couldn't be formatted.
    IncompleteStatic(&'static str),

    #[cfg(feature = "repr_full")]
    Formatted(alloc::string::String),
}
impl MessageContainer {
    fn as_str(&self) -> &str {
        match self {
            MessageContainer::Static(v) => v,
            MessageContainer::IncompleteStatic(v) => v,
            #[cfg(feature = "repr_full")]
            MessageContainer::Formatted(v) => v.as_str(),
        }
    }

    fn is_incomplete(&self) -> bool {
        matches!(self, MessageContainer::IncompleteStatic(_))
    }
}
impl Display for MessageContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.is_incomplete() {
            write!(f, "<unformatted message:>")?;
        }
        write!(f, "{}", self.as_str())?;
        Ok(())
    }
}

enum InternalContextType {
    /// Used to represent when an error type is constructed at a significantly different location
    /// from the `Location` stored in the error type.
    ///
    /// This often implies that `#[track_caller]` was used, though it could also just be a code
    /// style that broke the `error!` macro call into a different line.
    ErrorTypeConstructed,

    /// Used to represent when the original type the error type was converted from was lost. This
    /// occurs on the compact representation used when `alloc` isn't set fairly often.
    OriginalTypeLost,

    /// Used to note to the user that additional frames of context may have been omitted from the
    /// trace. This occurs on the compact representation used when `alloc` isn't set.
    FurtherFramesOmitted,
}
impl InternalContextType {
    fn message(&self) -> &'static str {
        match self {
            InternalContextType::ErrorTypeConstructed => "<ErrorInfo constructed at:>",
            InternalContextType::OriginalTypeLost => "<original error type lost>",
            InternalContextType::FurtherFramesOmitted => "<some frames have been omitted>",
        }
    }
}

const _COMMON_CHECKS: () = {
    const fn test<T: ErrorImplFunctions>() {}
    test::<ErrorImpl>();
};
