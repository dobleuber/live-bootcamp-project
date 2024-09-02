pub trait Parsable {
    type Error;

    fn parse(s: &str) -> Result<Self, Self::Error> where Self: Sized;

    fn parse_or_error<E>(input: &str, map_err: impl FnOnce(Self::Error) -> E) -> Result<Self, E> where Self: Sized {
        Self::parse(input).map_err(map_err)
    }
}