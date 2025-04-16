/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{
    abi::Abi, access_type::AccessType, registers::Gpr, registers_meta::Register, vram::VramOffset,
};

use crate::{
    addresses::{GlobalOffsetTable, GotRequestedAddress, GpValue, Rom, Vram},
    config::{Endian, GpConfig},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GprRegisterValue {
    Garbage,
    HardwiredZero,
    SoftZero,
    GlobalPointer {
        gp: GpValue,
        hi_rom: Option<Rom>,
    },
    StackPointer {
        offset: i32,
    },
    GivenAddress {
        vram: Vram,
    },
    // ReturnAddress,
    // Argument,
    Hi {
        value: u32,
        rom: Rom,
    },
    HiGp {
        value: u32,
        rom: Rom,
        hi_rom: Rom,
    },
    ConstantInfo {
        setter_rom: Rom,
        info: GprRegConstantInfo,
    },
    RawAddress {
        vram: Vram,
        /// The rom address where this vram is paired.
        /// It can be either `%lo`, `%gp_rel`, `%got`, `%got_lo`, etc
        setter_rom: Rom,
        info: GprRegRawAddress,
    },
    DereferencedAddress {
        original_address: Vram,
        /// The rom where `original_address` was dereferenced.
        /// It may not be the same rom address where the `%lo` reloc is (when applicable).
        deref_rom: Rom,
        access_info: (AccessType, bool),
        info: GprRegDereferencedAddress,
    },
    DereferencedAddressBranchChecked {
        original_address: Vram,
        /// The rom where `original_address` was dereferenced.
        /// It may not be the same rom address where the `%lo` reloc is (when applicable).
        deref_rom: Rom,
        access_info: (AccessType, bool),
        info: GprRegDereferencedAddress,
    },
    DereferencedAddressAddedWithGp {
        original_address: Vram,
        /// The rom where `original_address` was dereferenced.
        /// It may not be the same rom address where the `%lo` reloc is (when applicable).
        deref_rom: Rom,
        access_info: (AccessType, bool),
        info: GprRegDereferencedAddress,
    },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GprRegConstantInfo {
    /// ```mips
    /// lui         $reg, 0xXXXX
    /// ori         $reg2, $reg, 0xYYYY
    /// ```
    Constant { value: u32, hi_rom: Rom },
    /// ```mips
    /// ori         $reg, $zero, 0xXXXX
    /// ```
    SmallConstant { value: u16 },
    /// ```mips
    /// lui         $reg, 0xXXXX
    /// or          $reg2, $reg, $reg3
    /// ```
    OredHi { value: u32, hi_rom: Rom },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GprRegRawAddress {
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// addiu       $reg2, $reg, %lo(EXAMPLE)
    /// ```
    HiLo { hi_rom: Rom },
    /// ```mips
    /// addiu       $reg, $gp, %gp_rel(EXAMPLE)
    /// ```
    GpRel {},
    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// ```
    GpGotGlobal {},
    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// ```
    GpGotLazyResolver {},
    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// ```
    GpGotLocal {},
    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// addiu       $reg2, $reg, %lo(EXAMPLE)
    /// ```
    PairedGpGotLo { upper_rom: Rom },
    /// ```mips
    /// lui         $reg, %got_hi(EXAMPLE)
    /// addu        $reg2, $reg, $gp
    /// lw          $reg3, %got_lo(EXAMPLE)($reg2)
    /// ```
    HiLoGp { hi_rom: Rom },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GprRegDereferencedAddress {
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// lw          $reg2, %lo(EXAMPLE)($reg)
    /// ```
    Hi { hi_rom: Rom },

    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// addiu       $reg2, $reg, %lo(EXAMPLE)
    /// lw          $reg3, 0xXXXX($reg2)
    /// ```
    HiLo { lo_rom: Rom, addend: i16 },

    /// ```mips
    /// lw       $reg, %gp_rel(EXAMPLE)($gp)
    /// ```
    GpRel {},

    /// ```mips
    /// addiu       $reg, $gp, %gp_rel(EXAMPLE)
    /// lw          $reg2, 0xXXXX($reg)
    /// ```
    RawGpRel { lo_rom: Rom, addend: i16 },

    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// lw          $reg2, 0xXXXX($reg)
    /// ```
    GpGotGlobal { upper_rom: Rom, addend: i16 },

    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// lw          $reg2, %lo(EXAMPLE)($reg)
    /// ```
    GpGotLocal { upper_rom: Rom },

    /// ```mips
    /// lw          $reg, %got(EXAMPLE)($gp)
    /// addiu       $reg2, $reg, %lo(EXAMPLE)
    /// lw          $reg3, 0xXXXX($reg2)
    /// ```
    PairedGpGotLo { lo_rom: Rom, addend: i16 },

    /// ```mips
    /// lui         $reg, %got_hi(EXAMPLE)
    /// addu        $reg2, $reg, $gp
    /// lw          $reg3, %got_lo(EXAMPLE)($reg2)
    /// lw          $reg4, 0xXXXX($reg3)
    /// ```
    HiLoGp { lo_rom: Rom, addend: i16 },

    /// Unaligned word big endian:
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// lwr         $reg2, %lo(EXAMPLE + 0x3)($reg)
    /// ```
    ///
    /// Unaligned doubleword big endian:
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// ldr         $reg2, %lo(EXAMPLE + 0x7)($reg)
    /// ```
    ///
    /// Unaligned word little endian:
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// lwl         $reg2, %lo(EXAMPLE + 0x3)($reg)
    /// ```
    ///
    /// Unaligned doubleword little endian:
    /// ```mips
    /// lui         $reg, %hi(EXAMPLE)
    /// ldl         $reg2, %lo(EXAMPLE + 0x7)($reg)
    /// ```
    HiUnaligned {
        hi_rom: Rom,
        unaddended_address: Vram,
    },

    /// Unaligned word big endian:
    /// ```mips
    /// lwr         $reg, %lo(EXAMPLE + 0x3)($gp)
    /// ```
    ///
    /// Unaligned doubleword big endian:
    /// ```mips
    /// ldr         $reg, %lo(EXAMPLE + 0x7)($gp)
    /// ```
    ///
    /// Unaligned word little endian:
    /// ```mips
    /// lwl         $reg, %lo(EXAMPLE + 0x3)($gp)
    /// ```
    ///
    /// Unaligned doubleword little endian:
    /// ```mips
    /// ldl         $reg, %lo(EXAMPLE + 0x7)($gp)
    /// ```
    GpRelUnaligned { unaddended_address: Vram },
}

impl GprRegisterValue {
    pub(crate) fn new(
        reg: Gpr,
        abi: Abi,
        function_address: Option<Vram>,
        gp_config: Option<GpConfig>,
    ) -> Self {
        if reg.is_zero(abi) {
            Self::HardwiredZero
        } else if reg.is_global_pointer(abi) {
            if let Some(gp_config) = gp_config {
                Self::GlobalPointer {
                    gp: gp_config.gp_value(),
                    hi_rom: None,
                }
            } else {
                Self::Garbage
            }
        } else if reg.is_stack_pointer(abi) {
            Self::StackPointer { offset: 0 }
        } else if reg == Gpr::t9 {
            if let (Some(function_address), true) =
                (function_address, gp_config.is_some_and(|x| x.pic()))
            {
                Self::GivenAddress {
                    vram: function_address,
                }
            } else {
                Self::Garbage
            }
        } else {
            Self::Garbage
        }
    }

    pub(crate) fn apply_branch(&mut self) {
        match self {
            Self::DereferencedAddress {
                original_address,
                deref_rom,
                access_info,
                info,
            }
            | Self::DereferencedAddressBranchChecked {
                original_address,
                deref_rom,
                access_info,
                info,
            } => {
                *self = Self::DereferencedAddressBranchChecked {
                    original_address: *original_address,
                    deref_rom: *deref_rom,
                    access_info: *access_info,
                    info: *info,
                }
            }

            Self::Garbage
            | Self::HardwiredZero
            | Self::SoftZero
            | Self::GlobalPointer { .. }
            | Self::StackPointer { .. }
            | Self::GivenAddress { .. }
            | Self::Hi { .. }
            | Self::HiGp { .. }
            | Self::ConstantInfo { .. }
            | Self::RawAddress { .. }
            | Self::DereferencedAddressAddedWithGp { .. } => {}
        }
    }

    pub(crate) fn add_imm16(
        &self,
        imm: i16,
        current_rom: Rom,
        gp_config: Option<&GpConfig>,
        dst_reg: Gpr,
    ) -> Self {
        match self {
            Self::HardwiredZero | Self::SoftZero => {
                if imm == 0 {
                    Self::SoftZero
                } else {
                    Self::Garbage
                }
            }
            Self::GlobalPointer { gp, .. } => Self::RawAddress {
                vram: Vram::new(gp.inner().wrapping_add_signed(imm.into())),
                setter_rom: current_rom,
                info: GprRegRawAddress::GpRel {},
            },
            Self::StackPointer { offset } => Self::StackPointer {
                offset: offset.wrapping_add(imm.into()),
            },
            Self::Hi { value, rom } => {
                let new_value = value.wrapping_add_signed(imm.into());
                if imm.is_negative() && imm.unsigned_abs() as u32 > *value {
                    Self::Garbage
                } else if gp_config.map(|x| x.gp_value().inner()) == Some(new_value)
                    && dst_reg == Gpr::gp
                {
                    Self::GlobalPointer {
                        gp: GpValue::new(new_value),
                        hi_rom: Some(*rom),
                    }
                } else {
                    Self::RawAddress {
                        vram: Vram::new(new_value),
                        setter_rom: current_rom,
                        info: GprRegRawAddress::HiLo { hi_rom: *rom },
                    }
                }
            }
            Self::RawAddress {
                vram,
                setter_rom,
                info,
            } => match info {
                GprRegRawAddress::GpGotLocal {} => Self::RawAddress {
                    vram: Vram::new(vram.inner().wrapping_add_signed(imm.into())),
                    setter_rom: current_rom,
                    info: GprRegRawAddress::PairedGpGotLo {
                        upper_rom: *setter_rom,
                    },
                },
                GprRegRawAddress::HiLo { .. }
                | GprRegRawAddress::GpRel { .. }
                | GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::PairedGpGotLo { .. }
                | GprRegRawAddress::HiLoGp { .. } => Self::Garbage,
            },

            Self::Garbage
            | Self::GivenAddress { .. }
            | Self::HiGp { .. }
            | Self::ConstantInfo { .. }
            | Self::DereferencedAddress { .. }
            | Self::DereferencedAddressBranchChecked { .. }
            | Self::DereferencedAddressAddedWithGp { .. } => Self::Garbage,
        }
    }

    pub(crate) fn dereference(
        &self,
        imm: i16,
        current_rom: Rom,
        access_info: (AccessType, bool),
        global_offset_table: Option<&GlobalOffsetTable>,
        endian: Endian,
    ) -> Self {
        match self {
            Self::GlobalPointer { gp, .. } => {
                let vram = Vram::new(gp.inner().wrapping_add_signed(imm.into()));
                if let Some(requested_address) =
                    global_offset_table.and_then(|x| x.request_address(vram))
                {
                    let new_address = Vram::new(requested_address.address());
                    match requested_address {
                        GotRequestedAddress::LazyResolver(_) => Self::RawAddress {
                            vram: new_address,
                            setter_rom: current_rom,
                            info: GprRegRawAddress::GpGotLazyResolver {},
                        },
                        GotRequestedAddress::Local(_) => Self::RawAddress {
                            vram: new_address,
                            setter_rom: current_rom,
                            info: GprRegRawAddress::GpGotLocal {},
                        },
                        GotRequestedAddress::Global(_) => Self::RawAddress {
                            vram: new_address,
                            setter_rom: current_rom,
                            info: GprRegRawAddress::GpGotGlobal {},
                        },
                    }
                } else {
                    let info = match (access_info.0, endian) {
                        (AccessType::UNALIGNED_WORD_LEFT, Endian::Little) => {
                            GprRegDereferencedAddress::GpRelUnaligned {
                                unaddended_address: vram + VramOffset::new(-0x3),
                            }
                        }
                        (AccessType::UNALIGNED_WORD_RIGHT, Endian::Big) => {
                            GprRegDereferencedAddress::GpRelUnaligned {
                                unaddended_address: vram + VramOffset::new(-0x3),
                            }
                        }
                        (AccessType::UNALIGNED_DOUBLEWORD_LEFT, Endian::Little) => {
                            GprRegDereferencedAddress::GpRelUnaligned {
                                unaddended_address: vram + VramOffset::new(-0x7),
                            }
                        }
                        (AccessType::UNALIGNED_DOUBLEWORD_RIGHT, Endian::Big) => {
                            GprRegDereferencedAddress::GpRelUnaligned {
                                unaddended_address: vram + VramOffset::new(-0x7),
                            }
                        }
                        (_, _) => GprRegDereferencedAddress::GpRel {},
                    };

                    Self::DereferencedAddress {
                        original_address: vram,
                        deref_rom: current_rom,
                        access_info,
                        info,
                    }
                }
            }

            Self::StackPointer { .. } => {
                // Currently we do not track stack operations.
                // Maybe we should?
                Self::Garbage
            }

            Self::Hi { value, rom } => {
                let original_address = Vram::new(value.wrapping_add_signed(imm.into()));
                let info = match (access_info.0, endian) {
                    (AccessType::UNALIGNED_WORD_LEFT, Endian::Little) => {
                        GprRegDereferencedAddress::HiUnaligned {
                            hi_rom: *rom,
                            unaddended_address: original_address + VramOffset::new(-0x3),
                        }
                    }
                    (AccessType::UNALIGNED_WORD_RIGHT, Endian::Big) => {
                        GprRegDereferencedAddress::HiUnaligned {
                            hi_rom: *rom,
                            unaddended_address: original_address + VramOffset::new(-0x3),
                        }
                    }
                    (AccessType::UNALIGNED_DOUBLEWORD_LEFT, Endian::Little) => {
                        GprRegDereferencedAddress::HiUnaligned {
                            hi_rom: *rom,
                            unaddended_address: original_address + VramOffset::new(-0x7),
                        }
                    }
                    (AccessType::UNALIGNED_DOUBLEWORD_RIGHT, Endian::Big) => {
                        GprRegDereferencedAddress::HiUnaligned {
                            hi_rom: *rom,
                            unaddended_address: original_address + VramOffset::new(-0x7),
                        }
                    }
                    (_, _) => GprRegDereferencedAddress::Hi { hi_rom: *rom },
                };

                Self::DereferencedAddress {
                    original_address,
                    deref_rom: current_rom,
                    access_info,
                    info,
                }
            }

            Self::HiGp { value, hi_rom, .. } => {
                let vram = Vram::new(value.wrapping_add_signed(imm.into()));
                if let Some(requested_address) =
                    global_offset_table.and_then(|x| x.request_address(vram))
                {
                    let new_address = Vram::new(requested_address.address());
                    Self::RawAddress {
                        vram: new_address,
                        setter_rom: current_rom,
                        info: GprRegRawAddress::HiLoGp { hi_rom: *hi_rom },
                    }
                } else {
                    Self::Garbage
                }
            }

            Self::RawAddress {
                vram,
                setter_rom,
                info,
            } => match info {
                GprRegRawAddress::HiLo { .. } => Self::DereferencedAddress {
                    original_address: *vram,
                    deref_rom: current_rom,
                    access_info,
                    info: GprRegDereferencedAddress::HiLo {
                        lo_rom: *setter_rom,
                        addend: imm,
                    },
                },
                GprRegRawAddress::GpRel { .. } => Self::DereferencedAddress {
                    original_address: *vram,
                    deref_rom: current_rom,
                    access_info,
                    info: GprRegDereferencedAddress::RawGpRel {
                        lo_rom: *setter_rom,
                        addend: imm,
                    },
                },
                GprRegRawAddress::GpGotGlobal { .. } => Self::DereferencedAddress {
                    original_address: *vram,
                    deref_rom: current_rom,
                    access_info,
                    info: GprRegDereferencedAddress::GpGotGlobal {
                        addend: imm,
                        upper_rom: *setter_rom,
                    },
                },
                GprRegRawAddress::GpGotLazyResolver { .. } => Self::Garbage,
                GprRegRawAddress::GpGotLocal {} => {
                    let original_address = Vram::new(vram.inner().wrapping_add_signed(imm.into()));
                    Self::DereferencedAddress {
                        original_address,
                        deref_rom: current_rom,
                        access_info,
                        info: GprRegDereferencedAddress::GpGotLocal {
                            upper_rom: *setter_rom,
                        },
                    }
                }
                GprRegRawAddress::PairedGpGotLo { .. } => Self::DereferencedAddress {
                    original_address: *vram,
                    deref_rom: current_rom,
                    access_info,
                    info: GprRegDereferencedAddress::PairedGpGotLo {
                        addend: imm,
                        lo_rom: *setter_rom,
                    },
                },
                GprRegRawAddress::HiLoGp { .. } => Self::DereferencedAddress {
                    original_address: *vram,
                    deref_rom: current_rom,
                    access_info,
                    info: GprRegDereferencedAddress::HiLoGp {
                        addend: imm,
                        lo_rom: *setter_rom,
                    },
                },
            },

            Self::Garbage
            | Self::HardwiredZero
            | Self::SoftZero
            | Self::GivenAddress { .. }
            | Self::ConstantInfo { .. }
            | Self::DereferencedAddress { .. }
            | Self::DereferencedAddressBranchChecked { .. }
            | Self::DereferencedAddressAddedWithGp { .. } => Self::Garbage,
        }
    }

    pub(crate) fn or_imm16(&self, imm: u16, current_rom: Rom) -> Self {
        match self {
            Self::HardwiredZero | Self::SoftZero => {
                if imm == 0 {
                    Self::SoftZero
                } else {
                    Self::ConstantInfo {
                        setter_rom: current_rom,
                        info: GprRegConstantInfo::SmallConstant { value: imm },
                    }
                }
            }

            Self::Hi { value, rom: hi_rom }
            | Self::ConstantInfo {
                info: GprRegConstantInfo::OredHi { value, hi_rom, .. },
                ..
            } => Self::ConstantInfo {
                setter_rom: current_rom,
                info: GprRegConstantInfo::Constant {
                    value: *value | imm as u32,
                    hi_rom: *hi_rom,
                },
            },

            Self::Garbage
            | Self::GlobalPointer { .. }
            | Self::StackPointer { .. }
            | Self::GivenAddress { .. }
            | Self::HiGp { .. }
            | Self::ConstantInfo { .. }
            | Self::RawAddress { .. }
            | Self::DereferencedAddress { .. }
            | Self::DereferencedAddressBranchChecked { .. }
            | Self::DereferencedAddressAddedWithGp { .. } => Self::Garbage,
        }
    }

    pub(crate) fn add_register(
        &self,
        other: &Self,
        current_rom: Rom,
        gp_config: Option<&GpConfig>,
    ) -> Self {
        // Addition must be symmetrical
        match (self, other) {
            (Self::HardwiredZero | Self::SoftZero, Self::HardwiredZero | Self::SoftZero) => {
                Self::SoftZero
            }

            // Adding zero to something gives something
            (x, Self::HardwiredZero | Self::SoftZero) => *x,
            (Self::HardwiredZero | Self::SoftZero, y) => *y,

            // Adding garbage to something gives something
            // We do this because a very typical pattern is to add "garbage" to something else as a
            // way to index into an array or offset into an struct
            (x, Self::Garbage) => *x,
            (Self::Garbage, y) => *y,

            // Prioritize the pointer-ness
            (
                Self::Hi { .. } | Self::HiGp { .. } | Self::RawAddress { .. },
                Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
            ) => *self,
            (
                Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
                Self::Hi { .. } | Self::HiGp { .. } | Self::RawAddress { .. },
            ) => *other,

            // $gp stuff
            (Self::GlobalPointer { .. }, Self::GlobalPointer { .. }) => Self::Garbage,
            (Self::GlobalPointer { gp, .. }, Self::Hi { value, rom }) => Self::HiGp {
                value: gp.inner().wrapping_add(*value),
                rom: current_rom,
                hi_rom: *rom,
            },
            (Self::Hi { value, rom }, Self::GlobalPointer { gp, .. }) => Self::HiGp {
                value: gp.inner().wrapping_add(*value),
                rom: current_rom,
                hi_rom: *rom,
            },
            // gp + dereferenced => DereferencedAddressAddedWithGp
            (
                Self::GlobalPointer { .. },
                Self::DereferencedAddress {
                    original_address,
                    deref_rom,
                    access_info,
                    info,
                    ..
                }
                | Self::DereferencedAddressBranchChecked {
                    original_address,
                    deref_rom,
                    access_info,
                    info,
                    ..
                },
            ) => Self::DereferencedAddressAddedWithGp {
                original_address: *original_address,
                deref_rom: *deref_rom,
                access_info: *access_info,
                info: *info,
            },
            (
                Self::DereferencedAddress {
                    original_address,
                    deref_rom,
                    access_info,
                    info,
                    ..
                }
                | Self::DereferencedAddressBranchChecked {
                    original_address,
                    deref_rom,
                    access_info,
                    info,
                    ..
                },
                Self::GlobalPointer { .. },
            ) => Self::DereferencedAddressAddedWithGp {
                original_address: *original_address,
                deref_rom: *deref_rom,
                access_info: *access_info,
                info: *info,
            },
            (
                Self::GlobalPointer { .. },
                Self::StackPointer { .. }
                | Self::GivenAddress { .. }
                | Self::HiGp { .. }
                | Self::ConstantInfo { .. }
                | Self::RawAddress { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
            ) => Self::Garbage,
            (
                Self::StackPointer { .. }
                | Self::GivenAddress { .. }
                | Self::HiGp { .. }
                | Self::ConstantInfo { .. }
                | Self::RawAddress { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
                Self::GlobalPointer { .. },
            ) => Self::Garbage,

            // Stackpointer
            (Self::StackPointer { .. }, Self::StackPointer { .. }) => Self::Garbage,
            (Self::StackPointer { offset }, Self::Hi { value, .. }) => Self::StackPointer {
                offset: offset.wrapping_add_unsigned(*value),
            },
            (Self::Hi { value, .. }, Self::StackPointer { offset }) => Self::StackPointer {
                offset: offset.wrapping_add_unsigned(*value),
            },
            (Self::StackPointer { offset }, Self::RawAddress { vram, info, .. }) => match info {
                GprRegRawAddress::HiLo { .. } => Self::StackPointer {
                    offset: offset.wrapping_add_unsigned(vram.inner()),
                },
                GprRegRawAddress::GpRel {}
                | GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::GpGotLocal { .. }
                | GprRegRawAddress::PairedGpGotLo { .. }
                | GprRegRawAddress::HiLoGp { .. } => Self::Garbage,
            },
            (Self::RawAddress { vram, info, .. }, Self::StackPointer { offset }) => match info {
                GprRegRawAddress::HiLo { .. } => Self::StackPointer {
                    offset: offset.wrapping_add_unsigned(vram.inner()),
                },
                GprRegRawAddress::GpRel {}
                | GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::GpGotLocal { .. }
                | GprRegRawAddress::PairedGpGotLo { .. }
                | GprRegRawAddress::HiLoGp { .. } => Self::Garbage,
            },
            (
                Self::StackPointer { .. },
                Self::GivenAddress { .. }
                | Self::HiGp { .. }
                | Self::ConstantInfo { .. }
                | Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
            ) => Self::Garbage,
            (
                Self::GivenAddress { .. }
                | Self::HiGp { .. }
                | Self::ConstantInfo { .. }
                | Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
                Self::StackPointer { .. },
            ) => Self::Garbage,

            // GivenAddress
            // Adding a GivenAddress with most types of addresses or dereferenced addresses should
            // give back garbage.
            (Self::GivenAddress { .. }, Self::GivenAddress { .. }) => Self::Garbage,
            (
                Self::GivenAddress {
                    vram: given_address,
                },
                Self::RawAddress { vram, info, .. },
            ) => match info {
                GprRegRawAddress::HiLo { hi_rom, .. } => {
                    let result = given_address.inner().wrapping_add(vram.inner());
                    if gp_config.map(|x| x.gp_value().inner()) == Some(result) {
                        Self::GlobalPointer {
                            gp: GpValue::new(result),
                            hi_rom: Some(*hi_rom),
                        }
                    } else {
                        Self::Garbage
                    }
                }
                GprRegRawAddress::GpRel {}
                | GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::GpGotLocal { .. }
                | GprRegRawAddress::PairedGpGotLo { .. }
                | GprRegRawAddress::HiLoGp { .. } => Self::Garbage,
            },
            (
                Self::RawAddress { vram, info, .. },
                Self::GivenAddress {
                    vram: given_address,
                },
            ) => match info {
                GprRegRawAddress::HiLo { hi_rom, .. } => {
                    let result = given_address.inner().wrapping_add(vram.inner());
                    if gp_config.map(|x| x.gp_value().inner()) == Some(result) {
                        Self::GlobalPointer {
                            gp: GpValue::new(result),
                            hi_rom: Some(*hi_rom),
                        }
                    } else {
                        Self::Garbage
                    }
                }
                GprRegRawAddress::GpRel {}
                | GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::GpGotLocal { .. }
                | GprRegRawAddress::PairedGpGotLo { .. }
                | GprRegRawAddress::HiLoGp { .. } => Self::Garbage,
            },
            (
                Self::GivenAddress { .. },
                Self::Hi { .. }
                | Self::HiGp { .. }
                | Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. }
                | Self::ConstantInfo { .. },
            ) => Self::Garbage,
            (
                Self::Hi { .. }
                | Self::HiGp { .. }
                | Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. }
                | Self::ConstantInfo { .. },
                Self::GivenAddress { .. },
            ) => Self::Garbage,

            // Adding stuff to constants gives back garbage
            (Self::ConstantInfo { .. }, _) => Self::Garbage,
            (_, Self::ConstantInfo { .. }) => Self::Garbage,

            (Self::Hi { .. }, Self::Hi { .. }) => Self::Garbage,
            (Self::Hi { .. }, Self::HiGp { .. } | Self::RawAddress { .. }) => Self::Garbage,
            (Self::HiGp { .. } | Self::RawAddress { .. }, Self::Hi { .. }) => Self::Garbage,

            (Self::HiGp { .. }, Self::HiGp { .. }) => Self::Garbage,
            (Self::HiGp { .. }, Self::RawAddress { .. }) => Self::Garbage,
            (Self::RawAddress { .. }, Self::HiGp { .. }) => Self::Garbage,

            (Self::RawAddress { .. }, Self::RawAddress { .. }) => Self::Garbage,

            // Adding two dereferenced values together gives garbage
            (
                Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
                Self::DereferencedAddress { .. }
                | Self::DereferencedAddressBranchChecked { .. }
                | Self::DereferencedAddressAddedWithGp { .. },
            ) => Self::Garbage,
        }
    }

    pub(crate) fn sub_register(&self, other: &Self, _current_rom: Rom) -> Self {
        match (self, other) {
            (Self::HardwiredZero, Self::HardwiredZero | Self::SoftZero) => Self::SoftZero,
            (_, Self::HardwiredZero | Self::SoftZero) => *self,

            (Self::StackPointer { offset }, Self::ConstantInfo { info, .. }) => match info {
                GprRegConstantInfo::Constant { value, .. } => Self::StackPointer {
                    offset: offset.wrapping_sub_unsigned(*value),
                },
                GprRegConstantInfo::SmallConstant { value, .. } => Self::StackPointer {
                    offset: offset.wrapping_sub_unsigned((*value).into()),
                },
                GprRegConstantInfo::OredHi { .. } => Self::Garbage,
            },

            (_, _) => Self::Garbage,
        }
    }

    pub(crate) fn or_register(&self, other: &Self, current_rom: Rom) -> Self {
        // Or'ing must be symmetrical
        match (self, other) {
            (Self::HardwiredZero | Self::SoftZero, Self::HardwiredZero | Self::SoftZero) => {
                Self::SoftZero
            }

            // zero to something gives something
            (x, Self::HardwiredZero | Self::SoftZero) => *x,
            (Self::HardwiredZero | Self::SoftZero, y) => *y,

            (
                Self::Hi { value, rom },
                Self::Garbage
                | Self::DereferencedAddress { .. }
                | Self::ConstantInfo {
                    info: GprRegConstantInfo::SmallConstant { .. },
                    ..
                },
            ) => Self::ConstantInfo {
                setter_rom: current_rom,
                info: GprRegConstantInfo::OredHi {
                    value: *value,
                    hi_rom: *rom,
                },
            },
            (
                Self::Garbage
                | Self::DereferencedAddress { .. }
                | Self::ConstantInfo {
                    info: GprRegConstantInfo::SmallConstant { .. },
                    ..
                },
                Self::Hi { value, rom },
            ) => Self::ConstantInfo {
                setter_rom: current_rom,
                info: GprRegConstantInfo::OredHi {
                    value: *value,
                    hi_rom: *rom,
                },
            },

            (_, _) => Self::Garbage,
        }
    }
}
