use super::{
    hints, input_enabled, inside_own_house,
    native::{state, GuideButton, PlayerRef, PocketMenu, PocketOptions, StorageMenu},
    pressed, text,
};
use crate::quick_use_policy::storage_overlay_allowed;
use std::{cell::Cell, mem, ptr};

use super::offsets::{OPEN_POCKETS, STORAGE_GUIDE, STORAGE_PLAY};

#[derive(Clone, Copy)]
struct Overlay {
    storage: usize,
    pockets: usize,
    player: u8,
}

thread_local! {
    // Identities only: dereference the current callback/global objects below.
    static OVERLAY: Cell<Option<Overlay>> = const { Cell::new(None) };
}

unsafe fn house_storage(menu: *const StorageMenu) -> bool {
    // D-pad storage can belong to the house controller, not the generic flow.
    matches!((*menu).mode, 0 | 1)
        && (*menu).sequence.player.0 < 8
        && inside_own_house(ptr::addr_of!((*menu).sequence.player))
}

unsafe fn finished(pockets: *const PocketMenu) -> bool {
    let sequence = ptr::addr_of!((*pockets).sequence);
    // Same request/completion protocol as native cAddMusic_Exec.
    (*sequence).requested == 0 && ((*(*sequence).vtable).inactive)(sequence) & 1 != 0
}

unsafe fn suspend(menu: *mut StorageMenu) {
    let page = (*menu).page;
    if !page.is_null() && !(*page).input.is_null() {
        (*(*page).input).disabled = 1;
    }
    // Category arrows have separate enable flags, as in native Play_Exit.
    for button in (*menu).category_buttons {
        if !button.is_null() {
            (*button).enabled = 0;
        }
    }
    (*menu).events = 0;
}

#[skyline::hook(offset = STORAGE_PLAY)]
unsafe fn storage_play(menu: *mut StorageMenu) {
    let manager = hints::manager();
    let pockets = if manager.is_null() {
        ptr::null_mut()
    } else {
        (*manager).pockets
    };
    if let Some(owner) = OVERLAY.with(Cell::get) {
        if owner.storage == menu as usize
            && owner.pockets == pockets as usize
            && owner.player == (*menu).sequence.player.0
            && !pockets.is_null()
        {
            suspend(menu);
            if !finished(pockets) {
                return;
            }
            // Native Appear reloads the cached item list after deposits, then
            // Play_Enter restores input and the guide. Do not replay pockets B.
            (*menu).sequence.queue_state(state::APPEAR);
            OVERLAY.with(|pending| pending.set(None));
            return;
        }
        OVERLAY.with(|pending| pending.set(None));
    }

    let page = (*menu).page;
    if house_storage(menu)
        && !page.is_null()
        && storage_overlay_allowed(
            (*menu).sequence.state,
            (*menu).sequence.pending,
            (*menu).events,
            input_enabled((*page).input),
            pressed(ptr::addr_of!((*menu).sequence.player)),
        )
        && !pockets.is_null()
        && finished(pockets)
    {
        let open: unsafe extern "C" fn(*const PlayerRef, *const PocketOptions) =
            mem::transmute(text().add(OPEN_POCKETS));
        // Keep storage alive beneath normal pockets, with Close enabled.
        open(
            ptr::addr_of!((*menu).sequence.player),
            &PocketOptions::NORMAL,
        );
        if (*pockets).sequence.requested != 0 {
            OVERLAY.with(|pending| {
                pending.set(Some(Overlay {
                    storage: menu as usize,
                    pockets: pockets as usize,
                    player: (*menu).sequence.player.0,
                }))
            });
            suspend(menu);
            return;
        }
    }
    original!()(menu);
}

#[skyline::hook(offset = STORAGE_GUIDE)]
unsafe fn storage_guide(menu: *mut StorageMenu, flags: u32) {
    let add = matches!((*menu).sequence.state, state::APPEAR | state::PLAY) && house_storage(menu);
    // Replace interior storage's wall-specific X labels with X Pockets.
    original!()(menu, if add { flags & !0x60 } else { flags });
    if add {
        hints::append((*menu).sequence.player, &[(GuideButton::X, "3915")]);
    }
}

pub(super) fn install() {
    skyline::install_hooks!(storage_play, storage_guide);
}
