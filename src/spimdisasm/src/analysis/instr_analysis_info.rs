/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::addresses::Vram;

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
    },
    ConstantHi {
        constant: u32,
    },
    ConstantLo {
        constant: u32,
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
