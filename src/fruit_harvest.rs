use std::{cell::Cell, marker::PhantomData, ptr};

use crate::{
    item::Item,
    perfect_fruit_policy::{apply_slot, eligible_fruit, saved_slots, FruitCommand},
};

#[derive(Clone, Copy)]
struct Source {
    address: *mut Item,
    fruit: Item,
}

pub(crate) struct Context {
    tree: Item,
    slots: u8,
    source: Cell<Option<Source>>,
}

impl Context {
    pub(crate) fn new(tree: Item) -> Option<Self> {
        let slots = saved_slots(tree.free_param);
        // All-normal/uninitialized cycles require no overrides or lookups.
        (slots != 0).then(|| Self {
            tree,
            slots,
            source: Cell::new(None),
        })
    }
}

thread_local! {
    static CONTEXT: Cell<*const Context> = const { Cell::new(ptr::null()) };
}

#[must_use]
pub(crate) struct Scope<'a> {
    previous: *const Context,
    _borrow: PhantomData<&'a Context>,
}

impl<'a> Scope<'a> {
    pub(crate) fn enter(context: Option<&'a Context>) -> Self {
        Self {
            previous: CONTEXT.with(|active| active.replace(context.map_or(ptr::null(), |c| c))),
            _borrow: PhantomData,
        }
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        CONTEXT.with(|active| active.set(self.previous));
    }
}

/// The original caller must keep `fruit` alive through this harvest's prepare
/// calls. Scope restoration never dereferences it after the original returns.
/// Called only for successful, non-forced fruit-info outputs.
pub(crate) unsafe fn capture(
    tree: Item,
    fruit: *mut Item,
    count: i32,
    native: impl FnOnce() -> Option<u16>,
) {
    CONTEXT.with(|active| {
        let Some(context) = active.get().as_ref() else {
            return;
        };
        if context.source.get().is_none()
            && count == 3
            && tree == context.tree
            && !fruit.is_null()
            && eligible_fruit(tree, *fruit, native())
        {
            context.source.set(Some(Source {
                address: fruit,
                fruit: *fruit,
            }));
        }
    });
}

/// The command belongs to the native placement call; captured source storage
/// belongs to its enclosing native harvest call, which is still active.
pub(crate) unsafe fn prepare(command: *mut FruitCommand) {
    CONTEXT.with(|active| {
        let Some(context) = active.get().as_ref() else {
            return;
        };
        let Some(captured) = context.source.get() else {
            return;
        };
        let Some(command) = command.as_mut() else {
            return;
        };
        let Some(source) = captured.address.as_mut() else {
            return;
        };
        apply_slot(command, source, captured.fruit, context.slots);
    });
}
