//! Verified ACNH 3.0.3 prefixes, not complete class definitions. Unknown bytes
//! stay opaque. Use raw pointers at callbacks: do not retain Rust references
//! across calls into the game, which can mutate these same objects.
use core::mem::{offset_of, size_of};

pub mod state {
    pub const IDLE: i32 = 1;
    pub const APPEAR: i32 = 2;
    pub const PLAY: i32 = 3;
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PlayerRef(pub u8);

// Guide icon IDs are distinct from the normalized input masks in policy.
#[repr(i32)]
#[derive(Clone, Copy)]
pub enum GuideButton {
    X = 6,
    Zl = 10,
    Zr = 11,
}

#[repr(C)]
pub struct PocketOptions {
    mode: u32,
    close_enabled: u8,
    _reserved: [u8; 3],
}

impl PocketOptions {
    pub const NORMAL: Self = Self {
        mode: 0,
        close_enabled: 1,
        _reserved: [0; 3],
    };
}

#[repr(C)]
pub struct SequenceVtable {
    _unknown: [usize; 29],
    pub inactive: unsafe extern "C" fn(*const Sequence) -> u64,
}

#[repr(C)]
pub struct Sequence {
    pub vtable: *const SequenceVtable,
    _008: [u8; 0x40],
    pub parent: *mut Sequence,
    _050: [u8; 0x115],
    pub requested: u8,
    _166: [u8; 0x4a],
    pub state: i32,
    pub frame: i32,
    _1b8: [u8; 0x38],
    pub pending: [i32; 2],
    pub player: PlayerRef,
    _1f9: [u8; 3],
    pub request_id: i32,
    pub queue_index: u8,
    _201: [u8; 0xf],
}

impl Sequence {
    pub fn queue_state(&mut self, state: i32) -> bool {
        let Some(pending) = self.pending.get_mut(usize::from(self.queue_index)) else {
            return false;
        };
        if *pending != -1 {
            return false;
        }
        *pending = state;
        true
    }
}

#[repr(C)]
pub struct InputControl {
    _000: [u8; 0x54],
    pub disabled: u8,
    _055: [u8; 3],
    pub context_mask: u32,
}

#[repr(C)]
pub struct Page {
    _000: [u8; 0x210],
    pub input: *mut InputControl,
}

#[repr(C)]
pub struct Button {
    _000: [u8; 0xbc],
    pub enabled: u8,
    _0bd: [u8; 0x206],
    pub flags: u8,
}

#[repr(C)]
pub struct PocketPage {
    pub base: Page,
    _218: [u8; 0x378],
    pub slots: [*mut Button; 40],
}

#[repr(C)]
pub struct PocketMenu {
    pub sequence: Sequence,
    pub page: *mut PocketPage,
    pub selector: *mut ActionSelector,
    _220: [u8; 0x1c2d0],
    pub blocked_slots: u64,
    _1c4f8: [u8; 4],
    pub mode: i32,
    pub capacity: u32,
    _1c504: [u8; 0x10],
    pub latched_slot: u8,
    pub selected_slot: u8,
    _1c516: [u8; 2],
    pub command_slot: u8,
    _1c519: u8,
    pub events: u16,
}

#[repr(C)]
pub struct CommandRecord {
    _000: [u8; 0x168],
    pub command: i32,
    _16c: [u8; 0x3cc],
}

#[repr(C)]
pub struct ActionSelector {
    pub sequence: Sequence,
    _210: [u8; 0x28],
    pub records: [CommandRecord; 7],
    _26c0: [u8; 9],
    pub selected_button: u8,
    _26ca: [u8; 0xe],
    pub count: u32,
}

#[repr(C)]
pub struct StorageMenu {
    pub sequence: Sequence,
    pub page: *mut Page,
    _218: [u8; 8],
    pub mode: i32,
    _224: [u8; 0xaa6d4],
    pub category_buttons: [*mut Button; 2],
    _aa908: [u8; 0xd8],
    pub events: u64,
}

#[repr(C)]
pub struct UiManager {
    _000: [u8; 0x58],
    pub guide: *mut GuideMenu,
    _060: [u8; 0x28],
    pub pockets: *mut PocketMenu,
}

#[repr(C)]
pub struct GuideMenu {
    _000: [u8; 0x218],
    pub count: u32,
    pub capacity: u32,
    pub entries: *const *const GuideEntry,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct String64 {
    _vtable: *const (),
    pub buffer: *mut u8,
    pub capacity: u32,
    pub inline: [u8; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GuideEntry {
    pub buttons: [i32; 4],
    pub label: String64,
    pub player: PlayerRef,
    _069: [u8; 3],
    pub style: u32,
    pub override_label: u8,
    _071: [u8; 3],
    // Two 32-bit sentinels; deliberately not an unaligned u64 at +0x74.
    pub alternate: [i32; 2],
    _07c: [u8; 4],
    _sort_key: u32,
    _084: [u8; 4],
}

#[repr(C)]
pub struct ControllerState {
    _000: [u8; 0x8c],
    pub pressed: u32,
    _090: [u8; 0xb90],
    pub device: *const Device,
}

#[repr(C)]
pub struct Device {
    pub vtable: *const DeviceVtable,
}

#[repr(C)]
pub struct DeviceVtable {
    _unknown: [usize; 6],
    pub connected: unsafe extern "C" fn(*const Device) -> bool,
}

#[repr(C)]
pub struct FrameContext {
    _000: [u8; 0x418],
    pub frame: *const FrameState,
}

#[repr(C)]
pub struct FrameState {
    _000: [u8; 0x10],
    pub inhibited_contexts: u32,
}

// Assert every accessed member independently of the code that uses it. These
// also compile in the host tests; no game objects need to be instantiated.
macro_rules! layout {
    ($ty:ty, $($field:ident: $offset:expr),+ $(,)?) => {
        $(const _: () = assert!(offset_of!($ty, $field) == $offset);)+
    };
}
const _: () = assert!(size_of::<usize>() == 8);
const _: () = assert!(size_of::<Sequence>() == 0x210);
const _: () = assert!(size_of::<CommandRecord>() == 0x538);
const _: () = assert!(size_of::<String64>() == 0x58);
const _: () = assert!(size_of::<GuideEntry>() == 0x88);
const _: () = assert!(size_of::<PocketOptions>() == 8);
layout!(PocketOptions, mode: 0, close_enabled: 4);
layout!(SequenceVtable, inactive: 0xe8);
layout!(Sequence, vtable: 0, parent: 0x48, requested: 0x165, state: 0x1b0,
    frame: 0x1b4, pending: 0x1f0, player: 0x1f8, request_id: 0x1fc, queue_index: 0x200);
layout!(InputControl, disabled: 0x54, context_mask: 0x58);
layout!(Page, input: 0x210);
layout!(Button, enabled: 0xbc, flags: 0x2c3);
layout!(PocketPage, base: 0, slots: 0x590);
layout!(PocketMenu, sequence: 0, page: 0x210, selector: 0x218,
    blocked_slots: 0x1c4f0, mode: 0x1c4fc, capacity: 0x1c500,
    latched_slot: 0x1c514, selected_slot: 0x1c515, command_slot: 0x1c518, events: 0x1c51a);
layout!(CommandRecord, command: 0x168);
layout!(ActionSelector, sequence: 0, records: 0x238, selected_button: 0x26c9, count: 0x26d8);
layout!(StorageMenu, sequence: 0, page: 0x210, mode: 0x220, category_buttons: 0xaa8f8, events: 0xaa9e0);
layout!(UiManager, guide: 0x58, pockets: 0x88);
layout!(GuideMenu, count: 0x218, capacity: 0x21c, entries: 0x220);
layout!(String64, buffer: 8, capacity: 0x10, inline: 0x14);
layout!(GuideEntry, buttons: 0, label: 0x10, player: 0x68, style: 0x6c,
    override_label: 0x70, alternate: 0x74);
layout!(ControllerState, pressed: 0x8c, device: 0xc20);
layout!(Device, vtable: 0);
layout!(DeviceVtable, connected: 0x30);
layout!(FrameContext, frame: 0x418);
layout!(FrameState, inhibited_contexts: 0x10);
