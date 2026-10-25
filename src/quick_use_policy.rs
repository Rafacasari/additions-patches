// Native FUN_7103b17ab0 maps HID ZR (bit 9) to normalized bit 5.
pub(crate) const ZR: u32 = 1 << 5;
// The same native converter maps HID ZL (bit 8) to normalized bit 2.
pub(crate) const ZL: u32 = 1 << 2;
pub(crate) const X: u32 = 1 << 3;

pub(crate) fn storage_overlay_allowed(
    state: i32,
    pending: [i32; 2],
    events: u64,
    enabled: bool,
    pressed: u32,
) -> bool {
    // Interior storage may already have translated X into its accent-wall
    // event. Consume that event with X; other native actions retain priority.
    state == 3 && pending == [-1; 2] && events & !(1 << 12) == 0 && enabled && pressed == X
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Use,
    Drop,
    Transfer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Pending {
    pub pocket: usize,
    pub selector: usize,
    pub player: u8,
    pub slot: u8,
    pub action: Action,
}

#[derive(Clone, Copy)]
pub(crate) struct Pocket {
    pub address: usize,
    pub selector: usize,
    pub player: u8,
    pub state: i32,
    pub mode: i32,
    pub pending: [i32; 2],
    pub slot: u8,
    pub capacity: u32,
    pub blocked_slots: u64,
    pub events: u16,
    pub input_enabled: bool,
}

impl Pocket {
    pub fn start(self, pressed: u32, at_home: bool) -> Option<Pending> {
        let action = match pressed {
            ZR => Action::Use,
            ZL if at_home => Action::Transfer,
            ZL => Action::Drop,
            _ => return None,
        };
        (self.state == 3
            && self.mode == 0
            && self.pending == [-1; 2]
            && self.events == 0
            && self.input_enabled
            && self.player < 8
            && self.address != 0
            && self.selector != 0
            && (1..=40).contains(&self.capacity)
            && u32::from(self.slot) < self.capacity
            && self.blocked_slots & (1u64 << self.slot) == 0)
            .then_some(Pending {
                pocket: self.address,
                selector: self.selector,
                player: self.player,
                slot: self.slot,
                action,
            })
    }

    pub fn owns(self, request: Pending, selector: usize) -> bool {
        self.address == request.pocket
            && self.selector == request.selector
            && selector == request.selector
            && self.player == request.player
            && self.slot == request.slot
            && self.state == 5
            && self.mode == 0
            && self.pending == [-1; 2]
    }
}

// Never fall back to the first entry, planting/burying, or releasing a creature.
// Drop uses the native PutUnitIcon row, including its normal stack handling.
pub(crate) fn choose_button(commands: &[i32], action: Action) -> Option<u8> {
    if !(1..=7).contains(&commands.len()) {
        return None;
    }
    commands
        .iter()
        .position(|command| match action {
            Action::Use => matches!(command, 0x05 | 0x06 | 0x25 | 0x26),
            Action::Drop => *command == 0x1e,
            Action::Transfer => *command == 0x31,
        })
        .map(|index| (commands.len() - 1 - index) as u8)
}
