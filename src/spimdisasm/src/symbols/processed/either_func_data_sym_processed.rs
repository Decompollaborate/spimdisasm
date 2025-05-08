/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{
    addresses::{AddressRange, RomVramRange, Vram},
    context::Context,
    parent_segment_info::ParentSegmentInfo,
    relocation::RelocationInfo,
    section_type::SectionType,
    symbols::{
        display::{
            EitherFuncDataDisplay, FunctionDisplaySettings, InternalSymDisplSettings,
            SymDataDisplaySettings, SymDisplayError,
        },
        RomSymbol, RomSymbolProcessed, Symbol, SymbolProcessed,
    },
};

use super::{DataSymProcessed, FunctionSymProcessed};

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum EitherFuncDataSymProcessed {
    Func(FunctionSymProcessed),
    Data(DataSymProcessed),
}

impl<'ctx, 'sym, 'func_flg, 'data_flg> EitherFuncDataSymProcessed {
    pub fn display(
        &'sym self,
        context: &'ctx Context,
        func_settings: &'func_flg FunctionDisplaySettings,
        data_settings: &'data_flg SymDataDisplaySettings,
    ) -> Result<EitherFuncDataDisplay<'ctx, 'sym, 'func_flg, 'data_flg>, SymDisplayError> {
        self.display_internal(
            context,
            func_settings,
            data_settings,
            InternalSymDisplSettings::new(false),
        )
    }

    pub(crate) fn display_internal(
        &'sym self,
        context: &'ctx Context,
        func_settings: &'func_flg FunctionDisplaySettings,
        data_settings: &'data_flg SymDataDisplaySettings,
        internal_settings: InternalSymDisplSettings,
    ) -> Result<EitherFuncDataDisplay<'ctx, 'sym, 'func_flg, 'data_flg>, SymDisplayError> {
        Ok(match self {
            Self::Func(function_sym_processed) => {
                EitherFuncDataDisplay::Func(function_sym_processed.display_internal(
                    context,
                    func_settings,
                    internal_settings,
                )?)
            }
            Self::Data(data_sym_processed) => EitherFuncDataDisplay::Data(
                data_sym_processed.display_internal(context, data_settings, internal_settings)?,
            ),
        })
    }
}

impl Symbol for EitherFuncDataSymProcessed {
    fn vram_range(&self) -> &AddressRange<Vram> {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.vram_range(),
            Self::Data(data_sym_processed) => data_sym_processed.vram_range(),
        }
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.parent_segment_info(),
            Self::Data(data_sym_processed) => data_sym_processed.parent_segment_info(),
        }
    }

    fn section_type(&self) -> SectionType {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.section_type(),
            Self::Data(data_sym_processed) => data_sym_processed.section_type(),
        }
    }
}
impl RomSymbol for EitherFuncDataSymProcessed {
    fn rom_vram_range(&self) -> &RomVramRange {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.rom_vram_range(),
            Self::Data(data_sym_processed) => data_sym_processed.rom_vram_range(),
        }
    }
}
impl SymbolProcessed for EitherFuncDataSymProcessed {}
impl RomSymbolProcessed for EitherFuncDataSymProcessed {
    fn relocs(&self) -> &[Option<RelocationInfo>] {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.relocs(),
            Self::Data(data_sym_processed) => data_sym_processed.relocs(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd)]
pub enum EitherFuncDataSymProcessedRef<'sym> {
    Func(&'sym FunctionSymProcessed),
    Data(&'sym DataSymProcessed),
}

impl Symbol for EitherFuncDataSymProcessedRef<'_> {
    fn vram_range(&self) -> &AddressRange<Vram> {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.vram_range(),
            Self::Data(data_sym_processed) => data_sym_processed.vram_range(),
        }
    }

    fn parent_segment_info(&self) -> &ParentSegmentInfo {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.parent_segment_info(),
            Self::Data(data_sym_processed) => data_sym_processed.parent_segment_info(),
        }
    }

    fn section_type(&self) -> SectionType {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.section_type(),
            Self::Data(data_sym_processed) => data_sym_processed.section_type(),
        }
    }
}
impl RomSymbol for EitherFuncDataSymProcessedRef<'_> {
    fn rom_vram_range(&self) -> &RomVramRange {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.rom_vram_range(),
            Self::Data(data_sym_processed) => data_sym_processed.rom_vram_range(),
        }
    }
}
impl SymbolProcessed for EitherFuncDataSymProcessedRef<'_> {}
impl RomSymbolProcessed for EitherFuncDataSymProcessedRef<'_> {
    fn relocs(&self) -> &[Option<RelocationInfo>] {
        match self {
            Self::Func(function_sym_processed) => function_sym_processed.relocs(),
            Self::Data(data_sym_processed) => data_sym_processed.relocs(),
        }
    }
}
