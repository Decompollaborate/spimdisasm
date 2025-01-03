/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::Instruction;

use crate::addresses::Rom;

use super::JrRegData;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct HiInfo {
    pub(crate) instr_rom: Rom,

    // If the previous instructions is a branch likely, then nulify
    // the effects of this instruction for future analysis
    pub(crate) set_on_branch_likely: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TrackedRegisterState {
    // Maybe wrap in Option?
    value: u32,

    // TODO: maybe wrap in an enum?
    hi_info: Option<HiInfo>,
    gp_info: Option<Rom>,
    lo_info: Option<Rom>,
    dereferenced: Option<Rom>,
    branch_info: Option<Rom>,

    contains_float: bool,
}

impl TrackedRegisterState {
    pub(crate) fn new() -> Self {
        Self {
            value: 0,
            hi_info: None,
            gp_info: None,
            lo_info: None,
            dereferenced: None,
            branch_info: None,
            contains_float: false,
        }
    }

    pub(crate) fn value(&self) -> u32 {
        self.value
    }
    pub(crate) fn hi_info(&self) -> Option<HiInfo> {
        self.hi_info
    }
    pub(crate) fn gp_info(&self) -> Option<Rom> {
        self.gp_info
    }
    pub(crate) fn lo_info(&self) -> Option<Rom> {
        self.lo_info
    }
    pub(crate) fn dereferenced(&self) -> Option<Rom> {
        self.dereferenced
    }

    pub(crate) fn get_jr_reg_data(&self) -> Option<JrRegData> {
        if self.dereferenced.is_none() {
            None
        } else {
            self.lo_info
                .map(|lo_rom| JrRegData::new(lo_rom, self.value, self.branch_info))
        }
    }

    pub(crate) fn contains_float(&self) -> bool {
        self.contains_float
    }
}

impl TrackedRegisterState {
    pub fn clear(&mut self) {
        self.value = 0;

        self.clear_hi();
        self.clear_gp();
        self.clear_lo();
        self.clear_branch();
    }

    pub fn clear_hi(&mut self) {
        self.hi_info = None;
    }
    pub fn clear_gp(&mut self) {
        self.gp_info = None;
    }
    pub fn clear_lo(&mut self) {
        self.lo_info = None;
        self.dereferenced = None;
    }
    pub fn clear_branch(&mut self) {
        self.branch_info = None;
    }

    pub fn clear_contains_float(&mut self) {
        self.contains_float = false;
    }
}

impl TrackedRegisterState {
    pub fn set_hi(&mut self, value: u32, instr_rom: Rom, prev_instr: Option<&Instruction>) {
        assert!(self.gp_info.is_none());
        self.value = value << 16;

        self.hi_info = Some(HiInfo {
            instr_rom,
            set_on_branch_likely: prev_instr
                .is_some_and(|x| x.opcode().is_branch_likely() || x.is_unconditional_branch()),
        });
        self.dereferenced = None;
        self.clear_contains_float();
    }

    pub fn set_gp_load(&mut self, value: u32, instr_rom: Rom) {
        assert!(self.hi_info.is_none());
        self.value = value;

        self.gp_info = Some(instr_rom);
        self.clear_contains_float();
    }

    pub fn set_lo(&mut self, value: u32, instr_rom: Rom) {
        self.value = value;

        self.lo_info = Some(instr_rom);
        self.dereferenced = None;
        self.clear_contains_float();
    }

    pub fn set_branching(&mut self, instr_rom: Rom) {
        self.branch_info = Some(instr_rom);
    }

    pub fn set_deref(&mut self, instr_rom: Rom) {
        self.dereferenced = Some(instr_rom);
        self.clear_contains_float();
    }

    pub fn dereference_from(&mut self, other: Self, instr_rom: Rom) {
        *self = other;
        self.set_deref(instr_rom);
    }

    pub fn set_contains_float(&mut self) {
        self.contains_float = true;
    }
}

impl TrackedRegisterState {
    pub fn has_any_value(&self) -> bool {
        self.hi_info.is_some() || self.gp_info.is_some() || self.lo_info.is_some()
    }

    // TODO: rename to was_set_by_current_instr
    pub fn was_set_in_current_instr(&self, instr_rom: Rom) -> bool {
        self.lo_info == Some(instr_rom) || self.dereferenced == Some(instr_rom)
    }
}
