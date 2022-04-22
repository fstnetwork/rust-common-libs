#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ShutdownState {
    Initial,
    WaitForSignal,
    ShuttingDown,
    Aborting,
}

impl Default for ShutdownState {
    #[inline]
    fn default() -> Self { Self::Initial }
}

impl Iterator for ShutdownState {
    type Item = Self;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Self::Initial => {
                *self = Self::WaitForSignal;
            }
            Self::WaitForSignal => {
                *self = Self::ShuttingDown;
            }
            Self::ShuttingDown => {
                *self = Self::Aborting;
            }
            Self::Aborting => return None,
        }

        Some(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::ShutdownState;

    #[test]
    fn test_new() {
        let shutdown_state = ShutdownState::default();
        assert_eq!(shutdown_state, ShutdownState::Initial);
    }

    #[test]
    fn test_default() {
        let shutdown_state = ShutdownState::default();
        assert_eq!(shutdown_state, ShutdownState::Initial);
    }

    #[test]
    fn test_iterator() {
        let mut shutdown_state = ShutdownState::default();
        assert_eq!(shutdown_state, ShutdownState::Initial);
        assert_eq!(shutdown_state.next(), Some(ShutdownState::WaitForSignal));
        assert_eq!(shutdown_state.next(), Some(ShutdownState::ShuttingDown));
        assert_eq!(shutdown_state.next(), Some(ShutdownState::Aborting));
        assert_eq!(shutdown_state.next(), None);
    }
}
