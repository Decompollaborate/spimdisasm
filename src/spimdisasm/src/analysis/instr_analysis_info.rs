/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::{Rom, Vram};

use super::InstrOpJumptable;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InstrAnalysisInfo {
    No,

    DirectLink {
        target_vram: Vram,
    },
    BranchLink {
        target_vram: Vram,
    },
    JumpAndLinkRegisterRaw {
        raw_vram: Vram,
    },
    JumpAndLinkRegisterDereferenced {
        dereferenced_vram: Vram,
    },

    MaybeDirectTailCall {
        target_vram: Vram,
    },
    RawRegisterTailCall {
        raw_vram: Vram,
    },
    DereferencedRegisterTailCall {
        dereferenced_vram: Vram,
    },

    Jumptable {
        jumptable_vram: Vram,
        kind: InstrOpJumptable,
    },

    Branch {
        target_vram: Vram,
    },
    BranchOutside {
        target_vram: Vram,
    },

    UnpairedHi {
        value: u32,
    },
    PairedHi {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    PairedLo {
        addended_vram: Vram,
        unaddended_vram: Vram,
        upper_rom: Rom,
    },
    ConstantHi {
        constant: u32,
    },
    ConstantLo {
        constant: u32,
        upper_rom: Rom,
    },
    GpRel {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    GotLazyResolver {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    GotGlobal {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    GotLocal {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    GotLocalPaired {
        addended_vram: Vram,
        unaddended_vram: Vram,
    },
    GotCall16 {
        vram: Vram,
    },
    PairedGotHi {
        vram: Vram,
    },
    PairedGotLo {
        vram: Vram,
    },
    GotCallHi {
        vram: Vram,
    },
    GotCallLo {
        vram: Vram,
    },

    GpSetHi,
    GpSetLo,

    CploadHi,
    CploadLo,
    CploadAddu,
}

impl InstrAnalysisInfo {
    pub(crate) fn upper_rom(&self) -> Option<Rom> {
        match self {
            InstrAnalysisInfo::No => None,
            InstrAnalysisInfo::DirectLink { target_vram: _ } => None,
            InstrAnalysisInfo::BranchLink { target_vram: _ } => None,
            InstrAnalysisInfo::JumpAndLinkRegisterRaw { raw_vram: _ } => None,
            InstrAnalysisInfo::JumpAndLinkRegisterDereferenced {
                dereferenced_vram: _,
            } => None,
            InstrAnalysisInfo::MaybeDirectTailCall { target_vram: _ } => None,
            InstrAnalysisInfo::RawRegisterTailCall { raw_vram: _ } => None,
            InstrAnalysisInfo::DereferencedRegisterTailCall {
                dereferenced_vram: _,
            } => None,
            InstrAnalysisInfo::Jumptable {
                jumptable_vram: _,
                kind: _,
            } => None,
            InstrAnalysisInfo::Branch { target_vram: _ } => None,
            InstrAnalysisInfo::BranchOutside { target_vram: _ } => None,
            InstrAnalysisInfo::UnpairedHi { value: _ } => None,
            InstrAnalysisInfo::PairedHi {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::PairedLo {
                addended_vram: _,
                unaddended_vram: _,
                upper_rom,
            } => Some(*upper_rom),
            InstrAnalysisInfo::ConstantHi { constant: _ } => None,
            InstrAnalysisInfo::ConstantLo {
                constant: _,
                upper_rom,
            } => Some(*upper_rom),
            InstrAnalysisInfo::GpRel {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::GotLazyResolver {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::GotGlobal {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::GotLocal {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::GotLocalPaired {
                addended_vram: _,
                unaddended_vram: _,
            } => None,
            InstrAnalysisInfo::GotCall16 { vram: _ } => None,
            InstrAnalysisInfo::PairedGotHi { vram: _ } => None,
            InstrAnalysisInfo::PairedGotLo { vram: _ } => None,
            InstrAnalysisInfo::GotCallHi { vram: _ } => None,
            InstrAnalysisInfo::GotCallLo { vram: _ } => None,
            InstrAnalysisInfo::GpSetHi => None,
            InstrAnalysisInfo::GpSetLo => None,
            InstrAnalysisInfo::CploadHi => None,
            InstrAnalysisInfo::CploadLo => None,
            InstrAnalysisInfo::CploadAddu => None,
        }
    }

    pub(crate) fn align_down_unaddended(self, alignment: u8) -> Self {
        match self {
            InstrAnalysisInfo::PairedHi {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::PairedHi {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            InstrAnalysisInfo::PairedLo {
                addended_vram,
                unaddended_vram: _,
                upper_rom,
            } => InstrAnalysisInfo::PairedLo {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
                upper_rom,
            },
            InstrAnalysisInfo::GpRel {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::GpRel {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            InstrAnalysisInfo::GotLazyResolver {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::GotLazyResolver {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            InstrAnalysisInfo::GotGlobal {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::GotGlobal {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            InstrAnalysisInfo::GotLocal {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::GotLocal {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            InstrAnalysisInfo::GotLocalPaired {
                addended_vram,
                unaddended_vram: _,
            } => InstrAnalysisInfo::GotLocalPaired {
                addended_vram,
                unaddended_vram: addended_vram.align_down(alignment),
            },
            x => x,
        }
    }
}
