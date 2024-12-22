/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::symbols::SymbolData;

use super::FuncRodataPairing;

pub struct RodataIterator<'pairing, 'text, 'rodata> {
    pairing: &'pairing FuncRodataPairing<'text, 'rodata>,
    rodata_index: usize,
    late_rodata_index: usize,
}

impl<'pairing, 'text, 'rodata> RodataIterator<'pairing, 'text, 'rodata> {
    pub(super) fn new(pairing: &'pairing FuncRodataPairing<'text, 'rodata>) -> Self {
        Self {
            pairing,
            rodata_index: 0,
            late_rodata_index: 0,
        }
    }
}

impl<'rodata> Iterator for RodataIterator<'_, '_, 'rodata> {
    type Item = &'rodata SymbolData;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.pairing {
            FuncRodataPairing::SingleFunction { .. } => None,
            FuncRodataPairing::SingleRodata { rodata } => {
                if self.rodata_index == 0 {
                    self.rodata_index += 1;
                    Some(rodata)
                } else {
                    None
                }
            }
            FuncRodataPairing::Pairing {
                rodata_syms,
                late_rodata_syms,
                ..
            } => {
                let (sym, index) = if self.rodata_index < rodata_syms.len() {
                    (&rodata_syms[self.rodata_index], &mut self.rodata_index)
                } else if self.late_rodata_index < late_rodata_syms.len() {
                    (
                        &late_rodata_syms[self.late_rodata_index],
                        &mut self.late_rodata_index,
                    )
                } else {
                    return None;
                };

                *index += 1;
                Some(sym)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.pairing {
            FuncRodataPairing::SingleFunction { .. } => (0, Some(0)),
            FuncRodataPairing::SingleRodata { .. } => {
                let remaining = 1 - self.rodata_index;

                (remaining, Some(remaining))
            }
            FuncRodataPairing::Pairing {
                rodata_syms,
                late_rodata_syms,
                ..
            } => {
                let remaining = (rodata_syms.len() - self.rodata_index)
                    + (late_rodata_syms.len() - self.late_rodata_index);

                (remaining, Some(remaining))
            }
        }
    }
}
