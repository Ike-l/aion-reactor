use crate::id::Id;

#[derive(Debug, small_iter_fields::IterFields, small_iter_fields::HashFields, Hash, Eq, PartialEq, Clone, Copy)]
pub enum TransitionPhase {
    Enter,
    Compute,
    Exit,
}

impl TransitionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransitionPhase::Enter => "Enter",
            TransitionPhase::Compute => "Compute",
            TransitionPhase::Exit => "Exit",
        }
    }
}

impl From<TransitionPhase> for Id {
    fn from(value: TransitionPhase) -> Self {
        Id(value.as_str().to_string())
    }
}