//! This module contains the internal guts of the error type.

use crate::error_code::ErrorCodeInfo;
use core::fmt::{Arguments, Display, Formatter};
use core::panic::Location;

/// Common trait for [`ErrorImpl`] variants.
pub trait ErrorImplFunctions: Clone {
    /// The iterator type used to iterate frames.
    type FrameIter<'a>: Iterator<Item = ErrorFrame> + 'a
    where Self: 'a;

    /// Creates a new error type.
    fn new(source: ErrorOrigin, args: Option<&Arguments<'_>>) -> ErrorImpl;

    /// Pushes a new context frame onto this type.
    fn push_context(&mut self, source: &'static ErrorInfoImpl, args: Option<&Arguments<'_>>);

    /// Gets the current error code of this type.
    fn code(&self) -> Option<&'static ErrorCodeInfo>;

    /// Returns an iterator of the frames in this error type.
    fn iter<'a>(&'a self) -> Self::FrameIter<'a>;
}

#[derive(Copy, Clone)]
#[repr(align(4))]
pub struct ErrorInfoImpl {
    pub error_code: Option<&'static ErrorCodeInfo>,
    pub message_static: StaticMessageInfo,
    pub location: Option<&'static DecodedLocation>,
}
impl ErrorInfoImpl {
    /// Returns `true` if the only information in this object is the error code itself.
    pub fn is_code_only(&self) -> bool {
        self.location.is_none()
    }
}

#[derive(Copy, Clone)]
pub enum StaticMessageInfo {
    Unformatted(&'static str),
    NoFormat(&'static str),
    None,
}

#[derive(Copy, Clone, Debug)]
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
    StaticOrigin(&'static ErrorInfoImpl),
    TypeOrigin(&'static str, Option<&'static ErrorInfoImpl>),
}

/// A decoded frame of error information, retrieved from an [`ErrorImpl`].
#[derive(Debug)]
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
                    "{} [{}::{}]",
                    info.message.unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) => {
                    write!(f, "<from type: {}> [{}::{}]", ty, info.type_name, info.variant_name)?
                }
                None => write!(f, "<from type: {}>", ty)?,
            },
            ErrorFrameData::NormalFrame(msg, info) => match info {
                Some(info) if info.message.is_some() && msg.is_none() => write!(
                    f,
                    "{} [{}::{}]",
                    info.message.unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) if msg.is_some() => write!(
                    f,
                    "{} [{}::{}]",
                    msg.as_ref().unwrap(),
                    info.type_name,
                    info.variant_name
                )?,
                Some(info) => write!(f, "[{}::{}]", info.type_name, info.variant_name)?,
                None if msg.is_some() => write!(f, "{}", msg.as_ref().unwrap())?,
                None => write!(f, "<internal error: no message or code given???>")?,
            },
        }

        if let Some(location) = &self.location {
            write!(f, " [at {}:{}:{}]", location.module, location.line, location.column)?;
        }

        Ok(())
    }
}

/// The data represented by an error frame.
#[derive(Debug)]
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
        data: Option<&'static ErrorInfoImpl>,
        formatted: Option<MessageContainer>,
    ) -> ErrorFrameData {
        ErrorFrameData::NormalFrame(
            formatted.or_else(|| match data.map(|x| x.message_static) {
                Some(StaticMessageInfo::Unformatted(msg)) => {
                    Some(MessageContainer::IncompleteStatic(msg))
                }
                Some(StaticMessageInfo::NoFormat(msg)) => Some(MessageContainer::Static(msg)),
                _ => None,
            }),
            data.and_then(|x| x.error_code),
        )
    }
}

#[derive(Debug)]
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
            write!(f, "<unformatted:> ")?;
        }
        write!(f, "{}", self.as_str())?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug)]
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
            InternalContextType::ErrorTypeConstructed => "<ErrorInfo constructed>",
            InternalContextType::OriginalTypeLost => "<original error type lost>",
            InternalContextType::FurtherFramesOmitted => "<some frames have been omitted>",
        }
    }
}

const _COMMON_CHECKS: () = {
    const fn test<T: ErrorImplFunctions + Sync + Send>() {}
    test::<ErrorImpl>();
};

// repr full
/////////////
#[cfg(feature = "repr_full")]
mod full;

#[cfg(feature = "repr_full")]
pub use full::ErrorImpl;

// repr unboxed
////////////////
#[cfg(any(
    feature = "repr_unboxed",
    feature = "repr_unboxed_location",
    not(any(feature = "repr_full"))
))]
mod unboxed;

#[cfg(any(
    feature = "repr_unboxed",
    feature = "repr_unboxed_location",
    not(any(feature = "repr_full"))
))]
pub use unboxed::ErrorImpl;

// fallback
////////////
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
