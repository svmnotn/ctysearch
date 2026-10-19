use core::fmt;
use core::str::Utf8Error;
use lexopt::Error as LexoptError;
use std::error::Error as StdError;
use std::io::Error as IoError;
use tree_sitter::{LanguageError, QueryError};

#[derive(Debug)]
pub enum Error {
    FailedToSetParserLanguage(LanguageError),
    FailedToParse,
    FailedToCreateQuery(QueryError),
    FailedToReadUtf8(Utf8Error),
    FoundReturnBeforeDeclaration,
    FoundNameBeforeDeclaration,
    FoundParameterBeforeDeclaration,
    NeverFoundAReturnType,
    NeverFoundAName,
    MissingSearchArgument,
    MissingHeaderArgument,
    Argument(LexoptError),
    Io(IoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FailedToSetParserLanguage(e) => {
                write!(f, "failed to set parser language: {e}")
            }
            Error::FailedToParse => write!(f, "failed to parse"),
            Error::FailedToCreateQuery(e) => write!(f, "failed to create query: {e}"),
            Error::FailedToReadUtf8(e) => write!(f, "failed to read utf8: {e}"),
            Error::FoundReturnBeforeDeclaration => {
                write!(f, "found return type before declaration")
            }
            Error::FoundNameBeforeDeclaration => write!(f, "found name before declaration"),
            Error::FoundParameterBeforeDeclaration => {
                write!(f, "found parameter before declaration")
            }
            Error::NeverFoundAReturnType => write!(f, "never found a return type"),
            Error::NeverFoundAName => write!(f, "never found a name"),
            Error::MissingSearchArgument => write!(f, "missing search argument"),
            Error::MissingHeaderArgument => write!(f, "missing header argument"),
            Error::Argument(e) => write!(f, "argument error: {e}"),
            Error::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::FailedToSetParserLanguage(e) => Some(e),
            Error::FailedToCreateQuery(e) => Some(e),
            Error::FailedToReadUtf8(e) => Some(e),
            Error::Argument(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<LanguageError> for Error {
    fn from(value: LanguageError) -> Self {
        Error::FailedToSetParserLanguage(value)
    }
}

impl From<QueryError> for Error {
    fn from(value: QueryError) -> Self {
        Error::FailedToCreateQuery(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Error::FailedToReadUtf8(value)
    }
}

impl From<LexoptError> for Error {
    fn from(value: LexoptError) -> Self {
        Error::Argument(value)
    }
}

impl From<IoError> for Error {
    fn from(value: IoError) -> Self {
        Error::Io(value)
    }
}
