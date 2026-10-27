use std::mem;

use nookkit::Player;

// The ownership/context check the native ToStorage builder uses
const OWN_HOUSE: usize = 0x02815598;

pub(super) fn supported() -> bool {
    crate::fruit::matches_entry(
        OWN_HOUSE,
        &[0xf0011988, 0x798fa908, 0xb100051f, 0x540007a0, 0xd100c3ff],
    )
}

pub(super) fn inside_own_house(player: Player) -> bool {
    let check: unsafe extern "C" fn(*const u8) -> bool =
        unsafe { mem::transmute(nookkit::module_base() + OWN_HOUSE) };

    unsafe { check(&player.0) }
}
