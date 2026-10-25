use std::{cell::Cell, mem};

use crate::quick_use_policy::{choose_button, Pending, Pocket, ZL};
use skyline::hooks::{getRegionAddress, Region};

mod hints;
mod native;
mod offsets;
mod storage;

use native::{
    state, ActionSelector, ControllerState, FrameContext, InputControl, PlayerRef, PocketMenu,
};

use offsets::*;

thread_local! {
    // Addresses are identity tokens. Only pointers supplied by the current
    // native callback (and their live parent/child links) are dereferenced.
    static REQUEST: Cell<Option<Pending>> = const { Cell::new(None) };
}

unsafe fn text() -> *const u8 {
    getRegionAddress(Region::Text).cast()
}

unsafe fn global<T>(offset: usize) -> *mut T {
    text().add(offset).cast::<*mut T>().read()
}

unsafe fn inside_own_house(player: *const PlayerRef) -> bool {
    // The same ownership/context check used by the native ToStorage builder.
    let check: unsafe extern "C" fn(*const PlayerRef) -> bool =
        mem::transmute(text().add(OWN_HOUSE));
    check(player)
}

unsafe fn snapshot(pocket: *const PocketMenu, slot: u8) -> Pocket {
    Pocket {
        address: pocket as usize,
        selector: (*pocket).selector as usize,
        player: (*pocket).sequence.player.0,
        state: (*pocket).sequence.state,
        mode: (*pocket).mode,
        pending: (*pocket).sequence.pending,
        slot,
        capacity: (*pocket).capacity,
        blocked_slots: (*pocket).blocked_slots,
        events: (*pocket).events,
        input_enabled: false,
    }
}

// Native assigned-controller lookup and normalized fresh presses.
unsafe fn pressed(player: *const PlayerRef) -> u32 {
    let manager = global::<u8>(GAME_MANAGER);
    if manager.is_null() {
        return 0;
    }
    // The native method ignores its receiver and resolves/validates the actor.
    let assigned: unsafe extern "C" fn(*const u8, *const PlayerRef) -> i32 =
        mem::transmute(text().add(PLAYER_CONTROLLER));
    let controller = assigned(std::ptr::null(), player);
    if !(0..4).contains(&controller) {
        return 0;
    }
    let get: unsafe extern "C" fn(*mut u8, u32) -> *const ControllerState =
        mem::transmute(text().add(GET_CONTROLLER));
    let input = get(manager, controller as u32);
    if input.is_null() || (*input).device.is_null() {
        return 0;
    }
    let device = (*input).device;
    if !((*(*device).vtable).connected)(device) {
        return 0;
    }
    (*input).pressed
}

unsafe fn input_enabled(control: *const InputControl) -> bool {
    if control.is_null() || (*control).disabled != 0 {
        return false;
    }
    // Same context inhibition as native FUN_7103c4b5a0.
    let context = global::<FrameContext>(FRAME_CONTEXT);
    if context.is_null() || (*context).frame.is_null() {
        return false;
    }
    (*control).context_mask & (*(*context).frame).inhibited_contexts == 0
}

#[skyline::hook(offset = POCKET_PLAY)]
unsafe fn pocket_play(pocket: *mut PocketMenu) {
    REQUEST.with(|request| request.set(None));
    let mut selection = snapshot(pocket, (*pocket).selected_slot);
    let page = (*pocket).page;
    if selection.player < 8 && selection.slot < 40 && selection.mode == 0 && !page.is_null() {
        let button = (*page).slots[usize::from(selection.slot)];
        selection.input_enabled =
            input_enabled((*page).base.input) && !button.is_null() && (*button).flags & 1 != 0;
        if selection.input_enabled {
            let player = std::ptr::addr_of!((*pocket).sequence.player);
            let buttons = pressed(player);
            if let Some(request) =
                selection.start(buttons, buttons == ZL && inside_own_house(player))
            {
                REQUEST.with(|pending| pending.set(Some(request)));
                // Original Play latches the slot and queues CommandSelect.
                (*pocket).command_slot = selection.slot;
                (*pocket).events = 4;
            }
        }
    }
    original!()(pocket);
}

#[skyline::hook(offset = SELECTOR_APPEAR)]
unsafe fn selector_appear(selector: *mut ActionSelector) {
    let pending = REQUEST.with(Cell::get);
    let Some(pending) = pending.filter(|p| p.selector == selector as usize) else {
        original!()(selector);
        return;
    };
    REQUEST.with(|request| request.set(None));

    let pocket = (*selector).sequence.parent.cast::<PocketMenu>();
    let owns = !pocket.is_null()
        && pocket as usize == pending.pocket
        && snapshot(pocket, (*pocket).latched_slot).owns(pending, selector as usize);
    if !owns
        || (*selector).sequence.state != state::APPEAR
        || (*selector).sequence.frame != 0
        || (*selector).sequence.pending != [-1; 2]
        || !(*selector).sequence.queue_state(state::IDLE)
    {
        original!()(selector);
        return;
    }

    // First Appear builds the available records while hidden. Queuing Idle
    // prevents the later visible popup and preserves the accepted result.
    (*selector).count = 0;
    original!()(selector);

    let count = (*selector).count as usize;
    let mut commands = [-1; 7];
    let button = if (1..=commands.len()).contains(&count) {
        for (index, command) in commands[..count].iter_mut().enumerate() {
            *command = (*selector).records[index].command;
        }
        choose_button(&commands[..count], pending.action)
    } else {
        None
    };
    // Parent CommandSelect sees Idle and enters native ProcCommand, preserving
    // restrictions, errors and asynchronous completion without popup animation.
    (*selector).selected_button = button.unwrap_or(0xff);
}

pub(crate) fn install() {
    // Footer and storage share helpers; verify the complete dependency set
    // before installing any inventory hook.
    if !offsets::supported() {
        skyline::println!(
            "[additions-patches] Inventory patches skipped: unexpected game instructions"
        );
        return;
    }
    skyline::install_hooks!(pocket_play, selector_appear);
    hints::install();
    storage::install();
    skyline::println!(
        "[additions-patches] Inventory: ZR Use / ZL Drop or Transfer / storage X Pockets"
    );
}
