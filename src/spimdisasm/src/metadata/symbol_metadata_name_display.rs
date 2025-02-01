/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use core::fmt;

use crate::section_type::SectionType;

use super::{OwnerSegmentKind, SymbolMetadata, SymbolType};

fn should_escape_symbol(name: &str) -> bool {
    name.contains('@')
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd)]
#[must_use]
pub struct SymbolMetadataNameDisplay<'sym> {
    sym: &'sym SymbolMetadata,
}

impl<'sym> SymbolMetadataNameDisplay<'sym> {
    pub fn new(sym: &'sym SymbolMetadata) -> Self {
        Self { sym }
    }
}

impl SymbolMetadataNameDisplay<'_> {
    fn display_section_prefix(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sym.sym_type() {
            Some(SymbolType::Function)
            | Some(SymbolType::BranchLabel)
            | Some(SymbolType::JumptableLabel)
            | Some(SymbolType::Jumptable)
            | Some(SymbolType::GccExceptTable)
            | Some(SymbolType::GccExceptTableLabel) => {
                // Functions, labels and jumptables don't get a section prefix because most of the time they are in
                // their respective sections.
                // But if the section type is missing, then we haven't seen this symbol actually defined anywhere, so
                // we tell the user by using this prefix.
                match self.sym.section_type() {
                    Some(_) => Ok(()),
                    None => write!(f, "UNK_"),
                }
            }
            _ => {
                match self.sym.section_type() {
                    Some(SectionType::Text) => write!(f, "T_"),
                    Some(SectionType::Data) => write!(f, "D_"),
                    // TODO: rename to R_
                    Some(SectionType::Rodata) => write!(f, "RO_"),
                    Some(SectionType::Bss) => write!(f, "B_"),
                    // TODO: rename to ET_
                    Some(SectionType::GccExceptTable) => write!(f, "EHTBL_"),
                    None => write!(f, "UNK_"),
                }
            }
        }
    }

    fn display_type_prefix(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.sym.sym_type() {
            Some(SymbolType::Function) => write!(f, "func_"),
            Some(SymbolType::BranchLabel) | Some(SymbolType::JumptableLabel) => write!(f, ".L"),
            Some(SymbolType::Jumptable) => write!(f, "jtbl_"),
            Some(SymbolType::GccExceptTable) => write!(f, "ehtbl_"),
            Some(SymbolType::GccExceptTableLabel) => write!(f, "$LEH_"),

            Some(SymbolType::Byte) => Ok(()),
            Some(SymbolType::Short) => Ok(()),
            Some(SymbolType::Word) => Ok(()),
            Some(SymbolType::DWord) => Ok(()),

            // GlobalConfig.AUTOGENERATED_NAMES_BASED_ON_DATA_TYPE
            Some(SymbolType::Float32) => {
                if false {
                    write!(f, "FLT_")
                } else {
                    Ok(())
                }
            }
            Some(SymbolType::Float64) => {
                if false {
                    write!(f, "DBL_")
                } else {
                    Ok(())
                }
            }
            Some(SymbolType::CString) => {
                if false {
                    write!(f, "STR_")
                } else {
                    Ok(())
                }
            }

            Some(SymbolType::UserCustom) => Ok(()),

            None => Ok(()),
        }
    }

    fn display_suffix(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let OwnerSegmentKind::Overlay(name) = self.sym.owner_segment_kind() {
            write!(f, "_{}", name)?;
        }

        /*
        if GlobalConfig.CUSTOM_SUFFIX:
            suffix += GlobalConfig.CUSTOM_SUFFIX
        */

        Ok(())
    }

    fn display_unique_identifier(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /*
        if GlobalConfig.SEQUENTIAL_LABEL_NAMES and self.parentFunction is not None:
            if symType in {SymbolSpecialType.branchlabel, SymbolSpecialType.jumptablelabel}:
                index = self.parentFunction.branchLabels.index(self.vram)
                if index is not None:
                    return f"{self.parentFunction.getName()}_{index + 1}"
            elif symType == SymbolSpecialType.jumptable:
                index = self.parentFunction.jumpTables.index(self.vram)
                if index is not None:
                    return f"{self.parentFunction.getName()}_{index + 1}"

        if GlobalConfig.AUTOGENERATED_NAMES_BASED_ON_FILE_NAME:
            if self.parentFileName is not None and self.inFileOffset is not None and symType != SymbolSpecialType.function:
                sectionName = self.sectionType.toStr().replace(".", "_")
                return f"{self.parentFileName}{sectionName}_{self.inFileOffset:06X}"
        */

        // TODO: logic for types and such
        write!(f, "{}", self.sym.vram())?;

        self.display_suffix(f)
    }

    pub fn autogenerate_name(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.display_section_prefix(f)?;
        self.display_type_prefix(f)?;
        self.display_unique_identifier(f)
    }
}

impl fmt::Display for SymbolMetadataNameDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(user_declared_name) = &self.sym.user_declared_name() {
            let should_escape = should_escape_symbol(user_declared_name);

            if should_escape {
                write!(f, "\"")?;
            }
            write!(f, "{}", user_declared_name)?;
            if should_escape {
                write!(f, "\"")?;
            }
            Ok(())
        } else {
            self.autogenerate_name(f)
        }
    }
}
