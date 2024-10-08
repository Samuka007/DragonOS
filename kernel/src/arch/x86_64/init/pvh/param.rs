/*
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to
 * deal in the Software without restriction, including without limitation the
 * rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
 * sell copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 *
 * Copyright (c) 2016, Citrix Systems, Inc.
 */

/*
 * automatically generated by rust-bindgen using:
 *
 * # bindgen start_info.h -- -include stdint.h > start_info.rs
 *
 * From the canonical version in upstream Xen repository
 * xen/include/public/arch-x86/hvm/start_info.h
 * at commit:
 * b4642c32c4d079916d5607ddda0232aae5e1690e
 *
 * The generated file has been edited to eliminate unnecessary
 * definitions, add comments, and relocate definitions and tests for clarity.
 * Added Default to the list of traits that are automatically derived.
 *
 * The definitions in this file are intended to be exported and used by a particular
 * VMM implementation in order to boot a Linux guest using the PVH entry point as
 * specified in the x86/HVM direct boot ABI.
 * These structures contain all the required information (cmdline address, ACPI RSDP,
 * memory maps, etc) that must be written to guest memory before starting guest
 * execution by jumping to the PVH entry point address.
 * A comparable set of definitions to hvm_start_info and hvm_memmap_table_entry in this
 * file would be the boot_params and boot_e820_entry definitions used by the Linux
 * 64-bit boot protocol.
 *
 * Start of day structure passed to PVH guests and to HVM guests in %ebx.
 *
 * NOTE: nothing will be loaded at physical address 0, so a 0 value in any
 * of the address fields should be treated as not present.
 *
 *  0 +----------------+
 *    | magic          | Contains the magic value XEN_HVM_START_MAGIC_VALUE
 *    |                | ("xEn3" with the 0x80 bit of the "E" set).
 *  4 +----------------+
 *    | version        | Version of this structure. Current version is 1. New
 *    |                | versions are guaranteed to be backwards-compatible.
 *  8 +----------------+
 *    | flags          | SIF_xxx flags.
 * 12 +----------------+
 *    | nr_modules     | Number of modules passed to the kernel.
 * 16 +----------------+
 *    | modlist_paddr  | Physical address of an array of modules
 *    |                | (layout of the structure below).
 * 24 +----------------+
 *    | cmdline_paddr  | Physical address of the command line,
 *    |                | a zero-terminated ASCII string.
 * 32 +----------------+
 *    | rsdp_paddr     | Physical address of the RSDP ACPI data structure.
 * 40 +----------------+
 *    | memmap_paddr   | Physical address of the (optional) memory map. Only
 *    |                | present in version 1 and newer of the structure.
 * 48 +----------------+
 *    | memmap_entries | Number of entries in the memory map table. Zero
 *    |                | if there is no memory map being provided. Only
 *    |                | present in version 1 and newer of the structure.
 * 52 +----------------+
 *    | reserved       | Version 1 and newer only.
 * 56 +----------------+
 *
 * The layout of each entry in the module structure is the following:
 *
 *  0 +----------------+
 *    | paddr          | Physical address of the module.
 *  8 +----------------+
 *    | size           | Size of the module in bytes.
 * 16 +----------------+
 *    | cmdline_paddr  | Physical address of the command line,
 *    |                | a zero-terminated ASCII string.
 * 24 +----------------+
 *    | reserved       |
 * 32 +----------------+
 *
 * The layout of each entry in the memory map table is as follows:
 *
 *  0 +----------------+
 *    | addr           | Base address
 *  8 +----------------+
 *    | size           | Size of mapping in bytes
 * 16 +----------------+
 *    | type           | Type of mapping as defined between the hypervisor
 *    |                | and guest. See XEN_HVM_MEMMAP_TYPE_* values below.
 * 20 +----------------|
 *    | reserved       |
 * 24 +----------------+
 *
 * The address and sizes are always a 64bit little endian unsigned integer.
 *
 * NB: Xen on x86 will always try to place all the data below the 4GiB
 * boundary.
 *
 * Version numbers of the hvm_start_info structure have evolved like this:
 *
 * Version 0:  Initial implementation.
 *
 * Version 1:  Added the memmap_paddr/memmap_entries fields (plus 4 bytes of
 *             padding) to the end of the hvm_start_info struct. These new
 *             fields can be used to pass a memory map to the guest. The
 *             memory map is optional and so guests that understand version 1
 *             of the structure must check that memmap_entries is non-zero
 *             before trying to read the memory map.
 */

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct HvmStartInfo {
    pub magic: u32,
    pub version: u32,
    pub flags: u32,
    pub nr_modules: u32,
    pub modlist_paddr: u64,
    pub cmdline_paddr: u64,
    pub rsdp_paddr: u64,
    pub memmap_paddr: u64,
    pub memmap_entries: u32,
    pub reserved: u32,
}

impl HvmStartInfo {
    pub const XEN_HVM_START_MAGIC_VALUE: u32 = 0x336ec578;
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct HvmModlistEntry {
    pub paddr: u64,
    pub size: u64,
    pub cmdline_paddr: u64,
    pub reserved: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct HvmMemmapTableEntry {
    pub addr: u64,
    pub size: u64,
    pub type_: u32,
    pub reserved: u32,
}

/// The E820 types known to the kernel.
#[derive(Copy, Clone, Debug)]
#[repr(u32)]
pub enum E820Type {
    Ram = 1,
    Reserved = 2,
    Acpi = 3,
    Nvs = 4,
    Unusable = 5,
    Pmem = 7,
    Pram = 12,
    SoftReserved = 0xefffffff,
    ReservedKern = 128,
}

impl From<u32> for E820Type {
    fn from(val: u32) -> Self {
        match val {
            1 => E820Type::Ram,
            2 => E820Type::Reserved,
            3 => E820Type::Acpi,
            4 => E820Type::Nvs,
            5 => E820Type::Unusable,
            7 => E820Type::Pmem,
            12 => E820Type::Pram,
            0xefffffff => E820Type::SoftReserved,
            128 => E820Type::ReservedKern,
            _ => E820Type::Reserved,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindgen_test_layout_hvm_start_info() {
        const UNINIT: ::std::mem::MaybeUninit<HvmStartInfo> = ::std::mem::MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            ::std::mem::size_of::<HvmStartInfo>(),
            56usize,
            concat!("Size of: ", stringify!(hvm_start_info))
        );
        assert_eq!(
            ::std::mem::align_of::<HvmStartInfo>(),
            8usize,
            concat!("Alignment of ", stringify!(hvm_start_info))
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).magic) as usize - ptr as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(magic)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).version) as usize - ptr as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(version)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).flags) as usize - ptr as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(flags)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).nr_modules) as usize - ptr as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(nr_modules)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).modlist_paddr) as usize - ptr as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(modlist_paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).cmdline_paddr) as usize - ptr as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(cmdline_paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).rsdp_paddr) as usize - ptr as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(rsdp_paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).memmap_paddr) as usize - ptr as usize },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(memmap_paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).memmap_entries) as usize - ptr as usize },
            48usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(memmap_entries)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).reserved) as usize - ptr as usize },
            52usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_start_info),
                "::",
                stringify!(reserved)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_hvm_modlist_entry() {
        const UNINIT: ::std::mem::MaybeUninit<HvmModlistEntry> = ::std::mem::MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            ::std::mem::size_of::<HvmModlistEntry>(),
            32usize,
            concat!("Size of: ", stringify!(hvm_modlist_entry))
        );
        assert_eq!(
            ::std::mem::align_of::<HvmModlistEntry>(),
            8usize,
            concat!("Alignment of ", stringify!(hvm_modlist_entry))
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).paddr) as usize - ptr as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_modlist_entry),
                "::",
                stringify!(paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).size) as usize - ptr as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_modlist_entry),
                "::",
                stringify!(size)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).cmdline_paddr) as usize - ptr as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_modlist_entry),
                "::",
                stringify!(cmdline_paddr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).reserved) as usize - ptr as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_modlist_entry),
                "::",
                stringify!(reserved)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_hvm_memmap_table_entry() {
        const UNINIT: ::std::mem::MaybeUninit<HvmMemmapTableEntry> =
            ::std::mem::MaybeUninit::uninit();
        let ptr = UNINIT.as_ptr();
        assert_eq!(
            ::std::mem::size_of::<HvmMemmapTableEntry>(),
            24usize,
            concat!("Size of: ", stringify!(hvm_memmap_table_entry))
        );
        assert_eq!(
            ::std::mem::align_of::<HvmMemmapTableEntry>(),
            8usize,
            concat!("Alignment of ", stringify!(hvm_memmap_table_entry))
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).addr) as usize - ptr as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_memmap_table_entry),
                "::",
                stringify!(addr)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).size) as usize - ptr as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_memmap_table_entry),
                "::",
                stringify!(size)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).type_) as usize - ptr as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_memmap_table_entry),
                "::",
                stringify!(type_)
            )
        );
        assert_eq!(
            unsafe { ::std::ptr::addr_of!((*ptr).reserved) as usize - ptr as usize },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(hvm_memmap_table_entry),
                "::",
                stringify!(reserved)
            )
        );
    }
}
