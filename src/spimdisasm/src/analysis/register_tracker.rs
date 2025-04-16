/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{
    abi::Abi, access_type::AccessType, opcodes::Opcode, registers::Gpr, registers_meta::Register,
    vram::VramOffset, Instruction,
};

use crate::{
    addresses::{GlobalOffsetTable, Rom, Vram},
    analysis::gpr_register_value::{GprRegDereferencedAddress, GprRegRawAddress},
    config::{Endian, GpConfig},
};

use super::{gpr_register_value::GprRegConstantInfo, GprRegisterValue};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) struct RegisterTracker {
    registers: [GprRegisterValue; Gpr::count()],
    gp_config: Option<GpConfig>,
    endian: Endian,
}

impl RegisterTracker {
    pub(crate) fn new(
        abi: Abi,
        function_address: Option<Vram>,
        gp_config: Option<GpConfig>,
        endian: Endian,
    ) -> Self {
        let registers = [GprRegisterValue::Garbage; Gpr::count()];
        let mut slf = Self {
            registers,
            gp_config,
            endian,
        };

        slf.soft_reset(abi, function_address);
        slf
    }

    pub(crate) fn soft_reset(&mut self, abi: Abi, function_address: Option<Vram>) {
        for reg in Gpr::iter() {
            let reg_value = &mut self.registers[reg.as_index()];

            if !matches!(reg_value, GprRegisterValue::StackPointer { .. }) {
                *reg_value = GprRegisterValue::new(reg, abi, function_address, self.gp_config);
            }
        }
    }

    // For debugging
    #[allow(dead_code)]
    #[doc(hidden)]
    pub(crate) fn get(&self, gpr: Gpr) -> &GprRegisterValue {
        &self.registers[gpr.as_index()]
    }

    fn set_gpr_value(&mut self, gpr: Gpr, value: GprRegisterValue) {
        let old_value = &mut self.registers[gpr.as_index()];

        match old_value {
            GprRegisterValue::HardwiredZero => {}
            _ => *old_value = value,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstructionOperation {
    Link {
        info: InstrOpLink,
    },

    TailCall {
        info: InstrOpTailCall,
    },

    /// Jump into a `case` of a `switch`. A `jr`.
    JumptableJump {
        jumptable_vram: Vram,
        dereferenced_rom: Rom,
        info: InstrOpJumptable,
    },

    ReturnJump,

    /// An usual non-linking branch.
    ///
    /// This may include the `j` instruction depending on the rabbitizer `Instruction`'s flags,
    /// specifically `j_as_branch` being `true`.
    Branch {
        target_vram: Vram,
    },

    /// This instruction can set the `%hi` part of the reloc to a symbol. A `lui`.
    Hi {
        value: u32,
        dst_reg: Gpr,
    },

    PairedAddress {
        vram: Vram,
        info: InstrOpPairedAddress,
    },

    DereferencedRawAddress {
        original_address: Vram,
        addend: i16,
        address_load_rom: Rom,
        access_info: (AccessType, bool),
    },

    GpSet {
        hi_rom: Rom,
    },

    /// A "lo" kind of instruction that couldn't be paired to anything.
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

    RegisterOperation {
        info: InstrOpRegisterOperation,
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrOpLink {
    /// "Branch and link" kind of instructions.
    ///
    /// These kind is usually used in handwritten assembly, so we can't expect ABI conventions to
    /// hold true, so this can branch into either a "real" function or into the middle of a
    /// function, even branching into the somewhere inside the current function!
    LinkingBranch { target_vram: Vram },

    /// A "normal" function call to a hardcoded address. A `jal`.
    ///
    /// This is the "normal" way to call functions on statically linked code, position independent
    /// code (PIC) won't be seeing much of this.
    DirectLinkingCall { target_vram: Vram },

    /// A "Jump and link register" to a register that contains a raw address. A `jalr`.
    ///
    /// Here we know the actual address of the function that is being called.
    RawRegisterLink { vram: Vram, rom: Rom },

    /// A "Jump and link register" to a register that contains a raw address from the GOT. A `jalr`.
    ///
    /// This is the result of calling an address loaded with the `%call16` reloc. See `GpGotGlobal`.
    Call16RegisterLink { vram: Vram, rom: Rom },

    /// A "Jump and link register" to a register that contains a raw address from the GOT. A `jalr`.
    ///
    /// This is the result of calling an address loaded with `%call_hi`/`%call_lo`. See `PairedGotLo`.
    CallHiLoRegisterLink { vram: Vram, rom: Rom },

    /// A "Jump and link register" to a register that has been dereferenced. A `jalr`.
    ///
    /// This usually happens on arrays of function pointers, meaning we only know the address of
    /// the array but not the address of the actual function that is being called.
    DereferencedRegisterLink {
        dereferenced_vram: Vram,
        dereferenced_rom: Rom,
    },

    /// A "Jump and link register", but we don't have info about what is being called. A `jalr`.
    UnknownJumpAndLinkRegister { reg: Gpr },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrOpTailCall {
    /// A suspected tail call to a hardcoded address. A `j`.
    ///
    /// This being an actual tail call is not certain since some compilers (or handwritten
    /// assembly) may use this instruction as an unconditional branch, a tail call or even both.
    MaybeDirectTailCall {
        target_vram: Vram,
    },

    RawRegisterTailCall {
        vram: Vram,
        rom: Rom,
    },

    // TODO: maybe make a variant for dereferenced with addends?
    DereferencedRegisterTailCall {
        dereferenced_vram: Vram,
        dereferenced_rom: Rom,
    },

    /// Jump to a register, but we don't have info about what it is pointing to. A `jr`.
    UnknownRegisterJump {
        reg: Gpr,
    },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrOpJumptable {
    Simple,
    Pic,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrOpPairedAddress {
    /// This instruction was paired to the `%hi` part of a reloc as a `%lo`.
    PairedLo {
        hi_rom: Rom,
        access_info: Option<(AccessType, bool)>,
    },

    /// An address relative to the global pointer.
    GpRel {
        access_info: Option<(AccessType, bool)>,
    },

    /// A pointer to a symbol contained within the global part of the Global Offset Table (GOT).
    GpGotGlobal {},

    /// A pointer to the Lazy Resolver.
    GpGotLazyResolver {},

    /// A pointer to a symbol contained within the local part of the Global Offset Table (GOT).
    ///
    /// This kind usually needs to be paired to create the actual real symbol's address.
    GpGotLocal {},

    /// A paired GOT local pointer. This pointer has been paired to a `GpGotLocal`.
    PairedGpGotLo {
        upper_rom: Rom,
        access_info: Option<(AccessType, bool)>,
    },

    /// A paired `%got_lo` pointer relocation.
    ///
    /// This usually follows a pattern like the following:
    /// ```mips
    /// lui         $reg, %got_hi(SYMBOL)
    /// addu        $reg, $reg, $gp
    /// lw          $reg2, %got_lo(SYMBOL)($reg)
    /// ```
    /// Note `$reg` and `$reg2` may or may not be the same register.
    ///
    /// Also spimdisasm may catch up this pattern even if `$gp` is not involved.
    /// This will only happen if it detects the used register has the "global pointer" value.
    ///
    /// Note this will also catch `%call_hi`/`%call_lo` pairings since they follow the same patter.
    PairedGotLo { hi_rom: Rom },

    PairedLoUnaligned {
        hi_rom: Rom,
        access_info: (AccessType, bool),
        unaddended_address: Vram,
    },

    GpRelUnaligned {
        access_info: (AccessType, bool),
        unaddended_address: Vram,
    },
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub(crate) enum InstrOpRegisterOperation {
    SuspectedCpload { hi_rom: Rom, lo_rom: Rom },

    RegisterAddition { rd: Gpr, rs: Gpr, rt: Gpr },

    RegisterSubtraction { rd: Gpr, rs: Gpr, rt: Gpr },

    Or { rd: Gpr, rs: Gpr, rt: Gpr },
}

impl RegisterTracker {
    pub(crate) fn process_instruction(
        &mut self,
        instr: &Instruction,
        instr_rom: Rom,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> InstructionOperation {
        if !instr.is_valid() {
            return InstructionOperation::InvalidInstr {};
        }

        let opcode = instr.opcode();
        if opcode.does_link() {
            if let Some(target_vram) = instr.get_instr_index_as_vram() {
                InstructionOperation::Link {
                    info: InstrOpLink::DirectLinkingCall { target_vram },
                }
            } else if let Some(target_vram) = instr.get_branch_vram_generic() {
                InstructionOperation::Link {
                    info: InstrOpLink::LinkingBranch { target_vram },
                }
            } else {
                self.handle_jalr(instr)
            }
        } else if let Some(target_vram) = instr.get_branch_vram_generic() {
            self.handle_branch(instr, target_vram)
        } else if opcode.jumps_to_register() {
            // At this point only `jr` should catched here, `jalr` should be catched by the "does_link" check.
            self.handle_jr(instr)
        } else if let Some(target_vram) = instr.get_instr_index_as_vram() {
            // At this point only `j` should have an instr_index field.
            debug_assert!(opcode == Opcode::core_j);

            // self.process_branch(instr, instr_rom);

            // Some compilers use `j` as an unconditional branch, as a tail call or even as both.
            // So it is hard to say if this a tail call or not, thus `Maybe`.
            InstructionOperation::TailCall {
                info: InstrOpTailCall::MaybeDirectTailCall { target_vram },
            }
        } else if opcode.can_be_hi() {
            let (reg, reg_value, info) = self.handle_hi(instr, instr_rom);
            self.set_gpr_value(reg, reg_value);

            info
        } else if opcode.can_be_lo() {
            let (new_val, info) = self.handle_lo(instr, instr_rom, global_offset_table);
            if let Some((reg, reg_value)) = new_val {
                self.set_gpr_value(reg, reg_value);
            }

            info
        } else if opcode.can_be_unsigned_lo() {
            let (reg, reg_value, info) = self.handle_unsigned_lo(instr, instr_rom);
            self.set_gpr_value(reg, reg_value);

            info
        } else if opcode.adds_registers() {
            let (reg, reg_value, info) = self.handle_add_registers(instr, instr_rom);
            self.set_gpr_value(reg, reg_value);

            info
        } else if opcode.subs_registers() {
            let (reg, reg_value, info) = self.handle_sub_registers(instr, instr_rom);
            self.set_gpr_value(reg, reg_value);

            info
        } else if opcode.ors_registers() {
            let (reg, reg_value, info) = self.handle_or_registers(instr, instr_rom);
            self.set_gpr_value(reg, reg_value);

            info
        } else if opcode.ands_registers() {
            if let (Some(rd), Some(rs), Some(rt)) =
                (instr.field_rd(), instr.field_rs(), instr.field_rt())
            {
                if rd.is_stack_pointer(instr.abi())
                    && (rs.is_stack_pointer(instr.abi()) || rt.is_stack_pointer(instr.abi()))
                {
                    // Some programs (IDO 7.1 programs to be precise) like to `and` the
                    // stack pointer as a way to align down the stack.
                    // I didn't want to actually implement logic for this silliness, so
                    // here we have a hardcoded check.
                } else {
                    self.set_gpr_value(rd, GprRegisterValue::Garbage);
                }
            }

            InstructionOperation::UnhandledOpcode { opcode }
        } else {
            if let Some(reg) = instr.get_destination_gpr() {
                self.set_gpr_value(reg, GprRegisterValue::Garbage);
            }
            InstructionOperation::UnhandledOpcode { opcode }
        }
    }

    fn handle_jalr(&self, instr: &Instruction) -> InstructionOperation {
        debug_assert!(instr.opcode() == Opcode::core_jalr);

        let rs = instr.field_rs().expect("jalr should have an rs field");
        let reg_value = &self.registers[rs.as_index()];

        let info = match reg_value {
            GprRegisterValue::RawAddress {
                vram,
                setter_rom,
                info,
            } => {
                match info {
                    GprRegRawAddress::HiLo { .. }
                    | GprRegRawAddress::GpRel {}
                    | GprRegRawAddress::PairedGpGotLo { .. } => InstrOpLink::RawRegisterLink {
                        vram: *vram,
                        rom: *setter_rom,
                    },
                    GprRegRawAddress::GpGotGlobal { .. } => InstrOpLink::Call16RegisterLink {
                        vram: *vram,
                        rom: *setter_rom,
                    },
                    GprRegRawAddress::GpGotLocal { .. } => {
                        // TODO: handle GpGotLocal differently
                        InstrOpLink::Call16RegisterLink {
                            vram: *vram,
                            rom: *setter_rom,
                        }
                    }
                    GprRegRawAddress::HiLoGp { .. } => InstrOpLink::CallHiLoRegisterLink {
                        vram: *vram,
                        rom: *setter_rom,
                    },
                    GprRegRawAddress::GpGotLazyResolver { .. } => {
                        InstrOpLink::UnknownJumpAndLinkRegister { reg: rs }
                    }
                }
            }
            GprRegisterValue::DereferencedAddress {
                original_address,
                deref_rom,
                ..
            }
            | GprRegisterValue::DereferencedAddressBranchChecked {
                original_address,
                deref_rom,
                ..
            } => InstrOpLink::DereferencedRegisterLink {
                dereferenced_vram: *original_address,
                dereferenced_rom: *deref_rom,
            },

            GprRegisterValue::Garbage
            | GprRegisterValue::HardwiredZero
            | GprRegisterValue::SoftZero
            | GprRegisterValue::GlobalPointer { .. }
            | GprRegisterValue::StackPointer { .. }
            | GprRegisterValue::GivenAddress { .. }
            | GprRegisterValue::Hi { .. }
            | GprRegisterValue::HiGp { .. }
            | GprRegisterValue::ConstantInfo { .. }
            | GprRegisterValue::DereferencedAddressAddedWithGp { .. } => {
                InstrOpLink::UnknownJumpAndLinkRegister { reg: rs }
            }
        };

        InstructionOperation::Link { info }
    }

    fn handle_branch(&mut self, instr: &Instruction, target_vram: Vram) -> InstructionOperation {
        let opcode = instr.opcode();

        if let (true, Some(reg)) = (opcode.reads_rs(), instr.field_rs()) {
            self.registers[reg.as_index()].apply_branch();
        }
        if let (true, Some(reg)) = (opcode.reads_rt(), instr.field_rt()) {
            self.registers[reg.as_index()].apply_branch();
        }
        if let (true, Some(reg)) = (opcode.reads_rd(), instr.field_rd()) {
            self.registers[reg.as_index()].apply_branch();
        }

        InstructionOperation::Branch { target_vram }
    }

    fn handle_jr(&self, instr: &Instruction) -> InstructionOperation {
        debug_assert!(instr.opcode() == Opcode::core_jr);

        let rs = instr.field_rs().expect("jr should have an rs field");
        let reg_value = &self.registers[rs.as_index()];

        match reg_value {
            GprRegisterValue::DereferencedAddress {
                original_address,
                deref_rom,
                info,
                ..
            } => match info {
                GprRegDereferencedAddress::Hi { .. } => {
                    if rs.holds_return_address(instr.abi()) {
                        InstructionOperation::ReturnJump
                    } else {
                        InstructionOperation::JumptableJump {
                            jumptable_vram: *original_address,
                            dereferenced_rom: *deref_rom,
                            info: InstrOpJumptable::Simple,
                        }
                    }
                }
                GprRegDereferencedAddress::HiLo { addend, .. } => {
                    if rs.holds_return_address(instr.abi()) {
                        InstructionOperation::ReturnJump
                    } else if *addend != 0 {
                        InstructionOperation::TailCall {
                            info: InstrOpTailCall::DereferencedRegisterTailCall {
                                dereferenced_vram: *original_address,
                                dereferenced_rom: *deref_rom,
                            },
                        }
                    } else {
                        InstructionOperation::JumptableJump {
                            jumptable_vram: *original_address,
                            dereferenced_rom: *deref_rom,
                            info: InstrOpJumptable::Pic,
                        }
                    }
                }
                GprRegDereferencedAddress::GpRel { .. }
                | GprRegDereferencedAddress::RawGpRel { .. }
                | GprRegDereferencedAddress::GpGotGlobal { .. }
                | GprRegDereferencedAddress::GpGotLocal { .. }
                | GprRegDereferencedAddress::PairedGpGotLo { .. }
                | GprRegDereferencedAddress::HiLoGp { .. } => {
                    if rs.holds_return_address(instr.abi()) {
                        // TODO: can this even happen?
                        InstructionOperation::ReturnJump
                    } else {
                        InstructionOperation::TailCall {
                            info: InstrOpTailCall::DereferencedRegisterTailCall {
                                dereferenced_vram: *original_address,
                                dereferenced_rom: *deref_rom,
                            },
                        }
                    }
                }
                GprRegDereferencedAddress::HiUnaligned { .. }
                | GprRegDereferencedAddress::GpRelUnaligned { .. } => {
                    InstructionOperation::TailCall {
                        info: InstrOpTailCall::UnknownRegisterJump { reg: rs },
                    }
                }
            },

            GprRegisterValue::DereferencedAddressBranchChecked {
                original_address,
                deref_rom,
                ..
            } => {
                if rs.holds_return_address(instr.abi()) {
                    // TODO: can this even happen?
                    InstructionOperation::ReturnJump
                } else {
                    InstructionOperation::TailCall {
                        info: InstrOpTailCall::DereferencedRegisterTailCall {
                            dereferenced_vram: *original_address,
                            dereferenced_rom: *deref_rom,
                        },
                    }
                }
            }
            GprRegisterValue::DereferencedAddressAddedWithGp {
                original_address,
                deref_rom,
                ..
            } => InstructionOperation::JumptableJump {
                jumptable_vram: *original_address,
                dereferenced_rom: *deref_rom,
                info: InstrOpJumptable::Pic,
            },
            GprRegisterValue::RawAddress {
                vram, setter_rom, ..
            } => InstructionOperation::TailCall {
                info: InstrOpTailCall::RawRegisterTailCall {
                    vram: *vram,
                    rom: *setter_rom,
                },
            },

            GprRegisterValue::Garbage
            | GprRegisterValue::HardwiredZero
            | GprRegisterValue::SoftZero
            | GprRegisterValue::GlobalPointer { .. }
            | GprRegisterValue::StackPointer { .. }
            | GprRegisterValue::GivenAddress { .. }
            | GprRegisterValue::Hi { .. }
            | GprRegisterValue::HiGp { .. }
            | GprRegisterValue::ConstantInfo { .. } => {
                if rs.holds_return_address(instr.abi()) {
                    // TODO: can this even happen?
                    InstructionOperation::ReturnJump
                } else {
                    InstructionOperation::TailCall {
                        info: InstrOpTailCall::UnknownRegisterJump { reg: rs },
                    }
                }
            }
        }
    }

    fn handle_hi(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> (Gpr, GprRegisterValue, InstructionOperation) {
        debug_assert!(instr.opcode().can_be_hi());

        let reg = instr.field_rt().expect("lui should have an rt field");
        let imm = instr
            .get_processed_immediate()
            .expect("lui should have an immediate field") as u32;
        let value = imm << 16;

        let reg_value = GprRegisterValue::Hi {
            rom: instr_rom,
            value,
        };
        let info = InstructionOperation::Hi {
            dst_reg: reg,
            value,
        };
        (reg, reg_value, info)
    }

    fn handle_lo(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) -> (Option<(Gpr, GprRegisterValue)>, InstructionOperation) {
        let opcode = instr.opcode();
        let imm = instr
            .get_processed_immediate()
            .expect("This instruction should have an immediate") as i16;
        let rs = instr
            .field_rs()
            .expect("lo instructions should have an rs field");

        if opcode.does_dereference() {
            self.handle_lo_dereference(instr, instr_rom, global_offset_table, rs, imm)
        } else {
            let rt = instr.field_rt().expect("should have an rt field");
            self.handle_lo_addiu(instr, instr_rom, rt, rs, imm)
        }
    }

    fn handle_lo_dereference(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
        global_offset_table: Option<&GlobalOffsetTable>,
        rs: Gpr,
        imm: i16,
    ) -> (Option<(Gpr, GprRegisterValue)>, InstructionOperation) {
        let opcode = instr.opcode();
        let src_reg_value = &self.registers[rs.as_index()];

        let access_type = opcode
            .access_type()
            .expect("An instruction that dereferences the memory must have an access type");
        let does_unsigned_memory_access = opcode.does_unsigned_memory_access();
        let access_info = (access_type, does_unsigned_memory_access);
        let reg_value = src_reg_value.dereference(
            imm,
            instr_rom,
            access_info,
            global_offset_table,
            self.endian,
        );

        let info = match &reg_value {
            GprRegisterValue::DereferencedAddress {
                original_address,
                access_info,
                info,
                ..
            }
            | GprRegisterValue::DereferencedAddressBranchChecked {
                original_address,
                access_info,
                info,
                ..
            } => match info {
                GprRegDereferencedAddress::Hi { hi_rom } => InstructionOperation::PairedAddress {
                    vram: *original_address,
                    info: InstrOpPairedAddress::PairedLo {
                        hi_rom: *hi_rom,
                        access_info: Some(*access_info),
                    },
                },
                GprRegDereferencedAddress::GpRel {} => InstructionOperation::PairedAddress {
                    vram: *original_address,
                    info: InstrOpPairedAddress::GpRel {
                        access_info: Some(*access_info),
                    },
                },
                GprRegDereferencedAddress::HiLo {
                    lo_rom: address_load_rom,
                    addend,
                }
                | GprRegDereferencedAddress::RawGpRel {
                    addend,
                    lo_rom: address_load_rom,
                }
                | GprRegDereferencedAddress::GpGotGlobal {
                    addend,
                    upper_rom: address_load_rom,
                }
                | GprRegDereferencedAddress::HiLoGp {
                    addend,
                    lo_rom: address_load_rom,
                    ..
                } => InstructionOperation::DereferencedRawAddress {
                    original_address: *original_address,
                    addend: *addend,
                    address_load_rom: *address_load_rom,
                    access_info: *access_info,
                },
                GprRegDereferencedAddress::GpGotLocal { upper_rom } => {
                    InstructionOperation::PairedAddress {
                        vram: *original_address,
                        info: InstrOpPairedAddress::PairedGpGotLo {
                            upper_rom: *upper_rom,
                            access_info: Some(*access_info),
                        },
                    }
                }
                GprRegDereferencedAddress::PairedGpGotLo {
                    addend,
                    lo_rom: address_load_rom,
                    ..
                } => InstructionOperation::DereferencedRawAddress {
                    original_address: *original_address,
                    addend: *addend,
                    address_load_rom: *address_load_rom,
                    access_info: *access_info,
                },
                GprRegDereferencedAddress::HiUnaligned {
                    hi_rom,
                    unaddended_address,
                } => InstructionOperation::PairedAddress {
                    vram: *original_address,
                    info: InstrOpPairedAddress::PairedLoUnaligned {
                        hi_rom: *hi_rom,
                        access_info: *access_info,
                        unaddended_address: *unaddended_address,
                    },
                },
                GprRegDereferencedAddress::GpRelUnaligned { unaddended_address } => {
                    InstructionOperation::PairedAddress {
                        vram: *original_address,
                        info: InstrOpPairedAddress::GpRelUnaligned {
                            access_info: *access_info,
                            unaddended_address: *unaddended_address,
                        },
                    }
                }
            },

            GprRegisterValue::RawAddress { vram, info, .. } => match info {
                GprRegRawAddress::GpGotGlobal {} => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::GpGotGlobal {},
                },
                GprRegRawAddress::GpGotLazyResolver {} => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::GpGotLazyResolver {},
                },
                GprRegRawAddress::GpGotLocal {} => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::GpGotLocal {},
                },
                GprRegRawAddress::HiLoGp { hi_rom } => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::PairedGotLo { hi_rom: *hi_rom },
                },
                GprRegRawAddress::HiLo { .. }
                | GprRegRawAddress::GpRel { .. }
                | GprRegRawAddress::PairedGpGotLo { .. } => {
                    InstructionOperation::DanglingLo { imm }
                }
            },

            GprRegisterValue::Garbage
            | GprRegisterValue::HardwiredZero
            | GprRegisterValue::SoftZero
            | GprRegisterValue::GlobalPointer { .. }
            | GprRegisterValue::StackPointer { .. }
            | GprRegisterValue::GivenAddress { .. }
            | GprRegisterValue::Hi { .. }
            | GprRegisterValue::HiGp { .. }
            | GprRegisterValue::ConstantInfo { .. }
            | GprRegisterValue::DereferencedAddressAddedWithGp { .. } => {
                InstructionOperation::DanglingLo { imm }
            }
        };

        let new_val = if let (true, Some(rt)) = (opcode.does_load(), instr.field_rt()) {
            // Hack to avoid ovewriting the $gp value when the asm is restoring it from the stack.
            if matches!(
                self.registers[rt.as_index()],
                GprRegisterValue::GlobalPointer { .. }
            ) && matches!(src_reg_value, GprRegisterValue::StackPointer { .. })
            {
                None
            } else {
                Some((rt, reg_value))
            }
        } else {
            None
        };
        (new_val, info)
    }

    fn handle_lo_addiu(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
        rt: Gpr,
        rs: Gpr,
        imm: i16,
    ) -> (Option<(Gpr, GprRegisterValue)>, InstructionOperation) {
        // Technically this covers stuff like daddiu, daddi, addi, etc.
        // But I don't remember seeing code that actually uses those instructions for address pairing.
        // I guess its better safe than sorry.

        debug_assert!(
            instr.opcode().modifies_rt(),
            "{:?} {:?}",
            instr_rom,
            instr.opcode()
        );

        let src_reg_value = &self.registers[rs.as_index()];
        let reg_value = src_reg_value.add_imm16(imm, instr_rom, self.gp_config.as_ref(), rt);

        let info = match &reg_value {
            GprRegisterValue::RawAddress { vram, info, .. } => match info {
                GprRegRawAddress::HiLo { hi_rom } => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::PairedLo {
                        hi_rom: *hi_rom,
                        access_info: None,
                    },
                },
                GprRegRawAddress::GpRel {} => InstructionOperation::PairedAddress {
                    vram: *vram,
                    info: InstrOpPairedAddress::GpRel { access_info: None },
                },
                GprRegRawAddress::PairedGpGotLo { upper_rom, .. } => {
                    InstructionOperation::PairedAddress {
                        vram: *vram,
                        info: InstrOpPairedAddress::PairedGpGotLo {
                            upper_rom: *upper_rom,
                            access_info: None,
                        },
                    }
                }
                GprRegRawAddress::GpGotGlobal { .. }
                | GprRegRawAddress::GpGotLazyResolver { .. }
                | GprRegRawAddress::GpGotLocal { .. }
                | GprRegRawAddress::HiLoGp { .. } => InstructionOperation::DanglingLo { imm },
            },

            GprRegisterValue::GlobalPointer { hi_rom, .. } => InstructionOperation::GpSet {
                hi_rom: hi_rom.expect("Should have set the hi_rom here"),
            },

            GprRegisterValue::Garbage
            | GprRegisterValue::HardwiredZero
            | GprRegisterValue::SoftZero
            | GprRegisterValue::StackPointer { .. }
            | GprRegisterValue::GivenAddress { .. }
            | GprRegisterValue::Hi { .. }
            | GprRegisterValue::HiGp { .. }
            | GprRegisterValue::DereferencedAddress { .. }
            | GprRegisterValue::DereferencedAddressBranchChecked { .. }
            | GprRegisterValue::DereferencedAddressAddedWithGp { .. }
            | GprRegisterValue::ConstantInfo { .. } => InstructionOperation::DanglingLo { imm },
        };

        (Some((rt, reg_value)), info)
    }

    fn handle_unsigned_lo(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> (Gpr, GprRegisterValue, InstructionOperation) {
        let rt = instr.field_rt().expect("should have an rt field");
        let rs = instr.field_rs().expect("should have an rs field");
        let imm = instr
            .get_processed_immediate()
            .expect("This instruction should have an immediate") as u16;

        let reg_value = self.registers[rs.as_index()].or_imm16(imm, instr_rom);

        let info = if let GprRegisterValue::ConstantInfo {
            info: GprRegConstantInfo::Constant { value, hi_rom, .. },
            ..
        } = &reg_value
        {
            InstructionOperation::Constant {
                constant: *value,
                hi_rom: *hi_rom,
            }
        } else {
            InstructionOperation::UnpairedConstant { imm }
        };

        (rt, reg_value, info)
    }

    fn handle_add_registers(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> (Gpr, GprRegisterValue, InstructionOperation) {
        let rd = instr
            .field_rd()
            .expect("This instruction should have a rd register");
        let rs = instr
            .field_rs()
            .expect("This instruction should have a rs register");
        let rt = instr
            .field_rt()
            .expect("This instruction should have a rt register");

        let rs_value = &self.registers[rs.as_index()];
        let rt_value = &self.registers[rt.as_index()];

        let reg_value = rs_value.add_register(rt_value, instr_rom, self.gp_config.as_ref());

        let mut info = InstructionOperation::RegisterOperation {
            info: InstrOpRegisterOperation::RegisterAddition { rd, rs, rt },
        };
        // Special check for `.cpload`
        if let GprRegisterValue::GlobalPointer { gp, .. } = &reg_value {
            if self.gp_config.map(|x| x.gp_value()) == Some(*gp) {
                match (rs_value, rt_value) {
                    (
                        GprRegisterValue::GivenAddress { .. },
                        GprRegisterValue::RawAddress {
                            setter_rom,
                            info: GprRegRawAddress::HiLo { hi_rom, .. },
                            ..
                        },
                    ) => {
                        info = InstructionOperation::RegisterOperation {
                            info: InstrOpRegisterOperation::SuspectedCpload {
                                hi_rom: *hi_rom,
                                lo_rom: *setter_rom,
                            },
                        };
                    }
                    (
                        GprRegisterValue::RawAddress {
                            setter_rom,
                            info: GprRegRawAddress::HiLo { hi_rom, .. },
                            ..
                        },
                        GprRegisterValue::GivenAddress { .. },
                    ) => {
                        info = InstructionOperation::RegisterOperation {
                            info: InstrOpRegisterOperation::SuspectedCpload {
                                hi_rom: *hi_rom,
                                lo_rom: *setter_rom,
                            },
                        };
                    }
                    (_, _) => {}
                }
            }
        }

        (rd, reg_value, info)
    }

    fn handle_sub_registers(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> (Gpr, GprRegisterValue, InstructionOperation) {
        let rd = instr
            .field_rd()
            .expect("This instruction should have a rd register");
        let rs = instr
            .field_rs()
            .expect("This instruction should have a rs register");
        let rt = instr
            .field_rt()
            .expect("This instruction should have a rt register");

        let rs_value = &self.registers[rs.as_index()];
        let rt_value = &self.registers[rt.as_index()];

        let reg_value = rs_value.sub_register(rt_value, instr_rom);

        let info = InstructionOperation::RegisterOperation {
            info: InstrOpRegisterOperation::RegisterSubtraction { rd, rs, rt },
        };

        (rd, reg_value, info)
    }

    fn handle_or_registers(
        &self,
        instr: &Instruction,
        instr_rom: Rom,
    ) -> (Gpr, GprRegisterValue, InstructionOperation) {
        let rd = instr
            .field_rd()
            .expect("This instruction should have a rd register");
        let rs = instr
            .field_rs()
            .expect("This instruction should have a rs register");
        let rt = instr
            .field_rt()
            .expect("This instruction should have a rt register");

        let reg_value =
            self.registers[rs.as_index()].or_register(&self.registers[rt.as_index()], instr_rom);

        let info = InstructionOperation::RegisterOperation {
            info: InstrOpRegisterOperation::Or { rd, rs, rt },
        };

        (rd, reg_value, info)
    }
}

impl RegisterTracker {
    // TODO: rename to a less silly name
    pub(crate) fn clear_afterwards(
        &mut self,
        prev_instr: Option<&Instruction>,
        new_function_address: Option<Vram>,
    ) -> bool {
        if let Some(prev) = prev_instr {
            if prev.is_function_call() {
                self.unset_registers_after_func_call(prev);
                let opcode = prev.opcode();
                if opcode.does_link() {
                    // This block of code exists only because of the rare cases where a function
                    // "abuses" the linking instructions as a way to get the current program
                    // counter into the `$ra` register.
                    //
                    // This is usually used as a way to calculate the gp value for the `$gp`
                    // register in position independent programs. The pattern is usually like this:
                    //
                    // ```mips
                    // bal         label
                    //  nop
                    // label:
                    // lui         $gp, %hi(_gp_disp)
                    // addiu       $gp, $gp, %lo(_gp_disp)
                    // addu        $gp, $gp, $ra
                    // ```
                    // Even if the currently known patterns only use `bal`, it was decided to make
                    // this more general and make it work for every link instruction.

                    let reg = if let (true, Some(rd)) = (opcode.modifies_rd(), prev.field_rd()) {
                        rd
                    } else {
                        Gpr::ra
                    };
                    let return_address = prev.vram() + VramOffset::new(0x8);
                    let value = GprRegisterValue::GivenAddress {
                        vram: return_address,
                    };
                    self.set_gpr_value(reg, value);
                }
            } else if (prev.opcode().is_jump() && !prev.opcode().does_link())
                || prev.is_unconditional_branch()
            {
                // TODO: handle function ends because of exceptions

                self.soft_reset(prev.abi(), new_function_address);
                return true;
            }
        }
        false
    }

    fn unset_registers_after_func_call(&mut self, prev_instr: &Instruction) {
        if !prev_instr.is_function_call() {
            return;
        }

        for reg in Gpr::iter() {
            if reg.is_clobbered_by_func_call(prev_instr.abi()) {
                self.set_gpr_value(reg, GprRegisterValue::Garbage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use rabbitizer::{InstructionFlags, IsaVersion};

    use crate::{
        addresses::{GotGlobalEntry, GotLocalEntry, GpValue, Size},
        config::Endian,
    };

    fn register_tracking_general_test(
        bytes: &[u8],
        expected_gpr_values: &[Option<GprRegisterValue>],
        expected_operations: &[InstructionOperation],
        gp_config: Option<GpConfig>,
        global_offset_table: Option<&GlobalOffsetTable>,
    ) {
        assert_eq!(bytes.len(), expected_gpr_values.len() * 4);
        assert_eq!(bytes.len(), expected_operations.len() * 4);
        if global_offset_table.is_some() {
            assert!(gp_config.is_some_and(|x| x.pic()));
        }

        let debug = false;

        let rom = Rom::new(0x00010000);
        let vram = Vram::new(0x80000000);
        let endian = Endian::Big;
        let abi = Abi::O32;

        let mut expected_gpr_values_iter = expected_gpr_values.iter();
        let mut expected_operations_iter = expected_operations.iter();

        let instructions: Vec<Instruction> = bytes
            .chunks_exact(4)
            .enumerate()
            .map(|(instr_index, w)| {
                let i = instr_index * 4;
                let word = endian.word_from_bytes(w);
                let current_vram = vram + Size::new(i as u32);

                Instruction::new(
                    word,
                    current_vram,
                    InstructionFlags::new(IsaVersion::MIPS_III)
                        .with_abi(abi)
                        .with_j_as_branch(false),
                )
            })
            .collect();

        let mut regs_tracker = RegisterTracker::new(abi, Some(vram), gp_config, endian);
        let mut prev_instr = None;
        for (instr_index, instr) in instructions.into_iter().enumerate() {
            let opcode = instr.opcode();
            let i = instr_index * 4;
            let current_rom = rom + Size::new(i as u32);

            #[cfg(feature = "std")]
            {
                use rabbitizer::InstructionDisplayFlags;

                let display_flags = InstructionDisplayFlags::new();
                let imm_override: Option<&str> = None;
                let instr_display = instr.display(&display_flags, imm_override, 0);
                println!("{} {:?} `{}`", instr_index, current_rom, instr_display);
            }

            let instr_processed_result =
                regs_tracker.process_instruction(&instr, current_rom, global_offset_table);

            let gpr_value =
                if let (false, Some(reg)) = (opcode.does_link(), instr.get_destination_gpr()) {
                    Some(regs_tracker.get(reg)).copied()
                } else {
                    None
                };

            #[cfg(feature = "std")]
            {
                println!("    {:?}", gpr_value);
                println!("    {:?}", instr_processed_result);
            }

            if !debug {
                assert_eq!(expected_gpr_values_iter.next(), Some(&gpr_value));
                assert_eq!(
                    expected_operations_iter.next(),
                    Some(&instr_processed_result)
                );
            }

            regs_tracker.clear_afterwards(prev_instr.as_ref(), None);
            prev_instr = Some(instr);
        }

        if debug {
            panic!();
        }
    }

    #[test]
    fn register_tracking_pairing_test_01() {
        static BYTES: [u8; 5 * 4] = [
            0x00, 0x04, 0x70, 0x80, // sll
            0x3C, 0x02, 0x80, 0x00, // lui
            0x00, 0x4E, 0x10, 0x21, // addu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x8C, 0x42, 0x00, 0x90, // lw
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 5] = [
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010004),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010004),
            }),
            None,
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000090),
                deref_rom: Rom::new(0x00010010),
                access_info: (AccessType::WORD, false),
                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010004),
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 5] = [
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_sll,
            },
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::v0,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::v0,
                    rt: Gpr::t6,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000090),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010004),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_02() {
        static BYTES: [u8; 4 * 4] = [
            0x3C, 0x02, 0x80, 0x00, // lui
            0x00, 0x44, 0x10, 0x21, // addu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x90, 0x42, 0x00, 0xC0, // lbu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            None,
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800000C0),
                deref_rom: Rom::new(0x0001000C),
                access_info: (AccessType::BYTE, true),
                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::v0,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::v0,
                    rt: Gpr::a0,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000C0),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: Some((AccessType::BYTE, true)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_03() {
        static BYTES: [u8; 3 * 4] = [
            0x27, 0x8E, 0x80, 0x10, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x00, 0x8E, 0x10, 0x21, // addu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 3] = [
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000B0),
                setter_rom: Rom::new(0x00010000),
                info: GprRegRawAddress::GpRel {},
            }),
            None,
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000B0),
                setter_rom: Rom::new(0x00010000),
                info: GprRegRawAddress::GpRel {},
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 3] = [
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000B0),
                info: InstrOpPairedAddress::GpRel { access_info: None },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::a0,
                    rt: Gpr::t6,
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_04() {
        static BYTES: [u8; 3 * 4] = [
            0x00, 0x9C, 0x08, 0x21, // addu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x90, 0x22, 0x80, 0x11, // lbu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 3] = [
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x800080A0),
                hi_rom: None,
            }),
            None,
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800000B1),
                deref_rom: Rom::new(0x00010008),
                access_info: (AccessType::BYTE, true),

                info: GprRegDereferencedAddress::GpRel {},
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 3] = [
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::at,
                    rs: Gpr::a0,
                    rt: Gpr::gp,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000B1),
                info: InstrOpPairedAddress::GpRel {
                    access_info: Some((AccessType::BYTE, true)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_05() {
        static BYTES: [u8; 4 * 4] = [
            0x00, 0x04, 0x70, 0x80, // sll
            0x27, 0x8F, 0x80, 0x14, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x01, 0xCF, 0x10, 0x21, // addu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000B4),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::GpRel {},
            }),
            None,
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000B4),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::GpRel {},
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_sll,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000B4),
                info: InstrOpPairedAddress::GpRel { access_info: None },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::t6,
                    rt: Gpr::t7,
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_06() {
        static BYTES: [u8; 4 * 4] = [
            0x00, 0x04, 0x70, 0x80, // sll
            0x01, 0xDC, 0x08, 0x21, // addu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x8C, 0x22, 0x80, 0x18, // lw
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x800080A0),
                hi_rom: None,
            }),
            None,
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800000B8),
                deref_rom: Rom::new(0x0001000C),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::GpRel {},
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_sll,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::at,
                    rs: Gpr::t6,
                    rt: Gpr::gp,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000B8),
                info: InstrOpPairedAddress::GpRel {
                    access_info: Some((AccessType::WORD, false)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_07() {
        static BYTES: [u8; 5 * 4] = [
            0x3C, 0x0F, 0x80, 0x00, // lui
            0x25, 0xEF, 0x00, 0x90, // addiu
            0x00, 0x04, 0x70, 0x80, // sll
            0x03, 0xE0, 0x00, 0x08, // jr
            0x01, 0xCF, 0x10, 0x21, // addu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 5] = [
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000090),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::Garbage),
            None,
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000090),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 5] = [
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t7,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000090),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_sll,
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::t6,
                    rt: Gpr::t7,
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_08() {
        static BYTES: [u8; 6 * 4] = [
            0x3C, 0x02, 0x12, 0x34, // lui
            0x34, 0x42, 0x56, 0x78, // ori
            0x3C, 0x03, 0x87, 0x65, // lui
            0x00, 0x64, 0x18, 0x25, // or
            0x03, 0xE0, 0x00, 0x08, // jr
            0x34, 0x63, 0x43, 0x00, // ori
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 6] = [
            Some(GprRegisterValue::Hi {
                value: 0x12340000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::ConstantInfo {
                setter_rom: Rom::new(0x00010004),
                info: GprRegConstantInfo::Constant {
                    value: 0x12345678,
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::Hi {
                value: 0x87650000,
                rom: Rom::new(0x00010008),
            }),
            Some(GprRegisterValue::ConstantInfo {
                setter_rom: Rom::new(0x0001000C),
                info: GprRegConstantInfo::OredHi {
                    value: 0x87650000,
                    hi_rom: Rom::new(0x00010008),
                },
            }),
            None,
            Some(GprRegisterValue::ConstantInfo {
                setter_rom: Rom::new(0x00010014),
                info: GprRegConstantInfo::Constant {
                    value: 0x87654300,
                    hi_rom: Rom::new(0x00010008),
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 6] = [
            InstructionOperation::Hi {
                value: 0x12340000,
                dst_reg: Gpr::v0,
            },
            InstructionOperation::Constant {
                constant: 0x12345678,
                hi_rom: Rom::new(0x00010000),
            },
            InstructionOperation::Hi {
                value: 0x87650000,
                dst_reg: Gpr::v1,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::Or {
                    rd: Gpr::v1,
                    rs: Gpr::v1,
                    rt: Gpr::a0,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::Constant {
                constant: 0x87654300,
                hi_rom: Rom::new(0x00010008),
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_09() {
        static BYTES: [u8; 28 * 4] = [
            0x27, 0xBD, 0xFF, 0xE0, // addiu
            0xAF, 0xBF, 0x00, 0x18, // sw
            0x00, 0x04, 0x70, 0x80, // sll
            0x3C, 0x01, 0x80, 0x00, // lui
            0x00, 0x2E, 0x08, 0x21, // addu
            0x8C, 0x2E, 0x00, 0xC0, // lw
            0x01, 0xC0, 0x00, 0x08, // jr
            0x00, 0x00, 0x00, 0x00, // nop
            0x04, 0x11, 0x00, 0x13, // bal
            0x00, 0x00, 0x00, 0x00, // nop
            0x0C, 0x00, 0x00, 0x1C, // jal
            0x00, 0x00, 0x00, 0x00, // nop
            0x08, 0x00, 0x00, 0x1C, // j
            0x27, 0xFF, 0x00, 0x08, // addiu
            0x3C, 0x19, 0x80, 0x00, // lui
            0x8F, 0x39, 0x01, 0x00, // lw
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x3C, 0x19, 0x80, 0x00, // lui
            0x27, 0x39, 0x00, 0x70, // addiu
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x00, 0x40, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBF, 0x00, 0x18, // lw
            0x27, 0xBD, 0x00, 0x20, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x00, 0x00, 0x00, 0x00, // nop
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 28] = [
            Some(GprRegisterValue::StackPointer { offset: -0x20 }),
            None,
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x0001000C),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x0001000C),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800000C0),
                deref_rom: Rom::new(0x00010014),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x0001000C),
                },
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010038),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000100),
                deref_rom: Rom::new(0x0001003C),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010038),
                },
            }),
            None,
            None,
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010048),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000070),
                setter_rom: Rom::new(0x0001004C),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010048),
                },
            }),
            None,
            None,
            None,
            None,
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::StackPointer { offset: 0 }),
            None,
            None,
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 28] = [
            InstructionOperation::DanglingLo { imm: -0x20 },
            InstructionOperation::DanglingLo { imm: 0x18 },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_sll,
            },
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::at,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::at,
                    rs: Gpr::at,
                    rt: Gpr::t6,
                },
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000C0),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x0001000C),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            InstructionOperation::JumptableJump {
                jumptable_vram: Vram::new(0x800000C0),
                dereferenced_rom: Rom::new(0x00010014),
                info: InstrOpJumptable::Simple,
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::Link {
                info: InstrOpLink::LinkingBranch {
                    target_vram: Vram::new(0x80000070),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::Link {
                info: InstrOpLink::DirectLinkingCall {
                    target_vram: Vram::new(0x80000070),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::TailCall {
                info: InstrOpTailCall::MaybeDirectTailCall {
                    target_vram: Vram::new(0x80000070),
                },
            },
            InstructionOperation::DanglingLo { imm: 0x8 },
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t9,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000100),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010038),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            InstructionOperation::Link {
                info: InstrOpLink::DereferencedRegisterLink {
                    dereferenced_vram: Vram::new(0x80000100),
                    dereferenced_rom: Rom::new(0x0001003C),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t9,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000070),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010048),
                    access_info: None,
                },
            },
            InstructionOperation::Link {
                info: InstrOpLink::RawRegisterLink {
                    vram: Vram::new(0x80000070),
                    rom: Rom::new(0x0001004C),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::Link {
                info: InstrOpLink::UnknownJumpAndLinkRegister { reg: Gpr::v0 },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            InstructionOperation::DanglingLo { imm: 0x20 },
            InstructionOperation::ReturnJump,
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_10() {
        static BYTES: [u8; 4 * 4] = [
            0x3C, 0x08, 0x80, 0x00, // lui
            0x8D, 0x08, 0x01, 0x00, // lw
            0x01, 0x00, 0x00, 0x08, // jr
            0x24, 0x05, 0x00, 0x01, // addiu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000100),
                deref_rom: Rom::new(0x00010004),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            None,
            Some(GprRegisterValue::Garbage),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t0,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000100),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            InstructionOperation::JumptableJump {
                jumptable_vram: Vram::new(0x80000100),
                dereferenced_rom: Rom::new(0x00010004),
                info: InstrOpJumptable::Simple,
            },
            InstructionOperation::DanglingLo { imm: 0x1 },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_11() {
        static BYTES: [u8; 8 * 4] = [
            0x3C, 0x08, 0x80, 0x00, // lui
            0x8D, 0x08, 0x01, 0x00, // lw
            0x11, 0x00, 0x00, 0x03, // beqz
            0x00, 0x00, 0x00, 0x00, // nop
            0x01, 0x00, 0x00, 0x08, // jr
            0x24, 0x05, 0x00, 0x02, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x00, 0x00, 0x10, 0x25, // or
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 8] = [
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000100),
                deref_rom: Rom::new(0x00010004),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            None,
            None,
            None,
            Some(GprRegisterValue::Garbage),
            None,
            Some(GprRegisterValue::SoftZero),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 8] = [
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t0,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000100),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            InstructionOperation::Branch {
                target_vram: Vram::new(0x80000018),
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::TailCall {
                info: InstrOpTailCall::DereferencedRegisterTailCall {
                    dereferenced_vram: Vram::new(0x80000100),
                    dereferenced_rom: Rom::new(0x00010004),
                },
            },
            InstructionOperation::DanglingLo { imm: 0x2 },
            InstructionOperation::ReturnJump,
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::Or {
                    rd: Gpr::v0,
                    rs: Gpr::zero,
                    rt: Gpr::zero,
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_12() {
        static BYTES: [u8; 4 * 4] = [
            0x3C, 0x08, 0x80, 0x00, // lui
            0x25, 0x08, 0x00, 0x70, // addiu
            0x01, 0x00, 0x00, 0x08, // jr
            0x24, 0x05, 0x00, 0x03, // addiu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Hi {
                value: 0x80000000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000070),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            None,
            Some(GprRegisterValue::Garbage),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::Hi {
                value: 0x80000000,
                dst_reg: Gpr::t0,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000070),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::TailCall {
                info: InstrOpTailCall::RawRegisterTailCall {
                    vram: Vram::new(0x80000070),
                    rom: Rom::new(0x00010004),
                },
            },
            InstructionOperation::DanglingLo { imm: 0x3 },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_13() {
        static BYTES: [u8; 9 * 4] = [
            0x3C, 0x1C, 0x80, 0x01, // lui
            0x27, 0x9C, 0x80, 0xA0, // addiu
            0x03, 0xE0, 0x80, 0x25, // or
            0x04, 0x11, 0x00, 0x01, // bal
            0x00, 0x00, 0x00, 0x00, // nop
            0x3C, 0x1C, 0x00, 0x01, // lui
            0x27, 0x9C, 0x80, 0x8C, // addiu
            0x03, 0x9F, 0xE0, 0x21, // addu
            0x02, 0x00, 0xF8, 0x25, // or
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 9] = [
            Some(GprRegisterValue::Hi {
                value: 0x80010000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x800080A0),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::Garbage),
            None,
            None,
            Some(GprRegisterValue::Hi {
                value: 0x00010000,
                rom: Rom::new(0x00010014),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x0000808C),
                setter_rom: Rom::new(0x00010018),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010014),
                },
            }),
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x800080A0),
                hi_rom: Some(Rom::new(0x00010014)),
            }),
            Some(GprRegisterValue::Garbage),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 9] = [
            InstructionOperation::Hi {
                value: 0x80010000,
                dst_reg: Gpr::gp,
            },
            InstructionOperation::GpSet {
                hi_rom: Rom::new(0x00010000),
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::Or {
                    rd: Gpr::s0,
                    rs: Gpr::ra,
                    rt: Gpr::zero,
                },
            },
            InstructionOperation::Link {
                info: InstrOpLink::LinkingBranch {
                    target_vram: Vram::new(0x80000014),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::Hi {
                value: 0x00010000,
                dst_reg: Gpr::gp,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x0000808C),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010014),
                    access_info: None,
                },
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::SuspectedCpload {
                    hi_rom: Rom::new(0x00010014),
                    lo_rom: Rom::new(0x00010018),
                },
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::Or {
                    rd: Gpr::ra,
                    rs: Gpr::s0,
                    rt: Gpr::zero,
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    /// Using a value from a dereferenced pointer to index a different pointer
    #[test]
    fn register_tracking_pairing_test_14() {
        static BYTES: [u8; 5 * 4] = [
            0x3C, 0x03, 0x80, 0x0F, // lui
            0x8C, 0x63, 0xFC, 0xD0, // lw
            0x3C, 0x02, 0x80, 0x0B, // lui
            0x00, 0x43, 0x10, 0x21, // addu
            0x90, 0x42, 0xCE, 0x84, // lbu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 5] = [
            Some(GprRegisterValue::Hi {
                value: 0x800F0000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800EFCD0),
                deref_rom: Rom::new(0x00010004),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::Hi {
                value: 0x800B0000,
                rom: Rom::new(0x00010008),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x800B0000,
                rom: Rom::new(0x00010008),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x800ACE84),
                deref_rom: Rom::new(0x00010010),
                access_info: (AccessType::BYTE, true),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010008),
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 5] = [
            InstructionOperation::Hi {
                value: 0x800F0000,
                dst_reg: Gpr::v1,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800EFCD0),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            InstructionOperation::Hi {
                value: 0x800B0000,
                dst_reg: Gpr::v0,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::v0,
                    rs: Gpr::v0,
                    rt: Gpr::v1,
                },
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800ACE84),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010008),
                    access_info: Some((AccessType::BYTE, true)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    /// Raw value + Hi
    #[test]
    fn register_tracking_pairing_test_15() {
        static BYTES: [u8; 4 * 4] = [
            0x24, 0x02, 0x03, 0xC4, // addiu
            0x3C, 0x01, 0x80, 0x12, // lui
            0x00, 0x22, 0x08, 0x21, // addu
            0xA0, 0x20, 0x37, 0x4C, // sb
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::Hi {
                value: 0x80120000,
                rom: Rom::new(0x00010004),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x80120000,
                rom: Rom::new(0x00010004),
            }),
            None,
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::DanglingLo { imm: 0x3C4 },
            InstructionOperation::Hi {
                value: 0x80120000,
                dst_reg: Gpr::at,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::at,
                    rs: Gpr::at,
                    rt: Gpr::v0,
                },
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x8012374C),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010004),
                    access_info: Some((AccessType::BYTE, false)),
                },
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_16() {
        static BYTES: [u8; 3 * 4] = [
            0x3C, 0x02, 0x80, 0x21, // lui
            0x24, 0x42, 0x65, 0x40, // addiu
            0x8C, 0x42, 0x00, 0x0C, // lw
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 3] = [
            Some(GprRegisterValue::Hi {
                value: 0x80210000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80216540),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80216540),
                deref_rom: Rom::new(0x00010008),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::HiLo {
                    lo_rom: Rom::new(0x00010004),
                    addend: 0xC,
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 3] = [
            InstructionOperation::Hi {
                value: 0x80210000,
                dst_reg: Gpr::v0,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80216540),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x80216540),
                addend: 0x0C,
                address_load_rom: Rom::new(0x00010004),
                access_info: (AccessType::WORD, false),
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    /// Huge stack
    #[test]
    fn register_tracking_pairing_test_17() {
        static BYTES: [u8; 7 * 4] = [
            0x3C, 0x01, 0x00, 0x01, // lui
            0x34, 0x21, 0x00, 0x28, // ori
            0x03, 0xA1, 0xE8, 0x23, // subu
            0x34, 0x01, 0x97, 0x20, // ori
            0x03, 0xA1, 0xE8, 0x23, // subu
            0x24, 0x01, 0xFF, 0xF0, // addiu
            0x03, 0xA1, 0xE8, 0x24, // and
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 7] = [
            Some(GprRegisterValue::Hi {
                value: 0x00010000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::ConstantInfo {
                setter_rom: Rom::new(0x00010004),
                info: GprRegConstantInfo::Constant {
                    value: 0x00010028,
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::StackPointer {
                offset: -0x00010028,
            }),
            Some(GprRegisterValue::ConstantInfo {
                setter_rom: Rom::new(0x0001000C),
                info: GprRegConstantInfo::SmallConstant { value: 0x9720 },
            }),
            Some(GprRegisterValue::StackPointer {
                offset: -0x00019748,
            }),
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::StackPointer {
                offset: -0x00019748,
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 7] = [
            InstructionOperation::Hi {
                value: 0x00010000,
                dst_reg: Gpr::at,
            },
            InstructionOperation::Constant {
                constant: 0x00010028,
                hi_rom: Rom::new(0x00010000),
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterSubtraction {
                    rd: Gpr::sp,
                    rs: Gpr::sp,
                    rt: Gpr::at,
                },
            },
            InstructionOperation::UnpairedConstant { imm: 0x9720 },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterSubtraction {
                    rd: Gpr::sp,
                    rs: Gpr::sp,
                    rt: Gpr::at,
                },
            },
            InstructionOperation::DanglingLo { imm: -0x10 },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_and,
            },
        ];
        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x800080A0)));
        let global_offset_table = None;

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_18() {
        static BYTES: [u8; 4 * 4] = [
            0x8F, 0x8E, 0x80, 0x18, // lw
            0x25, 0xCE, 0x00, 0x80, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x8D, 0xC2, 0x00, 0x04, // lw
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000000),
                setter_rom: Rom::new(0x00010000),
                info: GprRegRawAddress::GpGotLocal {},
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000080),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x00010000),
                },
            }),
            None,
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000080),
                deref_rom: Rom::new(0x0001000C),
                access_info: (AccessType::WORD, false),
                info: GprRegDereferencedAddress::PairedGpGotLo {
                    lo_rom: Rom::new(0x00010004),
                    addend: 0x4,
                },
            }),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000000),
                info: InstrOpPairedAddress::GpGotLocal {},
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000080),
                info: InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x80000080),
                addend: 0x4,
                address_load_rom: Rom::new(0x00010004),
                access_info: (AccessType::WORD, false),
            },
        ];

        let got_locals = vec![
            /* -0x7FF0($gp) */ GotLocalEntry::new(0x0F000000), /* Lazy resolver */
            /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000), /* GNU extension */
            /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000), /* */
        ];
        let got_globals = vec![];
        let got = GlobalOffsetTable::new(Vram::new(0x80000090), got_locals, got_globals);

        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x80008080)));
        let global_offset_table = Some(&got);

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn register_tracking_pairing_test_19() {
        static BYTES: [u8; 4 * 4] = [
            0x8F, 0x8E, 0x80, 0x18, // lw
            0x25, 0xCE, 0x00, 0x80, // addiu
            0x03, 0xE0, 0x00, 0x08, // jr
            0x25, 0xCF, 0x00, 0x04, // addiu
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 4] = [
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000000),
                setter_rom: Rom::new(0x00010000),
                info: GprRegRawAddress::GpGotLocal {},
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000080),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x00010000),
                },
            }),
            None,
            Some(GprRegisterValue::Garbage),
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 4] = [
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000000),
                info: InstrOpPairedAddress::GpGotLocal {},
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000080),
                info: InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::ReturnJump,
            InstructionOperation::DanglingLo { imm: 0x4 },
        ];

        let got_locals = vec![
            /* -0x7FF0($gp) */ GotLocalEntry::new(0x0F000000), /* Lazy resolver */
            /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000), /* GNU extension */
            /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000), /* */
        ];
        let got_globals = vec![];
        let got = GlobalOffsetTable::new(Vram::new(0x80000090), got_locals, got_globals);

        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x80008080)));
        let global_offset_table = Some(&got);

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }

    #[test]
    fn bunch_of_gp_pairing_combos() {
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

        lui     $v0, %hi(some_var+0x4)
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

        lui     $a0, %got_hi(some_var)
        addu    $a0, $a0, $gp
        lw      $a0, %got_lo(some_var)($a0)
        lw      $a1, 0x8($a0)
        lw      $a2, 0xC($a0)
        lw      $a0, 0x4($a0)

        lui     $t9, %call_hi(global_function)
        addu    $t9, $t9, $gp
        lw      $t9, %call_lo(global_function)($t9)
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
        static BYTES: [u8; 49 * 4] = [
            // _gp_disp
            0x3C, 0x1C, 0x00, 0x01, // lui
            0x27, 0x9C, 0x81, 0x00, // addiu
            0x03, 0x99, 0xE0, 0x21, // addu
            //
            0x27, 0xBD, 0xFF, 0xE0, // addiu
            0xAF, 0xBF, 0x00, 0x10, // sw
            0xAF, 0xBC, 0x00, 0x18, // sw
            // some_var
            0x3C, 0x04, 0x80, 0x00, // lui
            0x24, 0x85, 0x01, 0x2C, // addiu
            0x8C, 0xA4, 0x00, 0x08, // lw
            // some_var + 0x4
            0x3C, 0x02, 0x80, 0x00, // lui
            0x8C, 0x43, 0x01, 0x30, // lw
            // some_var + 0x4
            0x8F, 0x86, 0x80, 0x30, // lw
            // some_var + 0x8
            0x8F, 0x87, 0x80, 0x1C, // lw
            // static_sym
            0x8F, 0x88, 0x80, 0x18, // lw
            0x8D, 0x09, 0x01, 0x3C, // lw
            // global_function
            0x8F, 0x99, 0x80, 0x20, // lw
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBC, 0x00, 0x18, // lw
            // global_function
            0x8F, 0x99, 0x80, 0x20, // lw
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBC, 0x00, 0x18, // lw
            // non_global_function
            0x8F, 0x99, 0x80, 0x18, // lw
            0x27, 0x39, 0x00, 0xCC, // addiu
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBC, 0x00, 0x18, // lw
            // func_arr
            0x8F, 0x99, 0x80, 0x24, // lw
            0x8F, 0x39, 0x00, 0x04, // lw
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBC, 0x00, 0x18, // lw
            // some_var
            0x3C, 0x04, 0x00, 0x00, // lui
            0x00, 0x9C, 0x20, 0x21, // addu
            0x8C, 0x84, 0x80, 0x28, // lw
            0x8C, 0x85, 0x00, 0x08, // lw
            0x8C, 0x86, 0x00, 0x0C, // lw
            0x8C, 0x84, 0x00, 0x04, // lw
            // global_function
            0x3C, 0x19, 0x00, 0x00, // lui
            0x03, 0x3C, 0xC8, 0x21, // addu
            0x8F, 0x39, 0x80, 0x20, // lw
            0x03, 0x20, 0xF8, 0x09, // jalr
            0x00, 0x00, 0x00, 0x00, // nop
            0x8F, 0xBC, 0x00, 0x18, // lw
            //
            0x8F, 0xBF, 0x00, 0x10, // lw
            0x27, 0xBD, 0x00, 0x20, // lw
            //
            0x03, 0xE0, 0x00, 0x08, // jr
            0x00, 0x00, 0x00, 0x00, // nop
        ];
        static EXPECTED_GPR_VALUES: [Option<GprRegisterValue>; 49] = [
            Some(GprRegisterValue::Hi {
                value: 0x00010000,
                rom: Rom::new(0x00010000),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x00008100),
                setter_rom: Rom::new(0x00010004),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010000),
                },
            }),
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::StackPointer { offset: -0x20 }),
            None,
            None,
            Some(GprRegisterValue::Hi {
                value: 2147483648,
                rom: Rom::new(0x00010018),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x8000012C),
                setter_rom: Rom::new(0x0001001C),
                info: GprRegRawAddress::HiLo {
                    hi_rom: Rom::new(0x00010018),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000012C),
                deref_rom: Rom::new(0x00010020),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::HiLo {
                    lo_rom: Rom::new(0x0001001C),
                    addend: 0x8,
                },
            }),
            Some(GprRegisterValue::Hi {
                value: 2147483648,
                rom: Rom::new(0x00010024),
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000130),
                deref_rom: Rom::new(0x00010028),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::Hi {
                    hi_rom: Rom::new(0x00010024),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x80000130),
                deref_rom: Rom::new(0x0001002C),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::GpRel {},
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000134),
                setter_rom: Rom::new(0x00010030),
                info: GprRegRawAddress::GpGotGlobal {},
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000000),
                setter_rom: Rom::new(0x00010034),
                info: GprRegRawAddress::GpGotLocal {},
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000013C),
                deref_rom: Rom::new(0x00010038),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::GpGotLocal {
                    upper_rom: Rom::new(0x00010034),
                },
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000C4),
                setter_rom: Rom::new(0x0001003C),
                info: GprRegRawAddress::GpGotGlobal {},
            }),
            None,
            None,
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000C4),
                setter_rom: Rom::new(0x0001004C),
                info: GprRegRawAddress::GpGotGlobal {},
            }),
            None,
            None,
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x80000000),
                setter_rom: Rom::new(0x0001005C),
                info: GprRegRawAddress::GpGotLocal {},
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000CC),
                setter_rom: Rom::new(0x00010060),
                info: GprRegRawAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x0001005C),
                },
            }),
            None,
            None,
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x8000014C),
                setter_rom: Rom::new(0x00010070),
                info: GprRegRawAddress::GpGotGlobal {},
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000014C),
                deref_rom: Rom::new(0x00010074),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::GpGotGlobal {
                    addend: 0x4,
                    upper_rom: Rom::new(0x00010070),
                },
            }),
            None,
            None,
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::Hi {
                value: 0x00000000,
                rom: Rom::new(0x00010084),
            }),
            Some(GprRegisterValue::HiGp {
                value: 0x80008100,
                rom: Rom::new(0x00010088),
                hi_rom: Rom::new(0x00010084),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x8000012C),
                setter_rom: Rom::new(0x0001008C),
                info: GprRegRawAddress::HiLoGp {
                    hi_rom: Rom::new(0x00010084),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000012C),
                deref_rom: Rom::new(0x00010090),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::HiLoGp {
                    addend: 0x8,
                    lo_rom: Rom::new(0x0001008C),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000012C),
                deref_rom: Rom::new(0x00010094),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::HiLoGp {
                    addend: 0xC,
                    lo_rom: Rom::new(0x0001008C),
                },
            }),
            Some(GprRegisterValue::DereferencedAddress {
                original_address: Vram::new(0x8000012C),
                deref_rom: Rom::new(0x00010098),
                access_info: (AccessType::WORD, false),

                info: GprRegDereferencedAddress::HiLoGp {
                    addend: 0x4,
                    lo_rom: Rom::new(0x0001008C),
                },
            }),
            Some(GprRegisterValue::Hi {
                value: 0,
                rom: Rom::new(0x0001009C),
            }),
            Some(GprRegisterValue::HiGp {
                value: 0x80008100,
                rom: Rom::new(0x000100A0),
                hi_rom: Rom::new(0x0001009C),
            }),
            Some(GprRegisterValue::RawAddress {
                vram: Vram::new(0x800000C4),
                setter_rom: Rom::new(0x000100A4),
                info: GprRegRawAddress::HiLoGp {
                    hi_rom: Rom::new(0x0001009C),
                },
            }),
            None,
            None,
            Some(GprRegisterValue::GlobalPointer {
                gp: GpValue::new(0x80008100),
                hi_rom: Some(Rom::new(0x00010000)),
            }),
            Some(GprRegisterValue::Garbage),
            Some(GprRegisterValue::StackPointer { offset: 0 }),
            None,
            None,
        ];
        static EXPECTED_OPERATIONS: [InstructionOperation; 49] = [
            // `.cpload`
            InstructionOperation::Hi {
                dst_reg: Gpr::gp,
                value: 0x00010000,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x00008100),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010000),
                    access_info: None,
                },
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::SuspectedCpload {
                    hi_rom: Rom::new(0x00010000),
                    lo_rom: Rom::new(0x00010004),
                },
            },
            //
            InstructionOperation::DanglingLo { imm: -0x20 },
            InstructionOperation::DanglingLo { imm: 0x10 },
            InstructionOperation::DanglingLo { imm: 0x18 },
            // some_var
            InstructionOperation::Hi {
                dst_reg: Gpr::a0,
                value: 0x80000000,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x8000012C),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010018),
                    access_info: None,
                },
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x8000012C),
                addend: 0x8,
                address_load_rom: Rom::new(0x0001001C),
                access_info: (AccessType::WORD, false),
            },
            // some_var + 0x4
            InstructionOperation::Hi {
                dst_reg: Gpr::v0,
                value: 0x80000000,
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000130),
                info: InstrOpPairedAddress::PairedLo {
                    hi_rom: Rom::new(0x00010024),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            // some_var + 0x4
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000130),
                info: InstrOpPairedAddress::GpRel {
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            // some_var + 0x8
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000134),
                info: InstrOpPairedAddress::GpGotGlobal {},
            },
            // static_sym
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000000),
                info: InstrOpPairedAddress::GpGotLocal {},
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x8000013C),
                info: InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x00010034),
                    access_info: Some((AccessType::WORD, false)),
                },
            },
            // global_function
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000C4),
                info: InstrOpPairedAddress::GpGotGlobal {},
            },
            InstructionOperation::Link {
                info: InstrOpLink::Call16RegisterLink {
                    vram: Vram::new(0x800000C4),
                    rom: Rom::new(0x0001003C),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            // global_function
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000C4),
                info: InstrOpPairedAddress::GpGotGlobal {},
            },
            InstructionOperation::Link {
                info: InstrOpLink::Call16RegisterLink {
                    vram: Vram::new(0x800000C4),
                    rom: Rom::new(0x0001004C),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            // non_global_function
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x80000000),
                info: InstrOpPairedAddress::GpGotLocal {},
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000CC),
                info: InstrOpPairedAddress::PairedGpGotLo {
                    upper_rom: Rom::new(0x0001005C),
                    access_info: None,
                },
            },
            InstructionOperation::Link {
                info: InstrOpLink::RawRegisterLink {
                    vram: Vram::new(0x800000CC),
                    rom: Rom::new(0x00010060),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            // func_arr
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x8000014C),
                info: InstrOpPairedAddress::GpGotGlobal {},
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x8000014C),
                addend: 0x4,
                address_load_rom: Rom::new(0x00010070),
                access_info: (AccessType::WORD, false),
            },
            InstructionOperation::Link {
                info: InstrOpLink::DereferencedRegisterLink {
                    dereferenced_vram: Vram::new(0x8000014C),
                    dereferenced_rom: Rom::new(0x00010074),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            // some_var
            InstructionOperation::Hi {
                dst_reg: Gpr::a0,
                value: 0x0,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::a0,
                    rs: Gpr::a0,
                    rt: Gpr::gp,
                },
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x8000012C),
                info: InstrOpPairedAddress::PairedGotLo {
                    hi_rom: Rom::new(0x00010084),
                },
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x8000012C),
                addend: 0x8,
                address_load_rom: Rom::new(0x0001008C),
                access_info: (AccessType::WORD, false),
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x8000012C),
                addend: 0xC,
                address_load_rom: Rom::new(0x0001008C),
                access_info: (AccessType::WORD, false),
            },
            InstructionOperation::DereferencedRawAddress {
                original_address: Vram::new(0x8000012C),
                addend: 0x4,
                address_load_rom: Rom::new(0x0001008C),
                access_info: (AccessType::WORD, false),
            },
            // global_function
            InstructionOperation::Hi {
                dst_reg: Gpr::t9,
                value: 0x0,
            },
            InstructionOperation::RegisterOperation {
                info: InstrOpRegisterOperation::RegisterAddition {
                    rd: Gpr::t9,
                    rs: Gpr::t9,
                    rt: Gpr::gp,
                },
            },
            InstructionOperation::PairedAddress {
                vram: Vram::new(0x800000C4),
                info: InstrOpPairedAddress::PairedGotLo {
                    hi_rom: Rom::new(0x0001009C),
                },
            },
            InstructionOperation::Link {
                info: InstrOpLink::CallHiLoRegisterLink {
                    vram: Vram::new(0x800000C4),
                    rom: Rom::new(0x000100A4),
                },
            },
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
            InstructionOperation::DanglingLo { imm: 0x18 },
            //
            InstructionOperation::DanglingLo { imm: 0x10 },
            InstructionOperation::DanglingLo { imm: 0x20 },
            //
            InstructionOperation::ReturnJump,
            InstructionOperation::UnhandledOpcode {
                opcode: Opcode::core_nop,
            },
        ];

        let got_locals = vec![
            /* -0x7FF0($gp) */ GotLocalEntry::new(0x0F000000), /* Lazy resolver */
            /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000), /* GNU extension */
            /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000), /* */
        ];
        let got_globals = vec![
            /* -0x7FE4($gp) */
            GotGlobalEntry::new(0x80000134, 0x80000134, false), /* some_var + 0x8 */
            /* -0x7FE0($gp) */
            GotGlobalEntry::new(0x800000C4, 0x800000C4, false), /* global_function */
            /* -0x7FDC($gp) */
            GotGlobalEntry::new(0x8000014C, 0x8000014C, false), /* func_arr */
            /* -0x7FD8($gp) */
            GotGlobalEntry::new(0x8000012C, 0x8000012C, false), /* some_var */
        ];
        let got = GlobalOffsetTable::new(Vram::new(0x80000110), got_locals, got_globals);

        let gp_config = Some(GpConfig::new_pic(GpValue::new(0x80008100)));
        let global_offset_table = Some(&got);

        register_tracking_general_test(
            &BYTES,
            &EXPECTED_GPR_VALUES,
            &EXPECTED_OPERATIONS,
            gp_config,
            global_offset_table,
        );
    }
}
