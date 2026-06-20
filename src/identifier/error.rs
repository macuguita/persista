use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub enum IdentifierError {
    Namespace { namespace: String, path: String },
    Path { namespace: String, path: String },
    Format(String),
}

impl Display for IdentifierError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            IdentifierError::Namespace { namespace, path } => {
                write!(
                    f,
                    "Non [a-z0-9_.-] character in namespace of identifier: {namespace}:{path}"
                )
            }
            IdentifierError::Path { namespace, path } => {
                write!(
                    f,
                    "Non [a-z0-9/._-] character in path of location:  {namespace}:{path}"
                )
            }
            IdentifierError::Format(s) => {
                write!(f, "Invalid identifier: {s}")
            }
        }
    }
}

impl Error for IdentifierError {}
