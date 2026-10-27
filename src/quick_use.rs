use nookkit::{SeqState, Ui};

mod house;
mod pockets;
mod storage;

// The pocket, command popup and storage seqs share these states
const IDLE: SeqState = SeqState(1);
const APPEAR: SeqState = SeqState(2);
const PLAY: SeqState = SeqState(3);

pub(crate) fn install(ui: Ui) {
    if !house::supported() {
        skyline::println!(
            "[additions-patches] Inventory patches skipped: unexpected game instructions"
        );
        return;
    }

    match pockets::install(ui).and_then(|()| storage::install(ui)) {
        Ok(()) => skyline::println!(
            "[additions-patches] Inventory: ZR Use / ZL Drop or Transfer / storage X Pockets"
        ),
        Err(e) => skyline::println!("[additions-patches] Inventory patches failed: {e}"),
    }
}
