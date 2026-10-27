use std::cell::Cell;

use nookkit::screens::{CommandSelect, PocketMenu};
use nookkit::{pages, Key, Page, Result, Seq, SeqState, Ui};

use super::house::inside_own_house;
use super::{APPEAR, IDLE, PLAY};
use crate::quick_use_policy::{choose_command, Action};

// Play with a slot latched for the command popup
const COMMANDS: SeqState = SeqState(5);

#[derive(Clone, Copy)]
struct Request {
    pocket: usize,
    slot: u8,
    action: Action,
}

thread_local! {
    static REQUEST: Cell<Option<Request>> = const { Cell::new(None) };

    // From the popup's first hidden Appear step to its end, the action it answers
    static ANSWERING: Cell<Option<Action>> = const { Cell::new(None) };
}

pub(super) fn install(ui: Ui) -> Result {
    ui.on_page::<pages::PocketMenu>(|page| {
        let zr = page.bind_key(Key::Zr).guide_label("ap_use").build()?;
        let zl = page.bind_key(Key::Zl).guide_label("ap_drop").build()?;

        zr.on_trigger(move |_| request(page, Action::Use))?;
        zl.on_trigger(move |event| {
            request(
                page,
                if inside_own_house(event.player) {
                    Action::Transfer
                } else {
                    Action::Drop
                },
            )
        })?;

        // Only the normal pockets, not selling, donating, …
        page.on_activate(move |seq| {
            let normal = match seq.downcast::<PocketMenu>()? {
                Some(pocket) => pocket.mode()? == 0,
                None => false,
            };

            zr.set_enabled(normal)?;
            zl.set_enabled(normal)?;

            zl.set_guide_label(if inside_own_house(seq.player()?) {
                "ap_transfer"
            } else {
                "ap_drop"
            })
        })?;

        page.on_state(PLAY).after_exec(drop_untaken)?;
        page.on_deactivate(|_| {
            REQUEST.with(|request| request.set(None));
            Ok(())
        })?;

        Ok(())
    })?;

    ui.on_page::<pages::CommandSelect>(|page| {
        page.on_state(APPEAR)
            .before_exec(intercept)?
            .after_exec(answer)?;

        Ok(())
    })
}

fn request(page: Page, action: Action) -> Result {
    let Some(seq) = page.seq()? else {
        return Ok(());
    };

    let Some(pocket) = seq.downcast::<PocketMenu>()? else {
        return Ok(());
    };

    let slot = pocket.focused_slot()?;

    let ready = seq.state()? == PLAY
        && !seq.has_request()?
        && pocket.mode()? == 0
        && pocket.events()? == 0
        && u32::from(slot) < pocket.capacity()?
        && !pocket.is_blocked(slot)?
        && pocket.is_slot_valid(slot)?;

    // Play latches the slot and opens the popup on its next step, as for A
    if ready && pocket.open_commands(slot)? {
        REQUEST.with(|request| {
            request.set(Some(Request {
                pocket: seq.raw() as usize,
                slot,
                action,
            }))
        });
    }

    Ok(())
}

// Play took no popup from its events, so a request left behind would answer a later A
fn drop_untaken(seq: Seq) -> Result {
    let Some(pocket) = seq.downcast::<PocketMenu>()? else {
        return Ok(());
    };

    if pocket.events()? == 0 && !seq.has_request()? && seq.state()? == PLAY {
        REQUEST.with(|request| request.set(None));
    }

    Ok(())
}

fn intercept(seq: Seq) -> Result {
    let Some(request) = REQUEST.with(Cell::get) else {
        return Ok(());
    };

    if seq.state_frame()? != 0 || seq.has_request()? {
        return Ok(());
    }

    REQUEST.with(|request| request.set(None));

    let (Some(select), Some(parent)) = (seq.downcast::<CommandSelect>()?, seq.parent()?) else {
        return Ok(());
    };

    let Some(pocket) = parent.downcast::<PocketMenu>()? else {
        return Ok(());
    };

    let owns = parent.raw() as usize == request.pocket
        && parent.state()? == COMMANDS
        && pocket.latched_slot()? == request.slot
        && pocket.mode()? == 0;

    // This first Appear step builds the records while hidden, and Idle keeps the popup from showing
    if owns && seq.request_state(IDLE)? {
        select.clear()?;
        ANSWERING.with(|answering| answering.set(Some(request.action)));
    }

    Ok(())
}

// Picking a command and going Idle makes the pockets run it as if chosen, with the game's own
// restrictions, errors and completion
fn answer(seq: Seq) -> Result {
    let Some(action) = ANSWERING.with(Cell::take) else {
        return Ok(());
    };

    let Some(select) = seq.downcast::<CommandSelect>()? else {
        return Ok(());
    };

    match choose_command(&select.commands()?, action) {
        Some(index) => {
            select.choose(index)?;
        }
        None => select.cancel()?,
    }

    Ok(())
}
