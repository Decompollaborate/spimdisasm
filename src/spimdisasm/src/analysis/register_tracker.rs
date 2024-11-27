/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{registers::Gpr, traits::Register, Instruction};

use super::TrackedRegisterState;

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
    pub fn clear(&mut self) {
        self.registers.iter_mut().for_each(|state| state.clear());
    }

    pub fn unset_registers_after_func_call(
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
}
