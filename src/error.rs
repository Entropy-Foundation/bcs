// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use core::fmt;
use serde::{de, ser};

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Eof,
    #[cfg(feature = "std")]
    Io(String),
    ExceededMaxLen(usize),
    ExceededContainerDepthLimit(&'static str),
    ExpectedBoolean,
    ExpectedMapKey,
    ExpectedMapValue,
    NonCanonicalMap,
    ExpectedOption,
    SerdeCustom,
    MissingLen,
    NotSupported(&'static str),
    RemainingInput,
    Utf8,
    NonCanonicalUleb128Encoding,
    IntegerOverflowDuringUleb128Decoding,
    CollectStrError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        write!(
            f,
            "{}",
            match self {
                Eof => "unexpected end of input",
                #[cfg(feature = "std")]
                Io(s) => s,
                ExceededMaxLen(_) => "exceeded max sequence length",
                ExceededContainerDepthLimit(_) => {
                    "exceeded max container depth while entering"
                }
                ExpectedBoolean => "expected boolean",
                ExpectedMapKey => "expected map key",
                ExpectedMapValue => "expected map value",
                NonCanonicalMap => {
                    "keys of serialized maps must be unique and in increasing order"
                }
                ExpectedOption => "expected option type",
                SerdeCustom => "Serde Custom Error",
                MissingLen => "sequence missing length",
                NotSupported(_) => "not supported",
                RemainingInput => "remaining input",
                Utf8 => "malformed utf8",
                NonCanonicalUleb128Encoding => "ULEB128 encoding was not minimal in size",
                IntegerOverflowDuringUleb128Decoding => {
                    "ULEB128-encoded integer did not fit in the target size"
                }
                CollectStrError => "Error while processing `collect_str` during serialization",
            },
        )
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Error::SerdeCustom
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Error::SerdeCustom
    }
}

impl serde::ser::StdError for Error {}
