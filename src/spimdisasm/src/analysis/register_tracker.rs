/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{opcodes::Opcode, registers::Gpr, traits::Register, Instruction};

use crate::rom_address::RomAddress;

use super::{tracked_register_state::HiInfo, JrRegData, LoPairingInfo, TrackedRegisterState};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterTracker {
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
    pub(crate) fn clear(&mut self) {
        self.registers.iter_mut().for_each(|state| state.clear());
    }

    pub(crate) fn unset_registers_after_func_call(
        &mut self,
        instr: &Instruction,
        prev_instr: &Instruction,
    ) {
        if !prev_instr.is_function_call() {
            return;
        }

        // TODO: consider writing an register iterator or something
        for i in 0..Gpr::count() as u32 {
            let reg: Gpr = i.try_into().expect("This should not panic");

            if reg.is_clobbered_by_func_call(instr.abi()) {
                self.registers[reg.as_index()].clear();
            }
        }
    }

    pub(crate) fn process_branch(&mut self, instr: &Instruction, instr_rom: RomAddress) {
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
        instr
            .field_rs()
            .and_then(|reg| self.registers[reg.as_index()].get_jr_reg_data())
    }

    pub(crate) fn process_hi(
        &mut self,
        instr: &Instruction,
        instr_rom: RomAddress,
        prev_instr: Option<&Instruction>,
    ) {
        assert!(instr.opcode().can_be_hi());

        let reg = instr
            .get_destination_gpr()
            .expect("lui should have dst register");
        let state = &mut self.registers[reg.as_index()];

        state.clear();
        state.set_hi(
            instr
                .get_processed_immediate()
                .expect("lui should have an immediate field") as u32,
            instr_rom,
            prev_instr,
        );
    }

    pub(crate) fn process_gp_load(&mut self, instr: &Instruction, instr_rom: RomAddress) {
        assert!(instr.opcode().can_be_lo());

        let reg = instr
            .get_destination_gpr()
            .expect("should have dst register");
        let state = &mut self.registers[reg.as_index()];

        state.clear();
        state.set_gp_load(
            instr
                .get_processed_immediate()
                .expect("should have immediate field") as u32,
            instr_rom,
        );
    }

    pub(crate) fn get_hi_info_for_constant(&self, instr: &Instruction) -> Option<HiInfo> {
        if let Some(rs) = instr.field_rs() {
            self.registers[rs.as_index()].hi_info()
        } else {
            None
        }
    }

    pub(crate) fn process_constant(
        &mut self,
        instr: &Instruction,
        value: u32,
        instr_rom: RomAddress,
    ) {
        if let Some(dst_reg) = instr.get_destination_gpr() {
            let state = &mut self.registers[dst_reg.as_index()];

            state.set_lo(value, instr_rom);
        }
    }

    pub(crate) fn process_lo(&mut self, instr: &Instruction, value: u32, instr_rom: RomAddress) {
        if let Some(dst_reg) = instr.get_destination_gpr() {
            let state = &mut self.registers[dst_reg.as_index()];
            state.set_lo(value, instr_rom);
            if instr.opcode().does_dereference() {
                state.set_deref(instr_rom);
            }
            if Some(dst_reg) == instr.field_rs() {
                state.clear_hi();
                state.clear_gp();
            }
            state.clear_branch();
        }
    }

    pub(crate) fn get_address_if_instr_can_set_type(
        &self,
        instr: &Instruction,
        instr_rom: RomAddress,
    ) -> Option<u32> {
        if let Some(rs) = instr.field_rs() {
            let state = &self.registers[rs.as_index()];

            if state.lo_info().is_some() && state.dereferenced().is_none_or(|x| x == instr_rom) {
                Some(state.value())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn overwrite_registers(&mut self, instr: &Instruction, instr_rom: RomAddress) {
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

    pub(crate) fn preprocess_lo_and_get_info(
        &mut self,
        instr: &Instruction,
        instr_rom: RomAddress,
    ) -> Option<LoPairingInfo> {
        if let Some(reg) = instr.field_rs() {
            let state = &self.registers[reg.as_index()];

            if let Some(hi_info) = state.hi_info() {
                if !hi_info.set_on_branch_likely {
                    return Some(LoPairingInfo {
                        instr_rom: hi_info.instr_rom,
                        value: state.value() as i64,
                        is_gp_rel: false,
                        is_gp_got: false,
                    });
                }
            } else if reg.is_global_pointer(instr.abi()) {
                return Some(LoPairingInfo {
                    instr_rom: RomAddress::new(0),
                    value: state.value() as i64,
                    is_gp_rel: true,
                    is_gp_got: false,
                });
            } else if let Some(gp_info) = state.gp_info() {
                return Some(LoPairingInfo {
                    instr_rom: gp_info,
                    value: state.value() as i64,
                    is_gp_rel: false,
                    is_gp_got: true,
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

    pub(crate) fn has_lo_but_not_hi(&self, instr: &Instruction) -> bool {
        instr.field_rs().is_some_and(|reg| {
            let state = self.registers[reg.as_index()];
            state.lo_info().is_some() && state.hi_info().is_none()
        })
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

    fn clear_reg(&mut self, reg: Gpr, instr: &Instruction, instr_rom: RomAddress) {
        let state = &mut self.registers[reg.as_index()];

        state.clear_hi();
        if !state.was_set_in_current_instr(instr_rom) {
            state.clear_gp();
            state.clear_lo();
        }
        state.clear_branch();

        if instr.opcode().reads_fd() || instr.opcode().reads_ft() || instr.opcode().reads_fs() {
            state.set_contains_float();
        } else {
            state.clear_contains_float();
        }
    }
}
