use color_eyre::eyre::{Error, Result};

pub trait Parsable: Sized {
    fn parse<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>;
    fn parse_or_error<E, S>(input: S, map_err: impl FnOnce(Error) -> E) -> Result<Self, E>
    where
        S: AsRef<str>,
    {
        Self::parse(input).map_err(map_err)
    }
}
