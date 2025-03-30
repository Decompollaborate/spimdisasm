/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{
    opcodes::Opcode, registers::Gpr, registers_meta::Register, vram::VramOffset, Instruction,
};

use crate::{
    addresses::{GlobalOffsetTable, GotRequestedAddress, GpValue, Rom, Vram},
    config::GpConfig,
};

use super::{tracked_register_state::HiInfo, JrRegData, LoPairingInfo, TrackedRegisterState};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RegisterTracker {
    registers: [TrackedRegisterState; Gpr::count()],
}

impl RegisterTracker {
    pub(crate) fn new() -> Self {
        Self {
            registers: [TrackedRegisterState::new(); Gpr::count()],
        }
    }
}

impl RegisterTracker {
    // For debugging
    #[allow(dead_code)]
    pub(crate) fn get(&self, reg: Gpr) -> &TrackedRegisterState {
        &self.registers[reg.as_index()]
    }

    fn clear(&mut self) {
        self.registers.iter_mut().for_each(|state| state.clear());
    }

    fn unset_registers_after_func_call(&mut self, prev_instr: &Instruction) {
        if !prev_instr.is_function_call() {
            return;
        }

        for reg in Gpr::iter() {
            if reg.is_clobbered_by_func_call(prev_instr.abi()) {
                self.registers[reg.as_index()].clear();
            }
        }
    }

    fn process_branch(&mut self, instr: &Instruction, instr_rom: Rom) {
        assert!(instr.get_branch_offset_generic().is_some());

        if let Some(reg) = instr.field_rs() {
            if instr.opcode().reads_rs() {
                self.registers[reg.as_index()].set_branching(instr_rom);
            }
        }
        if let Some(reg) = instr.field_rt() {
            if instr.opcode().reads_rt() {
                self.registers[reg.as_index()].set_branching(instr_rom);
            }
        }
        if let Some(reg) = instr.field_rd() {
            if instr.opcode().reads_rd() {
                self.registers[reg.as_index()].set_branching(instr_rom);
            }
        }
    }

    pub(crate) fn get_jr_reg_data(&self, instr: &Instruction) -> Option<JrRegData> {
        if instr.opcode().jumps_to_register() {
            let rs = instr.field_rs();
            rs.and_then(|reg| self.registers[reg.as_index()].get_jr_reg_data())
        } else {
            None
        }
    }

    pub(crate) fn get_jr_raw_reg_data(&self, instr: &Instruction) -> Option<JrRegData> {
        if instr.opcode().jumps_to_register() {
            let rs = instr.field_rs();
            rs.and_then(|reg| self.registers[reg.as_index()].get_jr_raw_reg_data())
        } else {
            None
        }
    }

    fn process_hi(&mut self, instr: &Instruction, instr_rom: Rom) -> (Gpr, u32) {
        assert!(instr.opcode().can_be_hi());

        let reg = instr
            .get_destination_gpr()
            .expect("lui should have dst register");
        let state = &mut self.registers[reg.as_index()];

        state.clear();

        let imm = instr
            .get_processed_immediate()
            .expect("lui should have an immediate field") as u32;
        state.set_hi(imm, instr_rom);

        (reg, imm << 16)
    }

    fn process_gp_load(&mut self, instr: &Instruction, instr_rom: Rom) {
        assert!(instr.opcode().can_be_lo());

        let reg = instr
            .get_destination_gpr()
            .expect("should have dst register");
        let state = &mut self.registers[reg.as_index()];

        state.clear();
        state.set_gp_load(
            instr
                .get_processed_immediate()
                .expect("should have immediate field") as i16,
            instr_rom,
        );
    }

    fn get_hi_info_for_constant(&self, instr: &Instruction) -> Option<HiInfo> {
        if let Some(rs) = instr.field_rs() {
            self.registers[rs.as_index()].hi_info()
        } else {
            None
        }
    }

    fn process_constant(&mut self, instr: &Instruction, value: u32, instr_rom: Rom) {
        if let Some(dst_reg) = instr.get_destination_gpr() {
            let state = &mut self.registers[dst_reg.as_index()];

            state.set_lo(value, instr_rom);
        }
    }

    fn process_lo(
        &mut self,
        instr: &Instruction,
        value: u32,
        instr_rom: Rom,
        is_got_deref: bool,
        is_got_global: bool,
        is_got_global_addend: bool,
    ) {
        if let Some(dst_reg) = instr.get_destination_gpr() {
            let state = &mut self.registers[dst_reg.as_index()];
            if !is_got_global_addend {
                state.set_lo(value, instr_rom);
            }
            if instr.opcode().does_dereference() && !is_got_deref {
                state.set_deref(instr_rom);
            }
            state.clear_hi();
            if Some(dst_reg) == instr.field_rs() {
                state.clear_gp();
            }
            state.clear_branch();
            if is_got_global {
                state.set_got_global(instr_rom);
            }
        }
    }

    pub(crate) fn get_address_if_instr_can_set_type(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> Option<u32> {
        if let Some(rs) = instr.field_rs() {
            let state = &self.registers[rs.as_index()];

            if state.lo_info().is_some()
                && state.dereferenced().is_none_or(|x| x == instr_rom)
                && state.gp_info().is_none()
            {
                Some(state.value())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn overwrite_registers(&mut self, instr: &Instruction, instr_rom: Rom) {
        if self.move_register(instr) {
            return;
        }

        match instr.opcode() {
            Opcode::core_mtc1 | Opcode::core_dmtc1 | Opcode::core_ctc1 => {
                // IDO usually use a reg as a temp when loading a constant value
                // into the float coprocessor, after that IDO never re-uses the value
                // in that reg for anything else
                self.clear_reg(
                    instr.field_rt().expect("This should not panic"),
                    instr,
                    instr_rom,
                );
            }
            _ => {
                if let Some(reg) = instr.get_destination_gpr() {
                    if instr.opcode().can_be_hi() {
                        self.registers[reg.as_index()].clear_lo();
                    } else {
                        self.clear_reg(reg, instr, instr_rom);
                    }
                }
            }
        }
    }

    #[must_use]
    fn preprocess_lo_and_get_info(
        &mut self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> Option<LoPairingInfo> {
        if let Some(reg) = instr.field_rs() {
            let state = &self.registers[reg.as_index()];

            if let Some(hi_info) = state.hi_info() {
                return Some(LoPairingInfo {
                    instr_rom: hi_info.instr_rom,
                    value: state.value() as i64,
                    is_gp_rel: false,
                    is_gp_got: false,
                    upper_imm: Some(hi_info.upper_imm.into()),
                    upper_is_got_global: false,
                });
            } else if reg.is_global_pointer(instr.abi()) {
                return Some(LoPairingInfo {
                    instr_rom: Rom::new(0),
                    value: state.value() as i64,
                    is_gp_rel: true,
                    is_gp_got: false,
                    upper_imm: None,
                    upper_is_got_global: false,
                });
            } else if let Some(gp_info) = state.gp_info() {
                return Some(LoPairingInfo {
                    instr_rom: gp_info.instr_rom,
                    value: state.value() as i64,
                    is_gp_rel: false,
                    is_gp_got: true,
                    upper_imm: Some(gp_info.upper_imm.into()),
                    upper_is_got_global: state.got_global().is_some(),
                });
            }

            if let Some(rt) = instr.field_rt() {
                if instr.opcode().does_dereference()
                    && state.lo_info().is_some()
                    && state.dereferenced().is_none()
                {
                    // Simulate a dereference
                    self.registers[rt.as_index()].dereference_from(*state, instr_rom);
                    self.registers[rt.as_index()].clear_branch();
                }
            }
        }

        None
    }

    pub(crate) fn get_gp_state(&self) -> &TrackedRegisterState {
        &self.registers[Gpr::gp.as_index()]
    }

    pub(crate) fn set_added_with_gp(&mut self, reg: Gpr, instr_rom: Rom) {
        self.registers[reg.as_index()].set_added_with_gp(instr_rom);
    }
}

impl RegisterTracker {
    fn move_register(&mut self, instr: &Instruction) -> bool {
        if !instr.opcode().maybe_is_move() {
            return false;
        }

        // TODO: rewrite

        // Destination register
        let rd = if let Some(rd) = instr.field_rd() {
            rd
        } else {
            return false;
        };
        let rs = if let Some(rs) = instr.field_rs() {
            rs
        } else {
            return false;
        };
        let rt = if let Some(rt) = instr.field_rt() {
            rt
        } else {
            return false;
        };

        if self.registers[rs.as_index()].contains_float()
            || self.registers[rt.as_index()].contains_float()
        {
            // Either of these registers contain a value that come from coprocessor 1 (the float coprocessor).
            // It wouldn't make sense to produce a pointer from any value that comes from that coprocessor.
            return false;
        }

        if rs.is_zero(instr.abi()) && rt.is_zero(instr.abi()) {
            return false;
        }

        if !rs.is_zero(instr.abi()) && !rt.is_zero(instr.abi()) {
            let reg = if self.registers[rs.as_index()].has_any_value()
                && !self.registers[rt.as_index()].has_any_value()
            {
                rs
            } else if !self.registers[rs.as_index()].has_any_value()
                && self.registers[rt.as_index()].has_any_value()
            {
                rt
            } else if rd == rs {
                // Check stuff like  `addu   $3, $3, $2`
                if self.registers[rs.as_index()].hi_info().is_some()
                    || self.registers[rs.as_index()].gp_info().is_some()
                {
                    rs
                } else {
                    rt
                }
            } else if rd == rt {
                if self.registers[rt.as_index()].hi_info().is_some()
                    || self.registers[rt.as_index()].gp_info().is_some()
                {
                    rt
                } else {
                    rs
                }
            } else {
                return false;
            };

            let src_state = &self.registers[reg.as_index()];

            self.registers[rd.as_index()] = *src_state;
            self.registers[rd.as_index()].clear_branch();
            return true;
        }

        let reg = if rt.is_zero(instr.abi()) { rs } else { rt };

        let src_state = &self.registers[reg.as_index()];

        if src_state.has_any_value() {
            self.registers[rd.as_index()] = *src_state;
            self.registers[rd.as_index()].clear_branch();

            true
        } else {
            self.registers[rd.as_index()].clear();

            false
        }
    }

    fn clear_reg(&mut self, reg: Gpr, instr: &Instruction, instr_rom: Rom) {
        let state = &mut self.registers[reg.as_index()];

        state.clear_hi();
        if !state.was_set_in_current_instr(instr_rom) {
            state.clear_gp();
            state.clear_lo();
            state.clear_got_global();
        }
        state.clear_branch();

        if instr.opcode().reads_fd() || instr.opcode().reads_ft() || instr.opcode().reads_fs() {
            state.set_contains_float();
        } else {
            state.clear_contains_float();
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrProcessedResult {
    /// "Branch and link" kind of instructions.
    ///
    /// These kind is usually used in handwritten assembly, so we can't expect ABI conventions to
    /// hold true, so this can branch into either a "real" function or into the middle of a
    /// function, even branching into the somewhere inside the current function!
    LinkingBranch {
        target_vram: Vram,
    },
    /// A "normal" function call to a hardcoded address. A `jal`.
    ///
    /// This is the "normal" way to call functions on statically linked code, position independent
    /// code (PIC) won't be seeing much of this.
    DirectLinkingCall {
        target_vram: Vram,
    },
    /// A suspected tail call to a hardcoded address. A `j`.
    ///
    /// This being an actual tail call is not certain since some compilers (or handwritten
    /// assembly) may use this instruction as an unconditional branch, a tail call or even both.
    MaybeDirectTailCall {
        target_vram: Vram,
    },
    /// A "Jump and link register" to a register that has been dereferenced. A `jalr`.
    ///
    /// This usually happens on arrays of function pointers, meaning we only know the address of
    /// the array but not the address of the actual function that is being called.
    DereferencedRegisterLink {
        jr_reg_data: JrRegData,
    },
    /// A "Jump and link register" to a register that contains a raw address. A `jalr`.
    ///
    /// Here we know the actual address of the function that is being called.
    RawRegisterLink {
        jr_reg_data: JrRegData,
    },
    /// A "Jump and link register", but we don't have info about what is being called. A `jalr`.
    UnknownJumpAndLinkRegister {
        reg: Gpr,
    },

    /// Jump into a `case` of a jumptable. A `jr`.
    JumptableJump {
        jr_reg_data: JrRegData,
    },
    // DereferencedRegisterTailCall {
    // },
    // RawRegisterTailCall {
    // },
    /// Jump to a register, but we don't have info about what it is pointing to. A `jr`.
    UnknownRegInfoJump {
        reg: Gpr,
    },

    /// An usual non-linking branch.
    ///
    /// This may include the `j` instruction depending on the rabbitizer `Instruction`'s flags,
    /// specifically `j_as_branch` being `true`.
    Branch {
        target_vram: Vram,
    },

    /// This instruction can set the `%hi` part of the reloc to a symbol. A `lui`.
    Hi {
        dst_reg: Gpr,
        value: u32,
    },

    PairedLo {
        hi_imm: u16,
        hi_rom: Rom,
        imm: i16,
        vram: Vram,
    },
    GpRel {
        imm: i16,
        vram: Vram,
    },
    GpGotGlobal {
        imm: i16,
        vram: Vram,
    },
    GpGotLazyResolver {
        imm: i16,
        vram: Vram,
    },
    GpGotLocal {
        imm: i16,
        vram: Vram,
    },
    PairedGpGotLo {
        upper_imm: i16,
        upper_rom: Rom,
        imm: i16,
        vram: Vram,
    },
    DanglingLo {
        imm: i16,
    },

    /// A constant value paired to a `Hi`. An `ori`.
    ///
    /// This is not covered by `PairedLo` because it can't be disassembled using a `%lo`.
    Constant {
        constant: u32,
        hi_rom: Rom,
    },
    /// An `ori` that couldn't be paired to a corresponding `Hi`.
    UnpairedConstant {
        imm: u16,
    },

    /// This instruction did not fall into any of the previous categories.
    UnhandledOpcode {
        opcode: Opcode,
    },
    /// The instruction wasn't a valid one.
    ///
    /// This was not applied into the tracker at all.
    InvalidInstr {},
}

impl RegisterTracker {
    pub(crate) fn process_instruction(
        &mut self,
        instr: &Instruction,
        instr_rom: Rom,
        global_offset_table: Option<&GlobalOffsetTable>,
        // TODO: remove these two
        original_gp_config: Option<&GpConfig>,
        current_gp_value: Option<&GpValue>,
    ) -> InstrProcessedResult {
        if !instr.is_valid() {
            return InstrProcessedResult::InvalidInstr {};
        }

        let opcode = instr.opcode();

        let out = if opcode.does_link() {
            if let Some(target_vram) = instr.get_instr_index_as_vram() {
                // InstrProcessedResult::FunctionCall { target_vram }
                InstrProcessedResult::DirectLinkingCall { target_vram }
            } else if let Some(target_vram) = instr.get_branch_vram_generic() {
                InstrProcessedResult::LinkingBranch { target_vram }
            } else {
                debug_assert!(opcode == Opcode::core_jalr);

                if let Some(jr_reg_data) = self.get_jr_reg_data(instr) {
                    InstrProcessedResult::DereferencedRegisterLink { jr_reg_data }
                } else if let Some(jr_reg_data) = self.get_jr_raw_reg_data(instr) {
                    InstrProcessedResult::RawRegisterLink { jr_reg_data }
                } else {
                    let rs = instr.field_rs().expect("");
                    InstrProcessedResult::UnknownJumpAndLinkRegister { reg: rs }
                }
            }
        } else if let Some(target_vram) = instr.get_branch_vram_generic() {
            // opcode.is_branch() or instr.is_unconditional_branch()
            self.process_branch(instr, instr_rom);
            InstrProcessedResult::Branch { target_vram }
        } else if instr.is_jumptable_jump() {
            if let Some(jr_reg_data) = self.get_jr_reg_data(instr) {
                InstrProcessedResult::JumptableJump { jr_reg_data }
            } else {
                let rs = instr.field_rs().expect("jr should have an rs field");
                InstrProcessedResult::UnknownRegInfoJump { reg: rs }
            }
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            // At this point only `j` should have an instr_index field.
            debug_assert!(opcode == Opcode::core_j);

            self.process_branch(instr, instr_rom);

            // Some compilers use `j` as an unconditional branch, as a tail call or even as both.
            // So it is hard to say if this a tail call or not, thus `Maybe`.
            InstrProcessedResult::MaybeDirectTailCall { target_vram }
        } else if opcode.can_be_hi() {
            let (dst_reg, value) = self.process_hi(instr, instr_rom);

            InstrProcessedResult::Hi { dst_reg, value }
        } else if opcode.can_be_lo() {
            let imm = instr
                .get_processed_immediate()
                .expect("This instruction should have an immediate") as i16;

            if opcode.does_load()
                && instr
                    .field_rs()
                    .is_some_and(|reg| reg.is_global_pointer(instr.abi()))
            {
                self.process_gp_load(instr, instr_rom);
            }

            if let Some(pairing_info) = self.preprocess_lo_and_get_info(instr, instr_rom) {
                if pairing_info.is_gp_got && !original_gp_config.is_some_and(|x| x.pic()) {
                    InstrProcessedResult::DanglingLo { imm }
                } else {
                    self.process_signed_lo(
                        instr,
                        instr_rom,
                        imm,
                        global_offset_table,
                        pairing_info,
                        current_gp_value,
                    )
                }
            } else {
                InstrProcessedResult::DanglingLo { imm }
            }
        } else if opcode.can_be_unsigned_lo() {
            let lower = instr
                .get_processed_immediate()
                .expect("This instruction should have an immediate") as u32;

            // Pairing with an `ori`, so we treat this as a constant.
            if let Some(hi_info) = self.get_hi_info_for_constant(instr) {
                let constant = hi_info.upper_imm | lower;
                self.process_constant(instr, constant, instr_rom);
                InstrProcessedResult::Constant {
                    constant,
                    hi_rom: hi_info.instr_rom,
                }
            } else {
                InstrProcessedResult::UnpairedConstant { imm: lower as u16 }
            }
        } else {
            InstrProcessedResult::UnhandledOpcode { opcode }
        };

        #[expect(clippy::let_and_return)]
        out
    }

    pub(crate) fn clear_afterwards(&mut self, prev_instr: Option<&Instruction>) -> bool {
        if let Some(prev) = &prev_instr {
            if prev.is_function_call() {
                self.unset_registers_after_func_call(prev);
            } else if (prev.opcode().is_jump() && !prev.opcode().does_link())
                || prev.is_unconditional_branch()
            {
                self.clear();
                return true;
            }
        }
        false
    }

    fn process_signed_lo(
        &mut self,
        instr: &Instruction,
        instr_rom: Rom,
        imm: i16,
        global_offset_table: Option<&GlobalOffsetTable>,
        pairing_info: LoPairingInfo,
        // TODO: remove these two
        current_gp_value: Option<&GpValue>,
    ) -> InstrProcessedResult {
        let upper_info = if pairing_info.is_gp_rel {
            None
        } else {
            Some((pairing_info.value, pairing_info.instr_rom))
        };

        if let Some(address) = Self::pair_hi_lo(upper_info.as_ref(), imm, current_gp_value) {
            if upper_info.is_none() {
                if let Some(got_requested_address) =
                    global_offset_table.and_then(|x| x.request_address(address))
                {
                    let new_address = got_requested_address.address();
                    let is_global = matches!(got_requested_address, GotRequestedAddress::Global(_));
                    self.process_lo(instr, new_address, instr_rom, true, is_global, false);
                    match got_requested_address {
                        GotRequestedAddress::LazyResolver(_) => {
                            InstrProcessedResult::GpGotLazyResolver {
                                imm,
                                vram: Vram::new(new_address),
                            }
                        }
                        GotRequestedAddress::Local(_) => InstrProcessedResult::GpGotLocal {
                            imm,
                            vram: Vram::new(new_address),
                        },
                        GotRequestedAddress::Global(_) => InstrProcessedResult::GpGotGlobal {
                            imm,
                            vram: Vram::new(new_address),
                        },
                    }
                } else {
                    self.process_lo(instr, address.inner(), instr_rom, false, false, false);
                    InstrProcessedResult::GpRel { imm, vram: address }
                }
            } else if let Some(upper_imm) = pairing_info.upper_imm {
                // println!("          {:?}", pairing_info);
                self.process_lo(
                    instr,
                    address.inner(),
                    instr_rom,
                    false,
                    false,
                    pairing_info.upper_is_got_global,
                );
                if pairing_info.upper_is_got_global {
                    InstrProcessedResult::DanglingLo { imm }
                } else if pairing_info.is_gp_got {
                    InstrProcessedResult::PairedGpGotLo {
                        upper_imm: upper_imm as i16,
                        upper_rom: pairing_info.instr_rom,
                        imm,
                        vram: address,
                    }
                } else {
                    InstrProcessedResult::PairedLo {
                        hi_imm: (upper_imm >> 16) as u16,
                        hi_rom: pairing_info.instr_rom,
                        imm,
                        vram: address,
                    }
                }
            } else {
                InstrProcessedResult::DanglingLo { imm }
            }
        } else {
            InstrProcessedResult::DanglingLo { imm }
        }
    }

    fn pair_hi_lo(
        upper_info: Option<&(i64, Rom)>,
        imm: i16,
        current_gp_value: Option<&GpValue>,
    ) -> Option<Vram> {
        // upper_info being None means this symbol is a $gp access

        let lower_half = VramOffset::new(imm as i32);

        if let Some((upper_half, _hi_rom)) = upper_info {
            if *upper_half < 0
                || (lower_half.is_negative()
                    && lower_half.inner().unsigned_abs() > *upper_half as u32)
            {
                None
            } else {
                Some(Vram::new(*upper_half as u32) + lower_half)
            }
        } else if let Some(gp_value) = current_gp_value {
            // TODO: implement comparison for Vram and VramOffset
            if lower_half.is_negative() && lower_half.inner().unsigned_abs() > gp_value.inner() {
                None
            } else {
                // TODO: proper abstraction
                Some(Vram::new(
                    gp_value.inner().wrapping_add_signed(lower_half.inner()),
                ))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use rabbitizer::{InstructionFlags, IsaVersion};

    use crate::{
        addresses::{GotGlobalEntry, GotLocalEntry, Size},
        config::Endian,
    };

    use super::*;

    #[test]
    fn bunch_of_pairing_combos() {
        /*
        $ mips-linux-gnu-as asdf.s -o asdf.o -KPIC
        */
        /*
        .abicalls
        .set noreorder
        .set noat

        .section .text

        .globl silly_func
        .type silly_func, @function
        silly_func:
        .ent silly_func

        lui     $gp, %hi(_gp_disp)
        addiu   $gp, $gp, %lo(_gp_disp)
        addu    $gp, $gp, $t9

        addiu   $sp, $sp, -0x20
        sw      $ra, 0x10($sp)
        sw      $gp, 0x18($sp)

        lui     $a0, %hi(some_var)
        addiu   $a1, $a0, %lo(some_var)
        lw      $a0, 0x8($a1)

        lui     $v0, %hi(some_var)
        lw      $v1, %lo(some_var+0x4)($v0)

        lw      $a2, %gp_rel(some_var+0x4)($gp)

        lw      $a3, %got(some_var+0x8)($gp)

        lw      $t0, %got(static_sym)($gp)
        lw      $t1, %lo(static_sym)($t0)

        lw      $t9, %call16(global_function)($gp)
        jalr    $t9
        nop
        lw      $gp, 0x18($sp)

        lw      $t9, %call16(global_function)($gp)
        jalr    $t9
        nop
        lw      $gp, 0x18($sp)

        lw      $t9, %got(non_global_function)($gp)
        addiu   $t9, $t9, %lo(non_global_function)
        jalr    $t9
        nop
        lw      $gp, 0x18($sp)

        lw      $t9, %got(func_arr)($gp)
        lw      $t9, 0x4($t9)
        jalr    $t9
        nop
        lw      $gp, 0x18($sp)

        lw      $ra, 0x10($sp)
        addiu   $sp, $sp, 0x20

        jr $ra
        nop
        .end silly_func
        .size silly_func, . - silly_func

        .globl global_function
        .type global_function, @function
        global_function:
        .ent global_function

        jr $ra
        nop

        .end global_function
        .size global_function, . - global_function

        .local non_global_function
        .type non_global_function, @function
        non_global_function:
        .ent non_global_function

        jr $ra
        nop

        .end non_global_function
        .size non_global_function, . - non_global_function

        .section .sdata

        .align 2

        .globl some_var
        .type some_var, @object
        some_var:
        .word 0x12345678
        .word 0x12345678
        .word 0x12345678
        .word 0x12345678
        .size some_var, .-some_var

        .local static_sym
        static_sym:
        .type static_sym, @object
        .word 0x12345678
        .word 0x12345678
        .word 0x12345678
        .word 0x12345678
        .size static_sym, .-static_sym

        .globl func_arr
        .type func_arr, @object
        func_arr:
        .word non_global_function
        .word global_function
        .word non_global_function
        .word global_function
        .size func_arr, .-func_arr

        */
        static BYTES: [u8; 37 * 4] = [
            0x3C, 0x1C, 0x00, 0x01, 0x27, 0x9C, 0x80, 0xB0, 0x03, 0x99, 0xE0, 0x21, 0x27, 0xBD,
            0xFF, 0xE0, 0xAF, 0xBF, 0x00, 0x10, 0xAF, 0xBC, 0x00, 0x18, 0x3C, 0x04, 0x80, 0x00,
            0x24, 0x85, 0x00, 0xFC, 0x8C, 0xA4, 0x00, 0x08, 0x3C, 0x02, 0x80, 0x00, 0x8C, 0x43,
            0x01, 0x00, 0x8F, 0x86, 0x80, 0x30, 0x8F, 0x87, 0x80, 0x1C, 0x8F, 0x88, 0x80, 0x18,
            0x8D, 0x09, 0x00, 0xE8, 0x8F, 0x99, 0x80, 0x20, 0x03, 0x20, 0xF8, 0x09, 0x00, 0x00,
            0x00, 0x00, 0x8F, 0xBC, 0x00, 0x18, 0x8F, 0x99, 0x80, 0x20, 0x03, 0x20, 0xF8, 0x09,
            0x00, 0x00, 0x00, 0x00, 0x8F, 0xBC, 0x00, 0x18, 0x8F, 0x99, 0x80, 0x18, 0x27, 0x39,
            0x00, 0x88, 0x03, 0x20, 0xF8, 0x09, 0x00, 0x00, 0x00, 0x00, 0x8F, 0xBC, 0x00, 0x18,
            0x8F, 0x99, 0x80, 0x24, 0x8F, 0x39, 0x00, 0x04, 0x03, 0x20, 0xF8, 0x09, 0x00, 0x00,
            0x00, 0x00, 0x8F, 0xBC, 0x00, 0x18, 0x8F, 0xBF, 0x00, 0x10, 0x27, 0xBD, 0x00, 0x20,
            0x03, 0xE0, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
        ];
        static EXPECTED_RESULTS: [InstrProcessedResult; 37] = [
            InstrProcessedResult::Hi {
                dst_reg: Gpr::gp,
                value: 0x00010000,
            },
            InstrProcessedResult::PairedLo {
                hi_imm: 0x0001,
                hi_rom: Rom::new(0x00010000),
                imm: -0x7F50,
                vram: Vram::new(0x000080B0),
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_addu,
            },
            InstrProcessedResult::DanglingLo { imm: -0x20 },
            InstrProcessedResult::DanglingLo { imm: 0x10 },
            InstrProcessedResult::DanglingLo { imm: 0x18 },
            InstrProcessedResult::Hi {
                dst_reg: Gpr::a0,
                value: 0x80000000,
            },
            InstrProcessedResult::PairedLo {
                hi_imm: 0x8000,
                hi_rom: Rom::new(0x00010018),
                imm: 0xFC,
                vram: Vram::new(0x800000FC),
            },
            InstrProcessedResult::DanglingLo { imm: 0x8 },
            InstrProcessedResult::Hi {
                dst_reg: Gpr::v0,
                value: 0x80000000,
            },
            InstrProcessedResult::PairedLo {
                hi_imm: 0x8000,
                hi_rom: Rom::new(0x00010024),
                imm: 0x100,
                vram: Vram::new(0x80000100),
            },
            InstrProcessedResult::GpRel {
                imm: -0x7FD0,
                vram: Vram::new(0x80000100),
            },
            InstrProcessedResult::GpGotGlobal {
                imm: -0x7FE4,
                vram: Vram::new(0x80000104),
            },
            InstrProcessedResult::GpGotLocal {
                imm: -0x7FE8,
                vram: Vram::new(0x80000000),
            },
            InstrProcessedResult::PairedGpGotLo {
                upper_imm: -0x7FE8,
                upper_rom: Rom::new(0x00010034),
                imm: 0xE8,
                vram: Vram::new(0x800000E8),
            },
            InstrProcessedResult::GpGotGlobal {
                imm: -0x7FE0,
                vram: Vram::new(0x80000094),
            },
            InstrProcessedResult::RawRegisterLink {
                jr_reg_data: JrRegData::new(Rom::new(0x0001003C), 0x80000094, None, None),
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstrProcessedResult::DanglingLo { imm: 0x18 },
            InstrProcessedResult::GpGotGlobal {
                imm: -0x7FE0,
                vram: Vram::new(0x80000094),
            },
            InstrProcessedResult::RawRegisterLink {
                jr_reg_data: JrRegData::new(Rom::new(0x0001004C), 0x80000094, None, None),
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstrProcessedResult::DanglingLo { imm: 0x18 },
            InstrProcessedResult::GpGotLocal {
                imm: -0x7FE8,
                vram: Vram::new(0x80000000),
            },
            InstrProcessedResult::PairedGpGotLo {
                upper_imm: -0x7FE8,
                upper_rom: Rom::new(0x0001005C),
                imm: 0x88,
                vram: Vram::new(0x80000088),
            },
            InstrProcessedResult::RawRegisterLink {
                jr_reg_data: JrRegData::new(Rom::new(0x00010060), 0x80000088, None, None),
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstrProcessedResult::DanglingLo { imm: 0x18 },
            InstrProcessedResult::GpGotGlobal {
                imm: -0x7FDC,
                vram: Vram::new(0x8000011C),
            },
            InstrProcessedResult::DanglingLo { imm: 0x4 },
            InstrProcessedResult::DereferencedRegisterLink {
                jr_reg_data: JrRegData::new(Rom::new(0x00010070), 0x8000011C, None, None),
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstrProcessedResult::DanglingLo { imm: 0x18 },
            InstrProcessedResult::DanglingLo { imm: 0x10 },
            InstrProcessedResult::DanglingLo { imm: 0x20 },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_jr,
            },
            InstrProcessedResult::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
        ];

        let rom = Rom::new(0x00010000);
        let vram = Vram::new(0x80000000);
        let endian = Endian::Big;
        let original_gp_config = GpConfig::new_pic(GpValue::new(0x800080D0));
        let current_gp_value = original_gp_config.gp_value();

        let got_locals = vec![
            /* -0x7FF0($gp) */ GotLocalEntry::new(0x0F000000),
            /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000),
            /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000),
        ];
        let got_globals = vec![
            /* -0x7FE4($gp) */ GotGlobalEntry::new(0x80000104, 0x80000104, false),
            /* -0x7FE0($gp) */ GotGlobalEntry::new(0x80000094, 0x80000094, false),
            /* -0x7FDC($gp) */ GotGlobalEntry::new(0x8000011C, 0x8000011C, false),
        ];
        let global_offset_table =
            GlobalOffsetTable::new(Vram::new(0x800000E0), got_locals, got_globals);

        let mut expected_results = EXPECTED_RESULTS.iter();

        let instructions: Vec<Instruction> = BYTES
            .chunks_exact(4)
            .enumerate()
            .map(|(instr_index, w)| {
                let i = instr_index * 4;
                let word = endian.word_from_bytes(w);
                let current_vram = vram + Size::new(i as u32);

                Instruction::new(
                    word,
                    current_vram,
                    InstructionFlags::new(IsaVersion::MIPS_III),
                )
            })
            .collect();

        let mut regs_tracker = RegisterTracker::new();
        let mut prev_instr = None;
        for (instr_index, instr) in instructions.into_iter().enumerate() {
            let i = instr_index * 4;
            let current_rom = rom + Size::new(i as u32);
            let instr_processed_result = regs_tracker.process_instruction(
                &instr,
                current_rom,
                Some(&global_offset_table),
                Some(&original_gp_config),
                Some(&current_gp_value),
            );

            #[cfg(feature = "std")]
            println!("{} {:?}", instr_index, instr_processed_result);

            assert_eq!(expected_results.next(), Some(&instr_processed_result));

            regs_tracker.overwrite_registers(&instr, current_rom);
            regs_tracker.clear_afterwards(prev_instr.as_ref());
            prev_instr = Some(instr);
        }
    }
}
