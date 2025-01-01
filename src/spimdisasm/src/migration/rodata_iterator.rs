/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use crate::{sections::SectionData, symbols::SymbolData};

use super::FuncRodataPairing;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct RodataIterator<'pairing, 'rodata> {
    pairing: &'pairing FuncRodataPairing,
    rodata_section: Option<&'rodata SectionData>,
    rodata_index: usize,
    late_rodata_index: usize,
}

impl<'pairing, 'rodata> RodataIterator<'pairing, 'rodata> {
    pub(super) fn new(
        pairing: &'pairing FuncRodataPairing,
        rodata_section: Option<&'rodata SectionData>,
    ) -> Self {
        Self {
            pairing,
            rodata_section,
            rodata_index: 0,
            late_rodata_index: 0,
        }
    }
}

impl<'rodata> Iterator for RodataIterator<'_, 'rodata> {
    type Item = &'rodata SymbolData;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.pairing {
            FuncRodataPairing::SingleRodata { rodata_index } => {
                if let Some(rodata_section) = self.rodata_section {
                    if self.rodata_index == 0 {
                        self.rodata_index += 1;
                        rodata_section.data_symbols().get(*rodata_index)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            FuncRodataPairing::Pairing {
                rodata_indices,
                late_rodata_indices,
                ..
            } => {
                if let Some(rodata_section) = self.rodata_section {
                    let (rodata_index, index) = if self.rodata_index < rodata_indices.len() {
                        (&rodata_indices[self.rodata_index], &mut self.rodata_index)
                    } else if self.late_rodata_index < late_rodata_indices.len() {
                        (
                            &late_rodata_indices[self.late_rodata_index],
                            &mut self.late_rodata_index,
                        )
                    } else {
                        return None;
                    };

                    *index += 1;
                    rodata_section.data_symbols().get(*rodata_index)
                } else {
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.pairing {
            FuncRodataPairing::SingleRodata { .. } => {
                let remaining = 1 - self.rodata_index;

                (remaining, Some(remaining))
            }
            FuncRodataPairing::Pairing {
                rodata_indices,
                late_rodata_indices,
                ..
            } => {
                let remaining = (rodata_indices.len() - self.rodata_index)
                    + (late_rodata_indices.len() - self.late_rodata_index);

                (remaining, Some(remaining))
            }
        }
    }
}
