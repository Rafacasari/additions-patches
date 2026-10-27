use std::cell::Cell;

use nookkit::screens::Storage;
use nookkit::{pages, Key, Page, Result, Seq, Ui};

use super::house::inside_own_house;
use super::{APPEAR, PLAY};
use crate::quick_use_policy::storage_events_clear;

#[derive(Clone, Copy)]
struct Overlay {
    storage: Page,
    pockets: Seq,
}

thread_local! {
    static OVERLAY: Cell<Option<Overlay>> = const { Cell::new(None) };
}

pub(super) fn install(ui: Ui) -> Result {
    ui.on_page::<pages::Storage>(move |page| {
        // Consumed, so the house's own X (accent wall options) never sees it
        let x = page.bind_key(Key::X).consume().guide_label("3915").build()?;
        x.set_enabled(false)?;
        x.on_trigger(move |_| open_pockets(ui, page))?;

        page.on_activate(move |seq| {
            let house = house_storage(seq)?;

            x.set_enabled(house)?;
            // X Pockets replaces the wall-specific X labels
            page.set_guide_hidden(Key::X, house)
        })?;

        page.on_state(PLAY).before_exec(move |seq| hold(page, seq))?;

        Ok(())
    })
}

// D-pad storage can belong to the house controller, not the generic flow
fn house_storage(seq: Seq) -> Result<bool> {
    let Some(storage) = seq.downcast::<Storage>()? else {
        return Ok(false);
    };

    Ok(matches!(storage.mode()?, 0 | 1) && inside_own_house(seq.player()?))
}

fn open_pockets(ui: Ui, page: Page) -> Result {
    if OVERLAY.with(Cell::get).is_some_and(|o| o.storage.is_alive()) {
        return Ok(());
    }

    let Some(seq) = page.seq()? else {
        return Ok(());
    };
    let Some(storage) = seq.downcast::<Storage>()? else {
        return Ok(());
    };
    let Some(pockets) = (match ui.find_page::<pages::PocketMenu>()? {
        Some(pockets) => pockets.seq()?,
        None => None,
    }) else {
        return Ok(());
    };

    let ready = seq.state()? == PLAY
        && !seq.has_request()?
        && storage_events_clear(storage.events()?)
        && pockets.is_idle()?
        && !pockets.has_request()?;
    if !ready {
        return Ok(());
    }

    storage.clear_events()?;
    // Normal pockets with Close, storage stays alive beneath them
    ui.open_pocket_menu(seq.player()?, true)?;

    if pockets.has_request()? {
        suspend(page, storage)?;
        OVERLAY.with(|overlay| overlay.set(Some(Overlay { storage: page, pockets })));
    }

    Ok(())
}

// Category arrows have their own enable flags, as in the game's Play_Exit
fn suspend(page: Page, storage: Storage) -> Result {
    page.set_input_enabled(false)?;
    storage.disable_category_arrows()?;
    storage.clear_events()
}

fn hold(page: Page, seq: Seq) -> Result {
    let Some(overlay) = OVERLAY.with(Cell::get).filter(|o| o.storage == page) else {
        return Ok(());
    };
    let Some(storage) = seq.downcast::<Storage>()? else {
        return Ok(());
    };

    suspend(page, storage)?;

    // Pockets the game destroyed count as closed
    let open = match (overlay.pockets.is_idle(), overlay.pockets.has_request()) {
        (Ok(idle), Ok(request)) => !idle || request,
        _ => false,
    };
    if open {
        return Ok(());
    }

    OVERLAY.with(|overlay| overlay.set(None));

    // Appear reloads the item list after deposits, then Play_Enter restores the guide
    seq.request_state(APPEAR)?;
    page.set_input_enabled(true)
}
