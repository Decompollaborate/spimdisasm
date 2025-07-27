/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{AddressRange, RomVramRange, Vram},
    parent_segment_info::ParentSegmentInfo,
    section_type::SectionType,
    symbols::{
        processed::EitherFuncDataSymProcessed, RomSymbol, RomSymbolPreprocessed, Symbol,
        SymbolPreprocessed,
    },
};

use super::{DataSym, FunctionSym};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum EitherFuncDataSym {
    Func(FunctionSym),
    Data(DataSym),
}

impl Symbol for EitherFuncDataSym {
    fn vram_range(&self) -> &AddressRange<Vram> {
        match self {
            EitherFuncDataSym::Func(function_sym) => function_sym.vram_range(),
            EitherFuncDataSym::Data(data_sym) => data_sym.vram_range(),
        }
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        match self {
            EitherFuncDataSym::Func(function_sym) => function_sym.parent_segment_info(),
            EitherFuncDataSym::Data(data_sym) => data_sym.parent_segment_info(),
        }
    }

    fn section_type(&self) -> SectionType {
        match self {
            EitherFuncDataSym::Func(function_sym) => function_sym.section_type(),
            EitherFuncDataSym::Data(data_sym) => data_sym.section_type(),
        }
    }
}
impl RomSymbol for EitherFuncDataSym {
    fn rom_vram_range(&self) -> &RomVramRange {
        match self {
            EitherFuncDataSym::Func(function_sym) => function_sym.rom_vram_range(),
            EitherFuncDataSym::Data(data_sym) => data_sym.rom_vram_range(),
        }
    }
}
impl SymbolPreprocessed for EitherFuncDataSym {
    type Output = EitherFuncDataSymProcessed;

    #[doc(hidden)]
    fn post_process(
        self,
        context: &mut crate::context::Context,
        user_relocs: &alloc::collections::btree_map::BTreeMap<
            crate::addresses::Rom,
            crate::relocation::RelocationInfo,
        >,
    ) -> Result<Self::Output, crate::symbols::SymbolPostProcessError> {
        Ok(match self {
            EitherFuncDataSym::Func(function_sym) => {
                EitherFuncDataSymProcessed::Func(function_sym.post_process(context, user_relocs)?)
            }
            EitherFuncDataSym::Data(data_sym) => {
                EitherFuncDataSymProcessed::Data(data_sym.post_process(context, user_relocs)?)
            }
        })
    }
}
impl RomSymbolPreprocessed for EitherFuncDataSym {}
