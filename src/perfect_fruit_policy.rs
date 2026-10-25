use crate::item::Item;

// Temporary testing rate. Restore to 1 for normal gameplay.
pub(crate) const PERFECT_CHANCE_PERCENT: u32 = 10;

pub(crate) const REGROWTH_MASK: u32 = 0x180000;
const SLOT_SHIFT: u32 = 25;
const INITIALIZED: u32 = 1 << 28;
const STATE_MASK: u32 = 0x1e000000;

pub(crate) fn saved_slots(free_param: u32) -> u8 {
    if free_param & INITIALIZED != 0 {
        ((free_param >> SLOT_SHIFT) & 7) as u8
    } else {
        0
    }
}

/// Shared by growth, rendering and harvest capture. Only plain native fruit
/// from a ripe tree is supported; embedded-item/money-tree payloads are excluded.
pub(crate) fn eligible_fruit(tree: Item, fruit: Item, native: Option<u16>) -> bool {
    tree.free_param & (REGROWTH_MASK | 0x200000) == 0
        && Some(fruit.unique_id) == native
        && fruit.system_param == 0
        && fruit.additional_param == 0
        && fruit.free_param == 0
}

pub(crate) fn render_slots(tree: Item, fruit: Item, native: Option<u16>) -> u8 {
    if eligible_fruit(tree, fruit, native) {
        saved_slots(tree.free_param)
    } else {
        0
    }
}

/// Runs after native growth. Keep one selection until a new fruit cycle is ready.
pub(crate) fn refresh_tree(
    before: Item,
    after: &mut Item,
    eligible: bool,
    mut roll_percent: impl FnMut() -> u32,
) -> bool {
    if !eligible
        || after.free_param & REGROWTH_MASK != 0
        || (after.free_param & INITIALIZED != 0
            && before.free_param & REGROWTH_MASK == 0
            && before.unique_id == after.unique_id)
    {
        return false;
    }
    let mut slots = 0u32;
    for slot in 0..3 {
        if roll_percent() < PERFECT_CHANCE_PERCENT {
            slots |= 1 << slot;
        }
    }
    after.free_param = (after.free_param & !STATE_MASK) | INITIALIZED | (slots << SLOT_SHIFT);
    true
}

pub(crate) fn fruit_at_slot(mut fruit: Item, slots: u8, slot: usize) -> Item {
    if slot < 3 && slots & (1 << slot) != 0 {
        if let Some(perfect) = perfect_variant(fruit.unique_id) {
            fruit.unique_id = perfect;
        }
    }
    fruit
}

/// The game's native-fruit getter returns a kind, not an item ID.
pub(crate) fn native_fruit_id(kind: u32) -> Option<u16> {
    [2287, 2214, 2285, 2286, 2213].get(kind as usize).copied()
}

/// Shared 36-byte tree command. Only fields needed by this patch are named.
#[repr(C, packed)]
pub(crate) struct FruitCommand {
    request: [u8; 4],
    pub item: Item,
    position: [u8; 10],
    pub kind: u8,
    player: u8,
    flags: [u8; 2],
    pub slot: u8,
    tail: [u8; 9],
}

const _: () = assert!(core::mem::size_of::<FruitCommand>() == 0x24);
const _: () = assert!(core::mem::offset_of!(FruitCommand, item) == 4);
const _: () = assert!(core::mem::offset_of!(FruitCommand, kind) == 0x16);
const _: () = assert!(core::mem::offset_of!(FruitCommand, slot) == 0x1a);

pub(crate) fn apply_slot(command: &mut FruitCommand, source: &mut Item, fruit: Item, slots: u8) {
    let index = (command.slot & 3) as usize;
    // A retry may still contain the preceding slot's perfect variant. Accept
    // only this harvest's plain/perfect Item, never unrelated commands which
    // happen to be submitted while the harvest scope is active.
    if matches!(command.kind, 14 | 15)
        && index < 3
        && (command.item == fruit || command.item == fruit_at_slot(fruit, 1, 0))
    {
        let chosen = fruit_at_slot(fruit, slots, index);
        command.item = chosen;
        *source = chosen;
    }
}

/// Normal item IDs -> this mod's perfect fruit IDs (not foreground IDs).
pub(crate) fn perfect_variant(id: u16) -> Option<u16> {
    match id {
        2213 => Some(2526), // Apple
        2214 => Some(2527), // Orange
        2285 => Some(2528), // Pear
        2286 => Some(2529), // Peach
        2287 => Some(2530), // Cherry
        _ => None,
    }
}
