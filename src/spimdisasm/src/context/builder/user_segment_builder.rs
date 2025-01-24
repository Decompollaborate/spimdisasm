/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use alloc::string::{String, ToString};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    addresses::{Size, Vram},
    collections::addended_ordered_map::{AddendedOrderedMap, FindSettings},
    metadata::{GeneratedBy, SymbolMetadata, SymbolType, UserSegmentMetadata},
};

use super::AddUserSegmentSymbolError;

#[derive(Debug, Clone, Hash, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(module = "spimdisasm"))]
pub struct UserSegmentBuilder {
    symbols: AddendedOrderedMap<Vram, SymbolMetadata>,
}

impl UserSegmentBuilder {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            symbols: AddendedOrderedMap::new(),
        }
    }

    fn add_symbol_impl(
        &mut self,
        vram: Vram,
        size: Size,
        name: Option<String>,
        typ: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSegmentSymbolError> {
        let generated_by = GeneratedBy::UserDeclared;
        let (metadata, newly_created) =
            self.symbols
                .find_mut_or_insert_with(vram, FindSettings::new(true), || {
                    (vram, SymbolMetadata::new(generated_by, vram))
                });

        if metadata.vram() != vram {
            return Err(AddUserSegmentSymbolError::new_overlap(
                vram,
                name,
                size,
                metadata.vram(),
                metadata.user_declared_name().map(|x| x.to_string()),
                metadata
                    .user_declared_size()
                    .expect("Should have size since it is required for this kind of segment."),
            ));
        }

        if !newly_created {
            return Err(AddUserSegmentSymbolError::new_duplicated(
                vram,
                name,
                size,
                metadata.vram(),
                metadata.user_declared_name().map(|x| x.to_string()),
                metadata
                    .user_declared_size()
                    .expect("Should have size since it is required for this kind of segment."),
            ));
        }

        metadata.set_defined();
        *metadata.user_declared_size_mut() = Some(size);

        if let Some(name) = name {
            *metadata.user_declared_name_mut() = Some(name);
        }
        if let Some(typ) = typ {
            metadata.set_type(typ, generated_by);
        }

        Ok(metadata)
    }

    pub fn add_user_symbol(
        &mut self,
        vram: Vram,
        name: String,
        size: Size,
        typ: Option<SymbolType>,
    ) -> Result<&mut SymbolMetadata, AddUserSegmentSymbolError> {
        self.add_symbol_impl(vram, size, Some(name), typ)
    }

    fn add_symbols(
        &mut self,
        syms: &[(Vram, SymbolType, Size, &str)],
        set_names: bool,
    ) -> Result<(), AddUserSegmentSymbolError> {
        for (vram, typ, size, name) in syms {
            self.add_symbol_impl(
                *vram,
                *size,
                set_names.then(|| name.to_string()),
                Some(*typ),
            )?;
        }

        Ok(())
    }

    pub fn n64_libultra_symbols(&mut self) -> Result<(), AddUserSegmentSymbolError> {
        #[rustfmt::skip]
        const SYMS: [(Vram, SymbolType, Size, &str); 9] = [
            (Vram::new(0x800001A0), SymbolType::Word, Size::new(0x4),  "leoBootID"),
            (Vram::new(0x80000300), SymbolType::Word, Size::new(0x4),  "osTvType"),
            (Vram::new(0x80000304), SymbolType::Word, Size::new(0x4),  "osRomType"),
            (Vram::new(0x80000308), SymbolType::Word, Size::new(0x4),  "osRomBase"),
            (Vram::new(0x8000030C), SymbolType::Word, Size::new(0x4),  "osResetType"),
            (Vram::new(0x80000310), SymbolType::Word, Size::new(0x4),  "osCicId"),
            (Vram::new(0x80000314), SymbolType::Word, Size::new(0x4),  "osVersion"),
            (Vram::new(0x80000318), SymbolType::Word, Size::new(0x4),  "osMemSize"),
            (Vram::new(0x8000031C), SymbolType::Byte, Size::new(0x40), "osAppNMIBuffer"),
        ];

        self.add_symbols(&SYMS, true)
    }

    pub fn n64_hardware_registers(
        &mut self,
        set_names: bool,
        _set_as_constants: bool,
    ) -> Result<(), AddUserSegmentSymbolError> {
        #[rustfmt::skip]
        const SYMS: [(Vram, SymbolType, Size, &str); 127] = [
            // Signal Processor Registers
            (Vram::new(0xA4040000), SymbolType::Word, Size::new(0x4), "SP_MEM_ADDR_REG"),
            (Vram::new(0xA4040004), SymbolType::Word, Size::new(0x4), "SP_DRAM_ADDR_REG"),
            (Vram::new(0xA4040008), SymbolType::Word, Size::new(0x4), "SP_RD_LEN_REG"),
            (Vram::new(0xA404000C), SymbolType::Word, Size::new(0x4), "SP_WR_LEN_REG"),
            (Vram::new(0xA4040010), SymbolType::Word, Size::new(0x4), "SP_STATUS_REG"),
            (Vram::new(0xA4040014), SymbolType::Word, Size::new(0x4), "SP_DMA_FULL_REG"),
            (Vram::new(0xA4040018), SymbolType::Word, Size::new(0x4), "SP_DMA_BUSY_REG"),
            (Vram::new(0xA404001C), SymbolType::Word, Size::new(0x4), "SP_SEMAPHORE_REG"),

            (Vram::new(0xA4080000), SymbolType::Word, Size::new(0x4), "SP_PC"),

            // Display Processor Command Registers / Rasterizer Interface
            (Vram::new(0xA4100000), SymbolType::Word, Size::new(0x4), "DPC_START_REG"),
            (Vram::new(0xA4100004), SymbolType::Word, Size::new(0x4), "DPC_END_REG"),
            (Vram::new(0xA4100008), SymbolType::Word, Size::new(0x4), "DPC_CURRENT_REG"),
            (Vram::new(0xA410000C), SymbolType::Word, Size::new(0x4), "DPC_STATUS_REG"),
            (Vram::new(0xA4100010), SymbolType::Word, Size::new(0x4), "DPC_CLOCK_REG"),
            (Vram::new(0xA4100014), SymbolType::Word, Size::new(0x4), "DPC_BUFBUSY_REG"),
            (Vram::new(0xA4100018), SymbolType::Word, Size::new(0x4), "DPC_PIPEBUSY_REG"),
            (Vram::new(0xA410001C), SymbolType::Word, Size::new(0x4), "DPC_TMEM_REG"),

            // Display Processor Span Registers
            (Vram::new(0xA4200000), SymbolType::Word, Size::new(0x4), "DPS_TBIST_REG"), // DPS_TBIST_REG / DP_TMEM_BIST
            (Vram::new(0xA4200004), SymbolType::Word, Size::new(0x4), "DPS_TEST_MODE_REG"),
            (Vram::new(0xA4200008), SymbolType::Word, Size::new(0x4), "DPS_BUFTEST_ADDR_REG"),
            (Vram::new(0xA420000C), SymbolType::Word, Size::new(0x4), "DPS_BUFTEST_DATA_REG"),

            // MIPS Interface Registers
            (Vram::new(0xA4300000), SymbolType::Word, Size::new(0x4), "MI_MODE_REG"), // MI_MODE_REG / MI_INIT_MODE_REG
            (Vram::new(0xA4300004), SymbolType::Word, Size::new(0x4), "MI_VERSION_REG"),
            (Vram::new(0xA4300008), SymbolType::Word, Size::new(0x4), "MI_INTR_REG"),
            (Vram::new(0xA430000C), SymbolType::Word, Size::new(0x4), "MI_INTR_MASK_REG"),
            (Vram::new(0xA4300010), SymbolType::Word, Size::new(0x4), "D_A4300010"),
            (Vram::new(0xA4300014), SymbolType::Word, Size::new(0x4), "MI_SK_EXCEPTION_REG"),
            (Vram::new(0xA4300018), SymbolType::Word, Size::new(0x4), "MI_SK_WATCHDOG_TIMER"),
            (Vram::new(0xA4300028), SymbolType::Word, Size::new(0x4), "D_A4300028"),
            (Vram::new(0xA430002C), SymbolType::Word, Size::new(0x4), "MI_RANDOM_BIT"),
            (Vram::new(0xA4300030), SymbolType::Word, Size::new(0x4), "D_A4300030"),
            (Vram::new(0xA4300038), SymbolType::Word, Size::new(0x4), "MI_HW_INTR_REG"),
            (Vram::new(0xA430003C), SymbolType::Word, Size::new(0x4), "MI_HW_INTR_MASK_REG"),

            // Video Interface Registers
            (Vram::new(0xA4400000), SymbolType::Word, Size::new(0x4), "VI_STATUS_REG"), // VI_STATUS_REG / VI_CONTROL_REG
            (Vram::new(0xA4400004), SymbolType::Word, Size::new(0x4), "VI_DRAM_ADDR_REG"), // VI_DRAM_ADDR_REG / VI_ORIGIN_REG
            (Vram::new(0xA4400008), SymbolType::Word, Size::new(0x4), "VI_WIDTH_REG"),
            (Vram::new(0xA440000C), SymbolType::Word, Size::new(0x4), "VI_INTR_REG"),
            (Vram::new(0xA4400010), SymbolType::Word, Size::new(0x4), "VI_CURRENT_REG"),
            (Vram::new(0xA4400014), SymbolType::Word, Size::new(0x4), "VI_BURST_REG"), // VI_BURST_REG / VI_TIMING_REG
            (Vram::new(0xA4400018), SymbolType::Word, Size::new(0x4), "VI_V_SYNC_REG"),
            (Vram::new(0xA440001C), SymbolType::Word, Size::new(0x4), "VI_H_SYNC_REG"),
            (Vram::new(0xA4400020), SymbolType::Word, Size::new(0x4), "VI_LEAP_REG"),
            (Vram::new(0xA4400024), SymbolType::Word, Size::new(0x4), "VI_H_START_REG"),
            (Vram::new(0xA4400028), SymbolType::Word, Size::new(0x4), "VI_V_START_REG"),
            (Vram::new(0xA440002C), SymbolType::Word, Size::new(0x4), "VI_V_BURST_REG"),
            (Vram::new(0xA4400030), SymbolType::Word, Size::new(0x4), "VI_X_SCALE_REG"),
            (Vram::new(0xA4400034), SymbolType::Word, Size::new(0x4), "VI_Y_SCALE_REG"),

            // Audio Interface Registers
            (Vram::new(0xA4500000), SymbolType::Word, Size::new(0x4), "AI_DRAM_ADDR_REG"),
            (Vram::new(0xA4500004), SymbolType::Word, Size::new(0x4), "AI_LEN_REG"),
            (Vram::new(0xA4500008), SymbolType::Word, Size::new(0x4), "AI_CONTROL_REG"),
            (Vram::new(0xA450000C), SymbolType::Word, Size::new(0x4), "AI_STATUS_REG"),
            (Vram::new(0xA4500010), SymbolType::Word, Size::new(0x4), "AI_DACRATE_REG"),
            (Vram::new(0xA4500014), SymbolType::Word, Size::new(0x4), "AI_BITRATE_REG"),

            // Peripheral/Parallel Interface Registers
            (Vram::new(0xA4600000), SymbolType::Word, Size::new(0x4), "PI_DRAM_ADDR_REG"),
            (Vram::new(0xA4600004), SymbolType::Word, Size::new(0x4), "PI_CART_ADDR_REG"),
            (Vram::new(0xA4600008), SymbolType::Word, Size::new(0x4), "PI_RD_LEN_REG"),
            (Vram::new(0xA460000C), SymbolType::Word, Size::new(0x4), "PI_WR_LEN_REG"),
            (Vram::new(0xA4600010), SymbolType::Word, Size::new(0x4), "PI_STATUS_REG"),
            (Vram::new(0xA4600014), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM1_LAT_REG"), // PI dom1 latency
            (Vram::new(0xA4600018), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM1_PWD_REG"), // PI dom1 pulse width
            (Vram::new(0xA460001C), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM1_PGS_REG"), // PI dom1 page size
            (Vram::new(0xA4600020), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM1_RLS_REG"), // PI dom1 release
            (Vram::new(0xA4600024), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM2_LAT_REG"), // PI dom2 latency
            (Vram::new(0xA4600028), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM2_LWD_REG"), // PI dom2 pulse width
            (Vram::new(0xA460002C), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM2_PGS_REG"), // PI dom2 page size
            (Vram::new(0xA4600030), SymbolType::Word, Size::new(0x4), "PI_BSD_DOM2_RLS_REG"), // PI dom2 release
            (Vram::new(0xA4600038), SymbolType::Word, Size::new(0x4), "PI_CARD_STATUS_REG"),
            (Vram::new(0xA4600040), SymbolType::Word, Size::new(0x4), "PI_ATB_NEXT_CONFIG"),
            (Vram::new(0xA4600044), SymbolType::Word, Size::new(0x4), "D_A4600044"),
            (Vram::new(0xA4600048), SymbolType::Word, Size::new(0x4), "PI_CARD_CNT_REG"),
            (Vram::new(0xA460004C), SymbolType::Word, Size::new(0x4), "PI_CARD_CONFIG_REG"),
            (Vram::new(0xA4600050), SymbolType::Word, Size::new(0x4), "PI_AES_CNT"),
            (Vram::new(0xA4600054), SymbolType::Word, Size::new(0x4), "PI_ALLOWED_IO"),
            (Vram::new(0xA4600058), SymbolType::Word, Size::new(0x4), "PI_EX_RD_LEN_REG"),
            (Vram::new(0xA460005C), SymbolType::Word, Size::new(0x4), "PI_EX_WR_LEN_REG"),
            (Vram::new(0xA4600060), SymbolType::Word, Size::new(0x4), "PI_MISC_REG"),
            (Vram::new(0xA4600064), SymbolType::Word, Size::new(0x4), "D_A4600064"),
            (Vram::new(0xA4600070), SymbolType::Word, Size::new(0x4), "PI_CARD_BLK_OFFSET_REG"),
            (Vram::new(0xA4610000), SymbolType::Word, Size::new(0x4), "PI_EX_DMA_BUF"),
            (Vram::new(0xA4610420), SymbolType::Word, Size::new(0x4), "PI_AES_EXPANDED_KEY"),
            (Vram::new(0xA46104D0), SymbolType::Word, Size::new(0x4), "PI_AES_IV"),
            (Vram::new(0xA4610500), SymbolType::Word, Size::new(0x4), "PI_ATB_ENTRY"),
            (Vram::new(0xA4620000), SymbolType::Word, Size::new(0x4), "D_A4620000"),
            (Vram::new(0xA46E0000), SymbolType::Short, Size::new(0x2), "PI_RDB_REQ_HI_REG"),
            (Vram::new(0xA46E0002), SymbolType::Short, Size::new(0x2), "PI_RDB_REQ_LO_REG"),
            (Vram::new(0xA46E0004), SymbolType::Word, Size::new(0x4), "D_A46E0004"),
            (Vram::new(0xA46E0400), SymbolType::Short, Size::new(0x2), "D_A46E0400"),
            (Vram::new(0xA46E0402), SymbolType::Short, Size::new(0x2), "D_A46E0402"),
            (Vram::new(0xA46E8000), SymbolType::Word, Size::new(0x4), "PI_RDB_STATUS_REG"),

            // RDRAM Interface Registers
            (Vram::new(0xA4700000), SymbolType::Word, Size::new(0x4), "RI_MODE_REG"),
            (Vram::new(0xA4700004), SymbolType::Word, Size::new(0x4), "RI_CONFIG_REG"),
            (Vram::new(0xA4700008), SymbolType::Word, Size::new(0x4), "RI_CURRENT_LOAD_REG"),
            (Vram::new(0xA470000C), SymbolType::Word, Size::new(0x4), "RI_SELECT_REG"),
            (Vram::new(0xA4700010), SymbolType::Word, Size::new(0x4), "RI_REFRESH_REG"),
            (Vram::new(0xA4700014), SymbolType::Word, Size::new(0x4), "RI_LATENCY_REG"),
            (Vram::new(0xA4700018), SymbolType::Word, Size::new(0x4), "RI_RERROR_REG"),
            (Vram::new(0xA470001C), SymbolType::Word, Size::new(0x4), "RI_WERROR_REG"),

            // Serial Interface Registers
            (Vram::new(0xA4800000), SymbolType::Word, Size::new(0x4), "SI_DRAM_ADDR_REG"),
            (Vram::new(0xA4800004), SymbolType::Word, Size::new(0x4), "SI_PIF_ADDR_RD64B_REG"),
            (Vram::new(0xA4800008), SymbolType::Word, Size::new(0x4), "D_A4800008"), // reserved
            (Vram::new(0xA480000C), SymbolType::Word, Size::new(0x4), "D_A480000C"), // reserved
            (Vram::new(0xA4800010), SymbolType::Word, Size::new(0x4), "SI_PIF_ADDR_WR64B_REG"),
            (Vram::new(0xA4800014), SymbolType::Word, Size::new(0x4), "D_A4800014"), // reserved
            (Vram::new(0xA4800018), SymbolType::Word, Size::new(0x4), "SI_STATUS_REG"),
            (Vram::new(0xA480001C), SymbolType::Word, Size::new(0x4), "D_A480001C"),

            // libleo (64DD) address range
            (Vram::new(0xA5000508), SymbolType::Word, Size::new(0x4), "LEO_CMD"),
            // (Vram::new(0xA5000508), SymbolType::Word, Size::new(0x4), "LEO_STATUS"),

            (Vram::new(0xA5000510), SymbolType::Word, Size::new(0x4), "LEO_BM_CTL"),
            // (Vram::new(0xA5000510), SymbolType::Word, Size::new(0x4), "LEO_BM_STATUS"),

            (Vram::new(0xA5000518), SymbolType::Word, Size::new(0x4), "LEO_SEQ_CTL"),
            // (Vram::new(0xA5000518), SymbolType::Word, Size::new(0x4), "LEO_SEQ_STATUS"),

            (Vram::new(0xA5000000), SymbolType::Word, Size::new(0x4), "LEO_C2_BUFF"),      // C2 Sector Buffer
            (Vram::new(0xA5000400), SymbolType::Word, Size::new(0x4), "LEO_SECTOR_BUFF"),  // Data Sector Buffer
            (Vram::new(0xA5000500), SymbolType::Word, Size::new(0x4), "LEO_DATA"),         // Data
            (Vram::new(0xA5000504), SymbolType::Word, Size::new(0x4), "LEO_MISC_REG"),     // Misc Register

            (Vram::new(0xA500050C), SymbolType::Word, Size::new(0x4), "LEO_CUR_TK"),       // Current Track

            (Vram::new(0xA5000514), SymbolType::Word, Size::new(0x4), "LEO_ERR_SECTOR"),   // Sector Error Status

            (Vram::new(0xA500051C), SymbolType::Word, Size::new(0x4), "LEO_CUR_SECTOR"),   // Current Sector
            (Vram::new(0xA5000520), SymbolType::Word, Size::new(0x4), "LEO_HARD_RESET"),   // Hard Reset
            (Vram::new(0xA5000524), SymbolType::Word, Size::new(0x4), "LEO_C1_S0"),        // C1
            (Vram::new(0xA5000528), SymbolType::Word, Size::new(0x4), "LEO_HOST_SECBYTE"), // Sector Size (in bytes)
            (Vram::new(0xA500052C), SymbolType::Word, Size::new(0x4), "LEO_C1_S2"),        // C1
            (Vram::new(0xA5000530), SymbolType::Word, Size::new(0x4), "LEO_SEC_BYTE"),     // Sectors per Block, Full Size
            (Vram::new(0xA5000534), SymbolType::Word, Size::new(0x4), "LEO_C1_S4"),        // C1
            (Vram::new(0xA5000538), SymbolType::Word, Size::new(0x4), "LEO_C1_S6"),        // C1
            (Vram::new(0xA500053C), SymbolType::Word, Size::new(0x4), "LEO_CUR_ADDR"),     // Current Address?
            (Vram::new(0xA5000540), SymbolType::Word, Size::new(0x4), "LEO_ID_REG"),       // ID
            (Vram::new(0xA5000544), SymbolType::Word, Size::new(0x4), "LEO_TEST_REG"),     // Test Read
            (Vram::new(0xA5000548), SymbolType::Word, Size::new(0x4), "LEO_TEST_PIN_SEL"), // Test Write
            (Vram::new(0xA5000580), SymbolType::Word, Size::new(0x4), "LEO_RAM_ADDR"),     // Microsequencer RAM
        ];

        self.add_symbols(&SYMS, set_names)?;

        Ok(())
    }

    pub fn ique_libultra_symbols(&mut self) -> Result<(), AddUserSegmentSymbolError> {
        #[rustfmt::skip]
        const SYMS: [(Vram, SymbolType, Size, &str); 15] = [
            (Vram::new(0x8000035C), SymbolType::Word,    Size::new(0x4),     "__osBbEepromAddress"),
            (Vram::new(0x80000360), SymbolType::Word,    Size::new(0x4),     "__osBbEepromSize"),
            (Vram::new(0x80000364), SymbolType::Word,    Size::new(0x4),     "__osBbFlashAddress"),
            (Vram::new(0x80000368), SymbolType::Word,    Size::new(0x4),     "__osBbFlashSize"),
            (Vram::new(0x8000036C), SymbolType::Word,    Size::new(0x4),     "__osBbSramAddress"),
            (Vram::new(0x80000370), SymbolType::Word,    Size::new(0x4),     "__osBbSramSize"),
            (Vram::new(0x80000374), SymbolType::Word,    Size::new(0x4 * 4), "__osBbPakAddress"),
            (Vram::new(0x80000384), SymbolType::Word,    Size::new(0x4),     "__osBbPakSize"),
            (Vram::new(0x80000388), SymbolType::Word,    Size::new(0x4),     "__osBbIsBb"),
            (Vram::new(0x8000038C), SymbolType::Word,    Size::new(0x4),     "__osBbHackFlags"),
            (Vram::new(0x80000390), SymbolType::Word,    Size::new(0x4),     "__osBbStashMagic"),
            (Vram::new(0x80000394), SymbolType::Word,    Size::new(0x4 * 4), "__osBbPakBindings"),
            (Vram::new(0x800003A4), SymbolType::CString, Size::new(0x10),    "__osBbStateName"),
            (Vram::new(0x800003B4), SymbolType::Word,    Size::new(0x4),     "__osBbStateDirty"),
            (Vram::new(0x800003B8), SymbolType::Word,    Size::new(0x4),     "__osBbAuxDataLimit"),
        ];

        self.add_symbols(&SYMS, true)
    }

    pub fn ique_hardware_registers(
        &mut self,
        set_names: bool,
        _set_as_constants: bool,
    ) -> Result<(), AddUserSegmentSymbolError> {
        // TODO: fill missing ones
        #[rustfmt::skip]
        const SYMS: [(Vram, SymbolType, Size, &str); 4] = [
            (Vram::new(0xA4900000), SymbolType::Word, Size::new(0x4), "D_A4900000"),
            (Vram::new(0xA4940010), SymbolType::Word, Size::new(0x4), "USB0_STATUS_REG"),
            (Vram::new(0xA4A00000), SymbolType::Word, Size::new(0x4), "D_A4A00000"),
            (Vram::new(0xA4A40010), SymbolType::Word, Size::new(0x4), "USB1_STATUS_REG"),
        ];

        self.add_symbols(&SYMS, set_names)?;

        Ok(())
    }

    pub(crate) fn build(self) -> UserSegmentMetadata {
        UserSegmentMetadata::new(self.symbols)
    }
}

#[cfg(feature = "pyo3")]
pub(crate) mod python_bindings {
    use super::*;

    #[pymethods]
    impl UserSegmentBuilder {
        #[new]
        fn py_new() -> Self {
            Self::new()
        }

        #[pyo3(name = "add_user_symbol", signature=(vram, name, size, typ))]
        pub fn py_add_symbol(
            &mut self,
            vram: Vram,
            name: String,
            size: Size,
            typ: Option<SymbolType>,
        ) -> Result<(), AddUserSegmentSymbolError> {
            self.add_user_symbol(vram, name, size, typ)?;
            Ok(())
        }

        #[pyo3(name = "n64_libultra_symbols")]
        pub fn py_n64_libultra_symbols(&mut self) -> Result<(), AddUserSegmentSymbolError> {
            self.n64_libultra_symbols()
        }

        #[pyo3(name = "n64_hardware_registers")]
        pub fn py_n64_hardware_registers(
            &mut self,
            set_names: bool,
            set_as_constants: bool,
        ) -> Result<(), AddUserSegmentSymbolError> {
            self.n64_hardware_registers(set_names, set_as_constants)
        }

        #[pyo3(name = "ique_libultra_symbols")]
        pub fn py_ique_libultra_symbols(&mut self) -> Result<(), AddUserSegmentSymbolError> {
            self.ique_libultra_symbols()
        }

        #[pyo3(name = "ique_hardware_registers")]
        pub fn py_ique_hardware_registers(
            &mut self,
            set_names: bool,
            set_as_constants: bool,
        ) -> Result<(), AddUserSegmentSymbolError> {
            self.ique_hardware_registers(set_names, set_as_constants)
        }
    }
}
