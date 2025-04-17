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
        vram: Vram,
    },
    PairedLo {
        vram: Vram,
    },
    PairedHiUnaligned {
        unaddended_vram: Vram,
        addended_vram: Vram,
    },
    PairedLoUnaligned {
        unaddended_vram: Vram,
        addended_vram: Vram,
    },
    ConstantHi {
        constant: u32,
    },
    ConstantLo {
        constant: u32,
    },
    GpRel {
        vram: Vram,
    },
    GpRelUnaligned {
        unaddended_vram: Vram,
        addended_vram: Vram,
    },
    GotLazyResolver {
        vram: Vram,
    },
    GotGlobal {
        vram: Vram,
    },
    GotLocal {
        vram: Vram,
    },
    GotLocalPaired {
        vram: Vram,
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
