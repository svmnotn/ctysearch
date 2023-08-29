use core::str::Utf8Error;
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
