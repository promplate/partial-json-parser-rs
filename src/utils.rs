#[derive(Default, Clone, Debug, PartialEq)]
pub enum RunState {
    #[default]
    None,
    Error(String),
    Success,
}

impl RunState {
    pub fn is_not_none(&self) -> bool {
        *self != RunState::None
    }

    pub fn is_none(&self) -> bool {
        !self.is_not_none()
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
}

impl<T> From<Result<T, String>> for RunState {
    fn from(value: Result<T, String>) -> Self {
        match value {
            Ok(_) => RunState::Success,
            Err(s) => RunState::Error(s),
        }
    }
}

pub fn add_title(s: &str) -> String {
    format!(
        "################################ {} ################################\n",
        s
    )
}
