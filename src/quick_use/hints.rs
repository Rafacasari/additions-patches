use super::{
    global, inside_own_house,
    native::{state, GuideButton, GuideEntry, GuideMenu, PlayerRef, PocketMenu, UiManager},
    text,
};
use std::mem;

use super::offsets::{APPEND_GUIDE, BUILD_GUIDE, UI_MANAGER};

pub(super) unsafe fn manager() -> *mut UiManager {
    global(UI_MANAGER)
}

#[skyline::hook(offset = BUILD_GUIDE)]
unsafe fn build_guide(pocket: *mut PocketMenu, style: i32) {
    original!()(pocket, style);
    if !matches!(style, 0 | 3)
        || (*pocket).mode != 0
        || !matches!((*pocket).sequence.state, state::APPEAR | state::PLAY)
    {
        return;
    }
    let player = std::ptr::addr_of!((*pocket).sequence.player);
    append(
        *player,
        &[
            (
                GuideButton::Zl,
                if inside_own_house(player) {
                    "ap_transfer"
                } else {
                    "ap_drop"
                },
            ),
            (GuideButton::Zr, "ap_use"),
        ],
    );
}

pub(super) unsafe fn append(player: PlayerRef, hints: &[(GuideButton, &str)]) {
    if hints.is_empty() || hints.iter().any(|(_, label)| label.len() >= 64) {
        return;
    }
    let manager = manager();
    if manager.is_null() || (*manager).guide.is_null() {
        return;
    }
    let guide = (*manager).guide;
    let count = (*guide).count as usize;
    let entries = (*guide).entries;
    if count == 0 || (*guide).capacity != 5 || count + hints.len() > 5 || entries.is_null() {
        return;
    }
    let first = *entries;
    if first.is_null() {
        return;
    }
    // Preserve the native String64 vtable and unknown entry fields. Append
    // synchronously copies this temporary record into the guide's bounded pool.
    let template = *first;
    let append: unsafe extern "C" fn(*mut GuideMenu, *const GuideEntry, u32) =
        mem::transmute(text().add(APPEND_GUIDE));
    for &(button, label) in hints {
        let mut entry = template;
        entry.buttons = [button as i32, -1, -1, -1];
        entry.label.inline = [0; 64];
        entry.label.inline[..label.len()].copy_from_slice(label.as_bytes());
        entry.label.buffer = entry.label.inline.as_mut_ptr();
        entry.label.capacity = 64;
        entry.player = player;
        entry.style = 0;
        entry.override_label = 0;
        entry.alternate = [-1; 2];
        append(guide, &entry, 1);
    }
}

pub(super) fn install() {
    skyline::install_hooks!(build_guide);
}
