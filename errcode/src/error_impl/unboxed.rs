//! Implementation for `repr_unboxed` and `repr_unboxed_location`.
//! 
//! TODO: Implement

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
