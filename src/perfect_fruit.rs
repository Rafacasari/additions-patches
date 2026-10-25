use std::{
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    fruit, fruit_harvest, fruit_render,
    item::Item,
    perfect_fruit_policy::{
        eligible_fruit, native_fruit_id, refresh_tree, render_slots, saved_slots, FruitCommand,
        REGROWTH_MASK,
    },
};

// ACNH 3.0.3. Fruit-info is shared with the existing banana hook.
const GROW_TREE: usize = 0x0114D940;
const ATTACH_FRUIT: usize = 0x010E669C;
const HARVEST_TREE: usize = 0x021C33E0;
// Select the Item on the caller's command before native placement copies it.
const PLACE_FRUIT: usize = 0x021AB010;
const FALLBACK_FRUIT: usize = 0x021B1CB0;
const NATIVE_FRUIT: usize = 0x00F4CB54;

// The shared banana hook can remain installed when perfect-fruit instruction
// checks fail. Leave its perfect-fruit observer inactive in that case.
static ENABLED: AtomicBool = AtomicBool::new(false);

#[skyline::from_offset(NATIVE_FRUIT)]
unsafe extern "C" fn native_fruit_kind() -> u32;

fn random_percent() -> u32 {
    let mut value = 0u32;
    unsafe { skyline::nn::os::GenerateRandomBytes(ptr::addr_of_mut!(value).cast(), 4) };
    ((u64::from(value) * 100) >> 32) as u32
}

#[repr(C)]
struct TreeInfo {
    vtable: *const u8,
    item: Item,
    row: *const u8,
}

const _: () = assert!(core::mem::offset_of!(TreeInfo, item) == 8);
const _: () = assert!(core::mem::offset_of!(TreeInfo, row) == 0x10);

unsafe fn native_tree_fruit(tree: &Item) -> Option<Item> {
    // Mature native fruit only; special embedded-item/money-tree payloads excluded.
    if tree.free_param & (REGROWTH_MASK | 0x200000) != 0 {
        return None;
    }
    // Our own lookups must never capture a short-lived output as the original
    // harvest's source, including when game callbacks nest these operations.
    let _scope = fruit_harvest::Scope::enter(None);
    let mut fruit = Item {
        unique_id: 0xfffe,
        system_param: 0,
        additional_param: 0,
        free_param: 0,
    };
    let mut count = 0;
    if fruit::query(tree, &mut fruit, &mut count)
        && count == 3
        && eligible_fruit(*tree, fruit, native_fruit_id(native_fruit_kind()))
    {
        Some(fruit)
    } else {
        None
    }
}

// Observe the original outputs without tagging or modifying fruit Items.
pub(crate) unsafe fn observe_fruit_info(
    tree: *const Item,
    fruit: *mut Item,
    count: *const i32,
    force: u32,
    success: bool,
) {
    if !ENABLED.load(Ordering::Acquire) {
        return;
    }
    fruit_render::clear();
    if !success || fruit.is_null() || force & 1 != 0 {
        return;
    }
    if count.is_null() {
        if saved_slots((*tree).free_param) != 0 {
            let slots = render_slots(*tree, *fruit, native_fruit_id(native_fruit_kind()));
            fruit_render::remember(fruit, *fruit, slots);
        }
        return;
    }
    if *count != 3 {
        return;
    }
    fruit_harvest::capture(
        *tree,
        fruit,
        *count,
        || native_fruit_id(native_fruit_kind()),
    );
}

#[skyline::hook(offset = GROW_TREE)]
unsafe fn grow_tree(info: *mut TreeInfo, days: i64, special: u32, special_days: i64) -> u32 {
    let before = (*info).item;
    let result = original!()(info, days, special, special_days);
    let after = &mut (*info).item;
    let eligible = native_tree_fruit(after).is_some();
    if refresh_tree(before, after, eligible, random_percent) {
        // Existing mature trees are initialized on their next growth update.
        // Report a changed Item so callers persist it through the usual path.
        result | 1
    } else {
        result
    }
}

#[skyline::hook(offset = ATTACH_FRUIT)]
// Five general-purpose arguments. The apparent s0/s1/s2 inputs in Ghidra
// actually belong to a three-float return value inside the original helper.
unsafe fn attach_fruit(
    manager: *mut u8,
    fruit: *const Item,
    position: *const u8,
    offset: *const u8,
    slot: u32,
) {
    let mut scope = fruit_render::AttachmentScope::enter();
    let chosen = scope.choose(fruit, fruit.read(), slot as usize);
    if let Some(chosen) = chosen {
        original!()(manager, ptr::addr_of!(chosen), position, offset, slot);
    } else {
        original!()(manager, fruit, position, offset, slot);
    }
}

#[skyline::hook(offset = HARVEST_TREE)]
unsafe fn harvest_tree(command: *mut FruitCommand) -> u64 {
    let context = fruit_harvest::Context::new((*command).item);
    let _scope = fruit_harvest::Scope::enter(context.as_ref());
    original!()(command)
}

#[skyline::hook(offset = PLACE_FRUIT)]
unsafe fn place_fruit(operation: *mut u8, request: *const u8, command: *mut FruitCommand) -> u32 {
    fruit_harvest::prepare(command);
    original!()(operation, request, command)
}

#[skyline::hook(offset = FALLBACK_FRUIT)]
unsafe fn fallback_fruit(
    operation: *mut u8,
    request: *const u8,
    command: *mut FruitCommand,
) -> u32 {
    fruit_harvest::prepare(command);
    original!()(operation, request, command)
}

pub fn install() {
    const TARGETS: [(usize, [u32; 5]); 6] = [
        (
            GROW_TREE,
            [0xa9ba7bfd, 0xa9016ffc, 0xa90267fa, 0xa9035ff8, 0xa90457f6],
        ),
        (
            ATTACH_FRUIT,
            [0xd10483ff, 0x6d0933ed, 0x6d0a2beb, 0x6d0b23e9, 0xa90c7bfd],
        ),
        (
            HARVEST_TREE,
            [0x39406808, 0x37100688, 0xa9ba7bfd, 0xa9016ffc, 0x910003fd],
        ),
        (
            PLACE_FRUIT,
            [0x39404408, 0x34000e88, 0xd10143ff, 0xa9027bfd, 0xa90357f6],
        ),
        (
            FALLBACK_FRUIT,
            [0x39404408, 0x34000e88, 0xd10143ff, 0xa9027bfd, 0xa90357f6],
        ),
        (
            NATIVE_FRUIT,
            [0xa9bd7bfd, 0xf9000bf5, 0xa9024ff4, 0x910003fd, 0x90022948],
        ),
    ];
    if let Some((offset, _)) = TARGETS
        .iter()
        .find(|(offset, expected)| !fruit::matches_entry(*offset, expected))
    {
        skyline::println!(
            "[additions-patches] Perfect fruit skipped: unexpected instructions at 0x{:X}",
            offset
        );
        return;
    }
    skyline::install_hooks!(
        place_fruit,
        fallback_fruit,
        harvest_tree,
        attach_fruit,
        grow_tree
    );
    ENABLED.store(true, Ordering::Release);
    skyline::println!("[additions-patches] Native perfect fruit: saved per-slot growth rolls");
}
