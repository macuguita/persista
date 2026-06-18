use std::cmp::Ordering;
use std::fmt;

#[derive(Debug)]
pub enum IdentifierError {
    Namespace { namespace: String, path: String },
    Path { namespace: String, path: String },
    Format(String),
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierError::Namespace { namespace, path } => {
                write!(
                    f,
                    "Invalid namespace '{namespace}' in identifier: {namespace}:{path}"
                )
            }
            IdentifierError::Path { namespace, path } => {
                write!(f, "Invalid path '{path}' in identifier: {namespace}:{path}")
            }
            IdentifierError::Format(s) => {
                write!(f, "Invalid identifier: {s}")
            }
        }
    }
}

impl std::error::Error for IdentifierError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    pub fn new(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, IdentifierError> {
        let namespace = namespace.into();
        let path = path.into();

        if !Self::is_valid_namespace(&namespace) {
            return Err(IdentifierError::Namespace { namespace, path });
        }
        if !Self::is_valid_path(&path) {
            return Err(IdentifierError::Path { namespace, path });
        }

        Ok(Identifier { namespace, path })
    }

    pub fn parse(identifier: &str) -> Result<Self, IdentifierError> {
        match identifier.find(':') {
            Some(i) if i > 0 => Self::new(&identifier[..i], &identifier[i + 1..]),
            _ => Err(IdentifierError::Format(format!(
                "Invalid identifier: {identifier}"
            ))),
        }
    }

    pub fn try_parse(identifier: &str) -> Option<Self> {
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

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        let result = self.path.cmp(&other.path);
        if result != Ordering::Equal {
            result
        } else {
            self.namespace.cmp(&other.path)
        }
    }
}
