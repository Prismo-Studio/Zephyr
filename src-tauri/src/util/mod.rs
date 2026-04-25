use std::cmp::Ordering;

use itertools::Itertools;

pub mod cmd;
pub mod error;
pub mod fs;
pub mod path;
pub mod process;
pub mod window;
pub mod keys;
pub mod zip;

pub fn cmp_ignore_case(a: impl AsRef<str>, b: impl AsRef<str>) -> Ordering {
    a.as_ref()
        .chars()
        .flat_map(char::to_lowercase)
        .zip_longest(b.as_ref().chars().flat_map(char::to_lowercase))
        .map(|ab| match ab {
            itertools::EitherOrBoth::Left(_) => Ordering::Greater,
            itertools::EitherOrBoth::Right(_) => Ordering::Less,
            itertools::EitherOrBoth::Both(a, b) => a.cmp(&b),
        })
        .find(|&ordering| ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_ignore_case_equal() {
        assert_eq!(cmp_ignore_case("abc", "ABC"), Ordering::Equal);
        assert_eq!(cmp_ignore_case("Hello", "hello"), Ordering::Equal);
    }

    #[test]
    fn cmp_ignore_case_less() {
        assert_eq!(cmp_ignore_case("abc", "abd"), Ordering::Less);
    }

    #[test]
    fn cmp_ignore_case_greater() {
        assert_eq!(cmp_ignore_case("b", "a"), Ordering::Greater);
    }

    #[test]
    fn cmp_ignore_case_different_lengths() {
        assert_eq!(cmp_ignore_case("ab", "abc"), Ordering::Less);
        assert_eq!(cmp_ignore_case("abc", "ab"), Ordering::Greater);
    }
}
