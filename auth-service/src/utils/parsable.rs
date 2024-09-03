use color_eyre::eyre::{Error, Result};

pub trait Parsable {
    fn parse(s: &str) -> Result<Self>
    where
        Self: Sized;

    fn parse_or_error<E>(input: &str, map_err: impl FnOnce(Error) -> E) -> Result<Self, E>
    where
        Self: Sized,
    {
        Self::parse(input).map_err(map_err)
    }
}