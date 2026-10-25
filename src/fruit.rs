use crate::item::Item;
use skyline::hooks::{getRegionAddress, Region};

const BANANA_ITEM_ID: u16 = 2522;

// ACNH 3.0.3 main-module offset, relative to the text base.
const FRUIT_DROP_INFO_OFFSET: usize = 0x0113B020;

#[skyline::hook(offset = FRUIT_DROP_INFO_OFFSET)]
unsafe fn tree_fruit_drop_info(
    tree: *const Item,
    out_fruit: *mut Item,
    out_count: *mut i32,
    force: u32,
) -> bool {
    let success = original!()(tree, out_fruit, out_count, force);

    // The original constructs a plain fruit Item from NutItem, so its raw ID
    // is sufficient here. Both output pointers are optional.
    if success
        && !out_fruit.is_null()
        && !out_count.is_null()
        && (*out_fruit).unique_id == BANANA_ITEM_ID
    {
        *out_count = 2;
    }

    crate::perfect_fruit::observe_fruit_info(tree, out_fruit, out_count, force, success);

    success
}

// Reuse this hook's original trampoline; no second fruit-info offset is needed.
pub(crate) unsafe fn query(tree: &Item, fruit: &mut Item, count: &mut i32) -> bool {
    tree_fruit_drop_info(tree, fruit, count, 0)
}

// Check complete instruction words; five words cover the largest entry patch
// used by the installed Skyline relocator. Shared by both feature installers.
pub(crate) fn matches_entry(offset: usize, expected: &[u32]) -> bool {
    unsafe {
        let text = getRegionAddress(Region::Text).cast::<u8>();
        expected.iter().enumerate().all(|(index, word)| {
            text.add(offset + index * 4).cast::<u32>().read_unaligned() == *word
        })
    }
}

pub fn install() -> bool {
    // Check the expected function entry before installing the hook.
    const FUNCTION_ENTRY: [u32; 5] = [0xd10183ff, 0xa9027bfd, 0xf9001bf7, 0xa90457f6, 0xa9054ff4];
    let matches = matches_entry(FRUIT_DROP_INFO_OFFSET, &FUNCTION_ENTRY);

    if !matches {
        skyline::println!("[additions-patches] Banana patch skipped: unexpected game instructions");
        return false;
    }

    skyline::install_hook!(tree_fruit_drop_info);
    skyline::println!("[additions-patches] Banana trees drop two fruits");
    true
}
