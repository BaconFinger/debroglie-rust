/// Indicates whether something has been fully resolved or not.
/// This is the return code for many functions, but can also
/// describe the state of individual locations in a generated output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Resolution {
    /// The operation has successfully completed and a value is known.
    Decided = 0,
    /// The operation has not yet found a value
    Undecided = -1,
    /// It was not possible to find a successful value.
    Contradiction = -2,
}

impl Resolution {
    /// Returns true if the resolution is Decided
    pub fn is_decided(self) -> bool {
        self == Resolution::Decided
    }

    /// Returns true if the resolution is Undecided
    pub fn is_undecided(self) -> bool {
        self == Resolution::Undecided
    }

    /// Returns true if the resolution is Contradiction
    pub fn is_contradiction(self) -> bool {
        self == Resolution::Contradiction
    }

    /// Returns true if the resolution is not Decided (either Undecided or Contradiction)
    pub fn is_unresolved(self) -> bool {
        self != Resolution::Decided
    }

    /// Returns true if the resolution represents a failure state (Contradiction)
    pub fn is_failure(self) -> bool {
        self == Resolution::Contradiction
    }

    /// Returns true if the resolution represents a success state (Decided)
    pub fn is_success(self) -> bool {
        self == Resolution::Decided
    }

    /// Returns true if the resolution is still in progress (Undecided)
    pub fn is_pending(self) -> bool {
        self == Resolution::Undecided
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Resolution::Decided => write!(f, "Decided"),
            Resolution::Undecided => write!(f, "Undecided"),
            Resolution::Contradiction => write!(f, "Contradiction"),
        }
    }
}

// Conversion from integers
impl TryFrom<i8> for Resolution {
    type Error = &'static str;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Resolution::Decided),
            -1 => Ok(Resolution::Undecided),
            -2 => Ok(Resolution::Contradiction),
            _ => Err("Invalid value for Resolution"),
        }
    }
}

// Conversion to integers
impl From<Resolution> for i8 {
    fn from(resolution: Resolution) -> Self {
        resolution as i8
    }
}

// Useful for Result-like operations
impl Resolution {
    /// Converts the Resolution to a Result, treating Decided as Ok and others as Err
    pub fn to_result(self) -> Result<(), Resolution> {
        match self {
            Resolution::Decided => Ok(()),
            other => Err(other),
        }
    }

    /// Creates a Resolution from a Result
    /// Note: This can only produce Decided (for Ok) or Contradiction (for Err).
    /// There's no way to represent Undecided from a binary Result type.
    pub fn from_result<T, E>(result: Result<T, E>) -> Self {
        match result {
            Ok(_) => Resolution::Decided,
            Err(_) => Resolution::Contradiction,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_values() {
        assert_eq!(Resolution::Decided as i8, 0);
        assert_eq!(Resolution::Undecided as i8, -1);
        assert_eq!(Resolution::Contradiction as i8, -2);
    }

    #[test]
    fn test_is_methods() {
        assert!(Resolution::Decided.is_decided());
        assert!(!Resolution::Undecided.is_decided());
        assert!(!Resolution::Contradiction.is_decided());

        assert!(Resolution::Undecided.is_undecided());
        assert!(!Resolution::Decided.is_undecided());
        assert!(!Resolution::Contradiction.is_undecided());

        assert!(Resolution::Contradiction.is_contradiction());
        assert!(!Resolution::Decided.is_contradiction());
        assert!(!Resolution::Undecided.is_contradiction());
    }

    #[test]
    fn test_state_methods() {
        assert!(Resolution::Decided.is_success());
        assert!(!Resolution::Undecided.is_success());
        assert!(!Resolution::Contradiction.is_success());

        assert!(!Resolution::Decided.is_failure());
        assert!(!Resolution::Undecided.is_failure());
        assert!(Resolution::Contradiction.is_failure());

        assert!(!Resolution::Decided.is_pending());
        assert!(Resolution::Undecided.is_pending());
        assert!(!Resolution::Contradiction.is_pending());

        assert!(!Resolution::Decided.is_unresolved());
        assert!(Resolution::Undecided.is_unresolved());
        assert!(Resolution::Contradiction.is_unresolved());
    }

    #[test]
    fn test_conversions() {
        assert_eq!(Resolution::try_from(0), Ok(Resolution::Decided));
        assert_eq!(Resolution::try_from(-1), Ok(Resolution::Undecided));
        assert_eq!(Resolution::try_from(-2), Ok(Resolution::Contradiction));
        assert!(Resolution::try_from(1).is_err());

        assert_eq!(i8::from(Resolution::Decided), 0);
        assert_eq!(i8::from(Resolution::Undecided), -1);
        assert_eq!(i8::from(Resolution::Contradiction), -2);
    }

    #[test]
    fn test_result_conversion() {
        assert_eq!(Resolution::Decided.to_result(), Ok(()));
        assert_eq!(Resolution::Undecided.to_result(), Err(Resolution::Undecided));
        assert_eq!(Resolution::Contradiction.to_result(), Err(Resolution::Contradiction));

        // from_result can only produce Decided or Contradiction (not Undecided)
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("error");

        assert_eq!(Resolution::from_result(ok_result), Resolution::Decided);
        assert_eq!(Resolution::from_result(err_result), Resolution::Contradiction);

        // Additional concrete examples
        assert_eq!(Resolution::from_result(Ok::<i32, &str>(100)), Resolution::Decided);
        assert_eq!(Resolution::from_result(Err::<i32, &str>("failed")), Resolution::Contradiction);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Resolution::Decided), "Decided");
        assert_eq!(format!("{}", Resolution::Undecided), "Undecided");
        assert_eq!(format!("{}", Resolution::Contradiction), "Contradiction");
    }
}