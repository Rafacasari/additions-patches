//! ACNH 3.0.3 inventory module addresses and entry fingerprints.
//! Keep version-specific dependencies together; object layouts are in native.rs.

pub(super) const POCKET_PLAY: usize = 0x0308B3F0; // State::cPlay_Exec
pub(super) const SELECTOR_APPEAR: usize = 0x02CB2F70; // State::cAppear_Exec
pub(super) const PLAYER_CONTROLLER: usize = 0x024835F4;
pub(super) const GET_CONTROLLER: usize = 0x008B56E8;
pub(super) const GAME_MANAGER: usize = 0x04C02F58;
pub(super) const FRAME_CONTEXT: usize = 0x059F83F0;
pub(super) const OWN_HOUSE: usize = 0x02815598;

pub(super) const BUILD_GUIDE: usize = 0x03094FB4;
pub(super) const APPEND_GUIDE: usize = 0x02DE2B00;
pub(super) const UI_MANAGER: usize = 0x05639190;
pub(super) const STORAGE_PLAY: usize = 0x0316D130;
pub(super) const OPEN_POCKETS: usize = 0x033598A0;
pub(super) const STORAGE_GUIDE: usize = 0x0316F388;

pub(super) fn supported() -> bool {
    let entries = [
        (
            POCKET_PLAY,
            [0xd10283ff, 0xfd001be8, 0xa9047bfd, 0xa9056ffc, 0xa90667fa],
        ),
        (
            SELECTOR_APPEAR,
            [0xfc180fea, 0x6d0123e9, 0xa9027bfd, 0x910083fd, 0xa9036ffc],
        ),
        (
            PLAYER_CONTROLLER,
            [0xa9bf7bfd, 0x910003fd, 0xaa0103e0, 0x52800021, 0x97fffde2],
        ),
        (
            GET_CONTROLLER,
            [0xd10103ff, 0xa9017bfd, 0xa90257f6, 0xa9034ff4, 0x910043fd],
        ),
        (
            OWN_HOUSE,
            [0xf0011988, 0x798fa908, 0xb100051f, 0x540007a0, 0xd100c3ff],
        ),
        (
            BUILD_GUIDE,
            [0xd10343ff, 0xa9087bfd, 0xf9004bf9, 0xa90a5ff8, 0xa90b57f6],
        ),
        (
            APPEND_GUIDE,
            [0xb941f008, 0x3100051f, 0xb941f409, 0xb941b00a, 0x3a410920],
        ),
        (
            STORAGE_PLAY,
            [0xd10303ff, 0xa9067bfd, 0xa9076ffc, 0xa90867fa, 0xa9095ff8],
        ),
        (
            OPEN_POCKETS,
            [0xa9bd7bfd, 0xf9000bf5, 0xa9024ff4, 0x910003fd, 0xb000bac9],
        ),
        (
            STORAGE_GUIDE,
            [0xd10343ff, 0xa9097bfd, 0xf90053f7, 0xa90b57f6, 0xa90c4ff4],
        ),
    ];
    entries
        .iter()
        .all(|(offset, words)| crate::fruit::matches_entry(*offset, words))
}
