use std::convert::TryFrom;

use self_cell::self_cell;

#[derive(Debug)]
enum NameParseError {
    Banned,
    TooLong,
}

#[derive(Debug)]
struct Name<'a>(&'a str);

impl<'a> TryFrom<&'a str> for Name<'a> {
    type Error = NameParseError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        if s.len() > 100 {
            return Err(NameParseError::TooLong);
        }

        if s.contains("bad") {
            return Err(NameParseError::Banned);
        }

        Ok(Self(s))
    }
}

#[derive(Debug)]
struct Names<'a>(Vec<Name<'a>>);

impl<'a> TryFrom<&'a String> for Names<'a> {
    type Error = NameParseError;

    fn try_from(s: &'a String) -> Result<Self, Self::Error> {
        let res: Result<Vec<_>, _> = s.split(" ").map(Name::try_from).collect();
        Ok(Self(res?))
    }
}

self_cell!(
    struct NameCell {
        #[try_from]
        owner: String,

        #[covariant]
        dependent: Names,
    }

    impl {Debug}
);

fn main() {
    dbg!(NameCell::try_from("this is good".into()).unwrap());
    dbg!(NameCell::try_from("this is bad".into()).unwrap_err());
}
