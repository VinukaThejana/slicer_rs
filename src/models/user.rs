use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct UserId(pub String);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
