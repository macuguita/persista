// Remake from https://mcsrc.dev/1/26.1.2/net/minecraft/resources/Identifier

use std::{cmp::Ordering, fmt};

use lazy_static::lazy_static;

pub mod error;

use error::IdentifierError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}

impl<'a> Identifier<'a> {
    pub fn new(
        namespace: &'a str,
        path: &'a str,
    ) -> Result<Self, IdentifierError> {
        if !Self::is_valid_namespace(namespace) {
            return Err(IdentifierError::Namespace { namespace: namespace.to_string(), path: path.to_string() });
        }
        if !Self::is_valid_path(path) {
            return Err(IdentifierError::Path { namespace: namespace.to_string(), path: path.to_string() });
        }

        Ok(Identifier { namespace, path })
    }

    pub fn parse(identifier: &'a str) -> Result<Self, IdentifierError> {
        match identifier.find(':') {
            Some(i) if i > 0 => Self::new(&identifier[..i], &identifier[i + 1..]),
            _ => Err(IdentifierError::Format(format!(
                "Invalid identifier: {identifier}"
            ))),
        }
    }

    pub fn try_parse(identifier: &'a str) -> Option<Self> {
        Self::parse(identifier).ok()
    }

    fn is_valid_namespace(namespace: &str) -> bool {
        namespace != ".."
            && namespace.chars().all(|c| {
                c == '_' || c == '-' || c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.'
            })
    }

    fn is_valid_path(path: &str) -> bool {
        path.chars().all(|c| {
            c == '_'
                || c == '-'
                || c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || c == '/'
                || c == '.'
        })
    }
}

impl fmt::Display for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl PartialOrd for Identifier<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.path.cmp(&other.path);
        if result == Ordering::Equal {
            self.namespace.cmp(&other.namespace)
        } else {
            result
        }
    }
}

lazy_static! {
    pub static ref ENTITLEMENTS_KEY: Identifier<'static> = Identifier {
        namespace: "persista",
        path: "entitlements",
    };
}