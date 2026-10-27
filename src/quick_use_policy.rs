// Interior storage may already have translated X into its accent-wall event (bit 12)
pub(crate) const ACCENT_WALL_EVENT: u64 = 1 << 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Use,
    Drop,
    Transfer,
}

// Never fall back to the first entry, planting/burying, or releasing a creature.
// Drop uses the native PutUnitIcon row, including its normal stack handling.
pub(crate) fn choose_command(commands: &[i32], action: Action) -> Option<usize> {
    if !(1..=7).contains(&commands.len()) {
        return None;
    }
    commands.iter().position(|command| match action {
        Action::Use => matches!(command, 0x05 | 0x06 | 0x25 | 0x26),
        Action::Drop => *command == 0x1e,
        Action::Transfer => *command == 0x31,
    })
}

pub(crate) fn storage_events_clear(events: u64) -> bool {
    events & !ACCENT_WALL_EVENT == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_matching_record() {
        assert_eq!(choose_command(&[0x1e, 0x05], Action::Use), Some(1));
        assert_eq!(choose_command(&[0x1e, 0x05], Action::Drop), Some(0));
        assert_eq!(choose_command(&[0x05], Action::Transfer), None);
        assert_eq!(choose_command(&[], Action::Use), None);
    }

    #[test]
    fn only_the_accent_wall_event_may_be_pending() {
        assert!(storage_events_clear(0));
        assert!(storage_events_clear(ACCENT_WALL_EVENT));
        assert!(!storage_events_clear(1));
    }
}
