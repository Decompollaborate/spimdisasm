/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use crate::symbols::processed::EitherFuncDataSymProcessedRef;

use super::{FunctionDisplay, SymCommonDisplaySettings, SymDataDisplay};

#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub enum EitherFuncDataDisplay<'ctx, 'sym, 'func_flg, 'data_flg> {
    Func(FunctionDisplay<'ctx, 'sym, 'func_flg>),
    Data(SymDataDisplay<'ctx, 'sym, 'data_flg>),
}

impl<'sym> EitherFuncDataDisplay<'_, 'sym, '_, '_> {
    #[must_use]
    pub(crate) fn sym(&self) -> EitherFuncDataSymProcessedRef<'sym> {
        match self {
            Self::Func(function_display) => {
                EitherFuncDataSymProcessedRef::Func(function_display.sym())
            }
            Self::Data(sym_data_display) => {
                EitherFuncDataSymProcessedRef::Data(sym_data_display.sym())
            }
        }
    }

    #[must_use]
    pub(crate) fn settings_common(&self) -> &SymCommonDisplaySettings {
        match self {
            Self::Func(function_display) => function_display.settings_common(),
            Self::Data(sym_data_display) => sym_data_display.settings_common(),
        }
    }
}

impl fmt::Display for EitherFuncDataDisplay<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Func(function_display) => function_display.fmt(f),
            Self::Data(sym_data_display) => sym_data_display.fmt(f),
        }
    }
}
