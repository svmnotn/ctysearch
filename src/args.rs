use crate::error::Error;
use std::path::PathBuf;

const USAGE_STRING: &str = r"Usage: ctysearch [-h|--help] [-l|--limit=NUM] [-f|--header=C_HEADER_FILE] FUNCTION

--help: Prints this message

--limit: Limit the number of results to return - Default: 10
--header: The C Header file to search through
";

/// Parsed Command Line Arguments
pub struct Args {
    /// Function type to search for in the header
    pub search: String,
    /// Limit the number of results to return. Default: 10
    pub limit: usize,
    /// The C Header file to search through
    pub header: PathBuf,
}

/// Parse the Command Line Arguments into [Args]
pub fn parse_args() -> Result<Args, Error> {
    use lexopt::prelude::*;

    let mut limit = 10;
    let mut header = None;
    let mut search = None;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('h') | Long("help") => {
                print!("{}", USAGE_STRING);
                std::process::exit(0);
            }
            Short('l') | Long("limit") => {
                limit = parser.value()?.parse()?;
            }
            Short('f') | Long("header") => {
                header.replace(parser.value()?.string()?.into());
            }
            Value(val) => {
                search = Some(val.string()?);
            }
            _ => Err(arg.unexpected())?,
        }
    }

    Ok(Args {
        search: search.ok_or(Error::MissingSearchArgument)?,
        header: header.ok_or(Error::MissingHeaderArgument)?,
        limit,
    })
}
