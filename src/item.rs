/// Eight-byte SaveItemName record; FreeParam is interpreted by item kind.
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Item {
    pub unique_id: u16,
    pub system_param: u8,
    pub additional_param: u8,
    pub free_param: u32,
}

const _: () = assert!(core::mem::size_of::<Item>() == 8);
const _: () = assert!(core::mem::align_of::<Item>() == 1);
const _: () = assert!(core::mem::offset_of!(Item, unique_id) == 0);
const _: () = assert!(core::mem::offset_of!(Item, system_param) == 2);
const _: () = assert!(core::mem::offset_of!(Item, additional_param) == 3);
const _: () = assert!(core::mem::offset_of!(Item, free_param) == 4);
