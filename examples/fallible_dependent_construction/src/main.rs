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

self_cell!(
    struct NameCell {
        owner: String,

        #[covariant]
        dependent: Names,
    }

    impl {Debug}
);

fn names_from_str<'a>(s: &'a String) -> Result<Names, NameParseError> {
    let res: Result<Vec<_>, _> = s.split(" ").map(Name::try_from).collect();
    Ok(Names(res?))
}

fn process_input(input: String) {
    let names: Result<_, NameParseError> = NameCell::try_new(input.clone(), names_from_str);

    println!("'{}' -> {:?}", input, names);
}

fn main() {
    process_input("this is good".into());
    process_input("this is bad".into());
}
