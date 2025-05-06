/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::BTreeMap;

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion};
use spimdisasm::{
    addresses::{
        AddressRange, GlobalOffsetTable, GotGlobalEntry, GotLocalEntry, GpValue, Rom, RomVramRange,
        Size, Vram,
    },
    config::{Endian, GlobalConfigBuilder, GpConfig},
    context::{builder::UserSegmentBuilder, Context, ContextBuilder, GlobalSegmentBuilder},
    metadata::SegmentMetadata,
    parent_segment_info::ParentSegmentInfo,
    sections::{
        before_proc::{DataSectionSettings, ExecutableSectionSettings, NobitsSectionSettings},
        processed::{DataSectionProcessed, ExecutableSectionProcessed, NobitsSectionProcessed},
    },
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNobitsDisplaySettings},
};

#[must_use]
struct RawSectionInfo<'a> {
    bytes: &'a [u8],
    rom: Rom,
    vram: Vram,
}
impl<'a> RawSectionInfo<'a> {
    pub fn new(bytes: &'a [u8], rom: Rom, vram: Vram) -> Self {
        Self { bytes, rom, vram }
    }

    pub fn ranges(&self) -> RomVramRange {
        RomVramRange::new(
            AddressRange::new(self.rom, self.rom + Size::new(self.bytes.len() as u32)),
            AddressRange::new(self.vram, self.vram + Size::new(self.bytes.len() as u32)),
        )
    }
}

#[must_use]
struct RawNobitsSectionInfo {
    size: Size,
    vram: Vram,
}
impl RawNobitsSectionInfo {
    pub fn new(size: Size, vram: Vram) -> Self {
        Self { size, vram }
    }

    pub fn ranges(&self) -> RomVramRange {
        RomVramRange::new(
            AddressRange::new(Rom::new(0), Rom::new(0)),
            AddressRange::new(self.vram, self.vram + self.size),
        )
    }
}

struct Sections {
    context: Context,
    text: Option<ExecutableSectionProcessed>,
    data: Option<DataSectionProcessed>,
    rodata: Option<DataSectionProcessed>,
    gcc_except_table: Option<DataSectionProcessed>,
    bss: Option<NobitsSectionProcessed>,
}
impl Sections {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        endian: Endian,
        text_info: (RawSectionInfo, ExecutableSectionSettings),
        data_info: (RawSectionInfo, DataSectionSettings),
        rodata_info: (RawSectionInfo, DataSectionSettings),
        gcc_except_table_info: (RawSectionInfo, DataSectionSettings),
        bss_info: (RawNobitsSectionInfo, NobitsSectionSettings),
        gp_config: Option<GpConfig>,
        global_offset_table: Option<GlobalOffsetTable>,
    ) -> Self {
        let mut global_ranges = text_info.0.ranges();

        global_ranges.expand_ranges(&data_info.0.ranges());
        global_ranges.expand_ranges(&rodata_info.0.ranges());
        global_ranges.expand_ranges(&gcc_except_table_info.0.ranges());
        global_ranges.expand_ranges(&bss_info.0.ranges());

        let global_config = GlobalConfigBuilder::new(endian)
            .with_gp_config(gp_config)
            .build();
        let mut context = {
            let mut global_builder = GlobalSegmentBuilder::new(global_ranges);
            if let Some(global_offset_table) = global_offset_table {
                global_builder
                    .add_global_offset_table(&global_config, global_offset_table)
                    .unwrap();
            }
            let mut global_heater = global_builder.finish_symbols();

            if !text_info.0.bytes.is_empty() {
                global_heater
                    .preheat_text(
                        &global_config,
                        &text_info.1,
                        ".text",
                        text_info.0.bytes,
                        text_info.0.rom,
                        text_info.0.vram,
                    )
                    .unwrap();
            }
            if !data_info.0.bytes.is_empty() {
                global_heater
                    .preheat_data(
                        &global_config,
                        &data_info.1,
                        ".data",
                        data_info.0.bytes,
                        data_info.0.rom,
                        data_info.0.vram,
                    )
                    .unwrap();
            }
            if !rodata_info.0.bytes.is_empty() {
                global_heater
                    .preheat_rodata(
                        &global_config,
                        &rodata_info.1,
                        ".rodata",
                        rodata_info.0.bytes,
                        rodata_info.0.rom,
                        rodata_info.0.vram,
                    )
                    .unwrap();
            }
            if !gcc_except_table_info.0.bytes.is_empty() {
                global_heater
                    .preheat_gcc_except_table(
                        &global_config,
                        &gcc_except_table_info.1,
                        ".except_table",
                        gcc_except_table_info.0.bytes,
                        gcc_except_table_info.0.rom,
                        gcc_except_table_info.0.vram,
                    )
                    .unwrap();
            }

            let mut platform_segment = UserSegmentBuilder::new();
            platform_segment.n64_libultra_symbols().unwrap();
            platform_segment.n64_hardware_registers(true, true).unwrap();

            let builder = ContextBuilder::new(global_heater, platform_segment);
            builder.build(global_config).unwrap()
        };

        let parent_info = ParentSegmentInfo::new(
            global_ranges.rom().start(),
            global_ranges.vram().start(),
            None,
        );

        let text = (!text_info.0.bytes.is_empty()).then(|| {
            context
                .create_section_text(
                    &text_info.1,
                    ".text",
                    text_info.0.bytes,
                    text_info.0.rom,
                    text_info.0.vram,
                    parent_info.clone(),
                )
                .unwrap()
        });
        let data = (!data_info.0.bytes.is_empty()).then(|| {
            context
                .create_section_data(
                    &data_info.1,
                    ".data",
                    data_info.0.bytes,
                    data_info.0.rom,
                    data_info.0.vram,
                    parent_info.clone(),
                )
                .unwrap()
        });
        let rodata = (!rodata_info.0.bytes.is_empty()).then(|| {
            context
                .create_section_rodata(
                    &rodata_info.1,
                    ".rodata",
                    rodata_info.0.bytes,
                    rodata_info.0.rom,
                    rodata_info.0.vram,
                    parent_info.clone(),
                )
                .unwrap()
        });
        let gcc_except_table = (!gcc_except_table_info.0.bytes.is_empty()).then(|| {
            context
                .create_section_gcc_except_table(
                    &gcc_except_table_info.1,
                    ".except_table",
                    gcc_except_table_info.0.bytes,
                    gcc_except_table_info.0.rom,
                    gcc_except_table_info.0.vram,
                    parent_info.clone(),
                )
                .unwrap()
        });
        let bss = (!bss_info.0.size.is_empty()).then(|| {
            context
                .create_section_bss(
                    &bss_info.1,
                    ".bss",
                    *bss_info.0.ranges().vram(),
                    parent_info,
                )
                .unwrap()
        });

        let user_relocs = BTreeMap::new();
        let text = text.map(|x| x.post_process(&mut context, &user_relocs).unwrap());
        let data = data.map(|x| x.post_process(&mut context, &user_relocs).unwrap());
        let rodata = rodata.map(|x| x.post_process(&mut context, &user_relocs).unwrap());
        let gcc_except_table =
            gcc_except_table.map(|x| x.post_process(&mut context, &user_relocs).unwrap());
        let bss = bss.map(|x| x.post_process(&mut context).unwrap());

        Self {
            context,
            text,
            data,
            rodata,
            gcc_except_table,
            bss,
        }
    }

    pub fn display_to_string(
        &self,
        text_display_settings: &FunctionDisplaySettings,
        data_display_settings: &SymDataDisplaySettings,
        rodata_display_settings: &SymDataDisplaySettings,
        gcc_except_table_display_settings: &SymDataDisplaySettings,
        bss_display_settings: &SymNobitsDisplaySettings,
    ) -> String {
        let mut out = String::new();

        if let Some(section) = &self.text {
            out.push_str(".section .text\n");
            for sym in section.functions() {
                out.push('\n');
                out.push_str(
                    &sym.display(&self.context, text_display_settings)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        if let Some(section) = &self.data {
            out.push_str("\n.section .data\n");
            for sym in section.data_symbols() {
                out.push('\n');
                out.push_str(
                    &sym.display(&self.context, data_display_settings)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        if let Some(section) = &self.rodata {
            out.push_str("\n.section .rodata\n");
            for sym in section.data_symbols() {
                out.push('\n');
                out.push_str(
                    &sym.display(&self.context, rodata_display_settings)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        if let Some(section) = &self.gcc_except_table {
            out.push_str("\n.section .gcc_except_table\n");
            for sym in section.data_symbols() {
                out.push('\n');
                out.push_str(
                    &sym.display(&self.context, gcc_except_table_display_settings)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        if let Some(section) = &self.bss {
            out.push_str("\n.section .bss\n");
            for sym in section.noload_symbols() {
                out.push('\n');
                out.push_str(
                    &sym.display(&self.context, bss_display_settings)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        out
    }

    pub fn print_global_segment_info(&self) {
        println!();
        println!("Global segment info:");
        print_segment_info(self.context.global_segment());
        println!();
        println!();
    }
}

fn print_segment_info(segment: &SegmentMetadata) {
    println!("Vram range: {:?}", segment.vram_range());
    println!("Rom range: {:?}", segment.rom_range());
    println!();

    println!("Symbols:");
    for (vram, metadata) in segment.symbols() {
        println!(
            "    {:?} {} {:?} {:?} {:?} {:?}",
            vram,
            metadata.display_name(),
            metadata.size(),
            metadata.sym_type(),
            metadata.section_type(),
            metadata.is_defined()
        );
    }
}

#[test]
fn test_jumptable_with_lo_in_each_case_for_same_hi() {
    // mips-linux-gnu-gcc -c -Os -mips3 -mno-abicalls -G0 -G 0 -nostdinc -march=vr4300 -mfix4300 -mabi=32 -mno-abicalls -fno-PIC a.c -o a.o
    /*
    struct some_struct {
        int a;
        int b;
        int c;
        int d;
        int e;
        int f;
        int g;
    } asdf;

    int no(int num) {
        switch (num) {
            case 0:
                return asdf.f;
            case 1:
                return asdf.b;
            case 2:
                return asdf.c;
            default:
            case 3:
                return asdf.a;
            case 4:
                return asdf.e;
            case 5:
                return asdf.g;
            case 6:
                return asdf.d;
        }
    }
    void dummy(void) {
    }
    */

    let text_bytes: [u8; 128] = [
        0x2C, 0x83, 0x00, 0x07, // sltiu
        0x10, 0x60, 0x00, 0x11, // beqz
        0x3C, 0x02, 0x80, 0x00, // lui
        0x3C, 0x03, 0x80, 0x00, // lui
        0x24, 0x63, 0x40, 0x80, // addiu
        0x00, 0x04, 0x20, 0x80, // sll
        0x00, 0x64, 0x20, 0x21, // addu
        0x8C, 0x83, 0x00, 0x00, // lw
        0x00, 0x60, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, // nop
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x14, // lw
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x04, // lw
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x08, // lw
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x40, 0xA0, // lw
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x10, // lw
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x18, // lw
        0x24, 0x42, 0x40, 0xA0, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x8C, 0x42, 0x00, 0x0C, // lw
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, // nop
    ];
    let text_rom = Rom::new(0x0);
    let text_vram = Vram::new(0x80004000);

    let data_bytes: [u8; 0] = [];
    let data_rom = Rom::new(0x80);
    let data_vram = Vram::new(0x80004080);

    let rodata_bytes: [u8; 32] = [
        0x80, 0x00, 0x40, 0x28, 0x80, 0x00, 0x40, 0x34, 0x80, 0x00, 0x40, 0x40, 0x80, 0x00, 0x40,
        0x4C, 0x80, 0x00, 0x40, 0x54, 0x80, 0x00, 0x40, 0x60, 0x80, 0x00, 0x40, 0x6C, 0x00, 0x00,
        0x00, 0x00,
    ];
    let rodata_rom = Rom::new(0x80);
    let rodata_vram = Vram::new(0x80004080);

    let gcc_except_table_bytes: [u8; 0] = [];
    let gcc_except_table_rom = Rom::new(0xA0);
    let gcc_except_table_vram = Vram::new(0x800040A0);

    let bss_size = Size::new(0x20);
    let bss_vram = Vram::new(0x800040A0);

    let gp_config = None;
    let global_offset_table = None;

    let executable_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let data_settings = DataSectionSettings::new(None);
    let noload_settings = NobitsSectionSettings::new(None);

    let sections = Sections::new(
        Endian::Big,
        (
            RawSectionInfo::new(&text_bytes, text_rom, text_vram),
            executable_settings,
        ),
        (
            RawSectionInfo::new(&data_bytes, data_rom, data_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(&rodata_bytes, rodata_rom, rodata_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(
                &gcc_except_table_bytes,
                gcc_except_table_rom,
                gcc_except_table_vram,
            ),
            data_settings,
        ),
        (
            RawNobitsSectionInfo::new(bss_size, bss_vram),
            noload_settings,
        ),
        gp_config,
        global_offset_table,
    );

    sections.print_global_segment_info();

    let instr_display_flags = InstructionDisplayFlags::default();
    let text_display_settings = FunctionDisplaySettings::new(instr_display_flags);

    let data_display_settings = SymDataDisplaySettings::new();
    let bss_display_settings = SymNobitsDisplaySettings::new();

    let disassembled_str = sections.display_to_string(
        &text_display_settings,
        &data_display_settings,
        &data_display_settings,
        &data_display_settings,
        &bss_display_settings,
    );

    println!("{}", disassembled_str);

    let expected_str = "\
.section .text

glabel func_80004000
    /* 000000 80004000 2C830007 */  sltiu       $v1, $a0, 0x7
    /* 000004 80004004 10600011 */  beqz        $v1, .L8000404C
    /* 000008 80004008 3C028000 */   lui        $v0, %hi(B_800040A0)
    /* 00000C 8000400C 3C038000 */  lui         $v1, %hi(jtbl_80004080)
    /* 000010 80004010 24634080 */  addiu       $v1, $v1, %lo(jtbl_80004080)
    /* 000014 80004014 00042080 */  sll         $a0, $a0, 2
    /* 000018 80004018 00642021 */  addu        $a0, $v1, $a0
    /* 00001C 8000401C 8C830000 */  lw          $v1, 0x0($a0)
    /* 000020 80004020 00600008 */  jr          $v1
    /* 000024 80004024 00000000 */   nop
  jlabel .L80004028
    /* 000028 80004028 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 00002C 8000402C 03E00008 */  jr          $ra
    /* 000030 80004030 8C420014 */   lw         $v0, 0x14($v0)
  jlabel .L80004034
    /* 000034 80004034 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 000038 80004038 03E00008 */  jr          $ra
    /* 00003C 8000403C 8C420004 */   lw         $v0, 0x4($v0)
  jlabel .L80004040
    /* 000040 80004040 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 000044 80004044 03E00008 */  jr          $ra
    /* 000048 80004048 8C420008 */   lw         $v0, 0x8($v0)
  jlabel .L8000404C
    /* 00004C 8000404C 03E00008 */  jr          $ra
    /* 000050 80004050 8C4240A0 */   lw         $v0, %lo(B_800040A0)($v0)
  jlabel .L80004054
    /* 000054 80004054 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 000058 80004058 03E00008 */  jr          $ra
    /* 00005C 8000405C 8C420010 */   lw         $v0, 0x10($v0)
  jlabel .L80004060
    /* 000060 80004060 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 000064 80004064 03E00008 */  jr          $ra
    /* 000068 80004068 8C420018 */   lw         $v0, 0x18($v0)
  jlabel .L8000406C
    /* 00006C 8000406C 244240A0 */  addiu       $v0, $v0, %lo(B_800040A0)
    /* 000070 80004070 03E00008 */  jr          $ra
    /* 000074 80004074 8C42000C */   lw         $v0, 0xC($v0)
.size func_80004000, . - func_80004000

/* Automatically generated and unreferenced pad */
glabel func_80004078
    /* 000078 80004078 03E00008 */  jr          $ra
    /* 00007C 8000407C 00000000 */   nop
.size func_80004078, . - func_80004078

.section .rodata

dlabel jtbl_80004080
    /* 000080 80004080 80004028 */ .word .L80004028
    /* 000084 80004084 80004034 */ .word .L80004034
    /* 000088 80004088 80004040 */ .word .L80004040
    /* 00008C 8000408C 8000404C */ .word .L8000404C
    /* 000090 80004090 80004054 */ .word .L80004054
    /* 000094 80004094 80004060 */ .word .L80004060
    /* 000098 80004098 8000406C */ .word .L8000406C
    /* 00009C 8000409C 00000000 */ .word 0x00000000
.size jtbl_80004080, . - jtbl_80004080

.section .bss

dlabel B_800040A0
    /* 800040A0 */ .space 0x20
";

    assert_eq!(disassembled_str, expected_str);
}

#[test]
fn test_mips1_doubles_eb() {
    let text_bytes: [u8; 21 * 4] = [
        // function
        0x3C, 0x04, 0x80, 0x00, // lui
        0xC4, 0x80, 0x00, 0xA4, // lwc1
        0x03, 0xE0, 0x00, 0x08, // jr
        0xC4, 0x81, 0x00, 0xA0, // lwc1
        // function
        0xC7, 0x80, 0x80, 0x24, // lwc1
        0x03, 0xE0, 0x00, 0x08, // jr
        0xC7, 0x81, 0x80, 0x20, // lwc1
        // function
        0x3C, 0x1C, 0x00, 0x01, // lui
        0x27, 0x9C, 0x80, 0x64, // addiu
        0x03, 0x99, 0xE0, 0x21, // addu
        0x8F, 0x84, 0x80, 0x1C, // lw
        0xC4, 0x80, 0x00, 0x04, // lwc1
        0x03, 0xE0, 0x00, 0x08, // jr
        0xC4, 0x81, 0x00, 0x00, // lwc1
        // function
        0x3C, 0x1C, 0x00, 0x01, // lui
        0x27, 0x9C, 0x80, 0x48, // addiu
        0x03, 0x99, 0xE0, 0x21, // addu
        0x8F, 0x84, 0x80, 0x18, // lw
        0xC4, 0x81, 0x00, 0xA8, // lwc1
        0x03, 0xE0, 0x00, 0x08, // jr
        0xC4, 0x80, 0x00, 0xAC, // lwc1
    ];
    let text_rom = Rom::new(0x0);
    let text_vram = Vram::new(0x80000000);

    let data_bytes: [u8; 0] = [];
    let data_rom = Rom::new(0x60);
    let data_vram = Vram::new(0x80000060);

    let rodata_bytes: [u8; 4 * 4] = [
        0x40, 0x93, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x93, 0x48, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    let rodata_rom = Rom::new(0xA0);
    let rodata_vram = Vram::new(0x800000A0);

    let gcc_except_table_bytes: [u8; 0] = [];
    let gcc_except_table_rom = Rom::new(0xB0);
    let gcc_except_table_vram = Vram::new(0x800000B0);

    let bss_size = Size::new(0x0);
    let bss_vram = Vram::new(0x800000B0);

    let gp_config = Some(GpConfig::new_pic(GpValue::new(0x80008080)));
    let got_locals = vec![
        /* -0x7FF0($gp) */ GotLocalEntry::new(0x00000000), /* Lazy resolver */
        /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000), /* GNU extension */
        /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000), /* */
    ];
    let got_globals = vec![
        /* -0x7FE4($gp) */
        GotGlobalEntry::new(0x800000A0, 0x800000A0, false, "R_DBL_800000A0"), /* R_DBL_800000A0 */
    ];
    let global_offset_table = Some(GlobalOffsetTable::new(
        Vram::new(0x80000090),
        got_locals,
        got_globals,
    ));

    let executable_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let data_settings = DataSectionSettings::new(None);
    let noload_settings = NobitsSectionSettings::new(None);

    let sections = Sections::new(
        Endian::Big,
        (
            RawSectionInfo::new(&text_bytes, text_rom, text_vram),
            executable_settings,
        ),
        (
            RawSectionInfo::new(&data_bytes, data_rom, data_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(&rodata_bytes, rodata_rom, rodata_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(
                &gcc_except_table_bytes,
                gcc_except_table_rom,
                gcc_except_table_vram,
            ),
            data_settings,
        ),
        (
            RawNobitsSectionInfo::new(bss_size, bss_vram),
            noload_settings,
        ),
        gp_config,
        global_offset_table,
    );

    sections.print_global_segment_info();

    let instr_display_flags = InstructionDisplayFlags::default();
    let text_display_settings = FunctionDisplaySettings::new(instr_display_flags);

    let data_display_settings = SymDataDisplaySettings::new();
    let bss_display_settings = SymNobitsDisplaySettings::new();

    let disassembled_str = sections.display_to_string(
        &text_display_settings,
        &data_display_settings,
        &data_display_settings,
        &data_display_settings,
        &bss_display_settings,
    );

    println!("{}", disassembled_str);

    let expected_str = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 3C048000 */  lui         $a0, %hi(R_DBL_800000A0 + 0x4)
    /* 000004 80000004 C48000A4 */  lwc1        $fv0, %lo(R_DBL_800000A0 + 0x4)($a0)
    /* 000008 80000008 03E00008 */  jr          $ra
    /* 00000C 8000000C C48100A0 */   lwc1       $fv0f, %lo(R_DBL_800000A0)($a0)
.size func_80000000, . - func_80000000

/* Automatically generated and unreferenced pad */
glabel func_80000010
    /* 000010 80000010 C7808024 */  lwc1        $fv0, %gp_rel(R_DBL_800000A0 + 0x4)($gp)
    /* 000014 80000014 03E00008 */  jr          $ra
    /* 000018 80000018 C7818020 */   lwc1       $fv0f, %gp_rel(R_DBL_800000A0)($gp)
.size func_80000010, . - func_80000010

/* Automatically generated and unreferenced pad */
glabel func_8000001C
    /* 00001C 8000001C 3C1C0001 */  lui         $gp, %hi(_gp_disp)
    /* 000020 80000020 279C8064 */  addiu       $gp, $gp, %lo(_gp_disp)
    /* 000024 80000024 0399E021 */  addu        $gp, $gp, $t9
    /* 000028 80000028 8F84801C */  lw          $a0, %got(R_DBL_800000A0)($gp)
    /* 00002C 8000002C C4800004 */  lwc1        $fv0, 0x4($a0)
    /* 000030 80000030 03E00008 */  jr          $ra
    /* 000034 80000034 C4810000 */   lwc1       $fv0f, 0x0($a0)
.size func_8000001C, . - func_8000001C

/* Automatically generated and unreferenced pad */
glabel func_80000038
    /* 000038 80000038 3C1C0001 */  lui         $gp, %hi(_gp_disp)
    /* 00003C 8000003C 279C8048 */  addiu       $gp, $gp, %lo(_gp_disp)
    /* 000040 80000040 0399E021 */  addu        $gp, $gp, $t9
    /* 000044 80000044 8F848018 */  lw          $a0, %got(R_DBL_800000A8 + 0x4)($gp)
    /* 000048 80000048 C48100A8 */  lwc1        $fv0f, %lo(R_DBL_800000A8)($a0)
    /* 00004C 8000004C 03E00008 */  jr          $ra
    /* 000050 80000050 C48000AC */   lwc1       $fv0, %lo(R_DBL_800000A8 + 0x4)($a0)
.size func_80000038, . - func_80000038

.section .rodata

dlabel R_DBL_800000A0
    /* 0000A0 800000A0 4093480000000000 */ .double 1234.0
.size R_DBL_800000A0, . - R_DBL_800000A0

dlabel R_DBL_800000A8
    /* 0000A8 800000A8 4093480000000000 */ .double 1234.0
.size R_DBL_800000A8, . - R_DBL_800000A8
";

    assert_eq!(disassembled_str, expected_str);
}

#[test]
fn test_mips1_doubles_el() {
    let text_bytes: [u8; 21 * 4] = [
        // function
        0x00, 0x80, 0x04, 0x3C, // lui
        0xA0, 0x00, 0x80, 0xC4, // lwc1
        0x08, 0x00, 0xE0, 0x03, // jr
        0xA4, 0x00, 0x81, 0xC4, // lwc1
        // function
        0x24, 0x80, 0x81, 0xC7, // lwc1
        0x08, 0x00, 0xE0, 0x03, // jr
        0x20, 0x80, 0x80, 0xC7, // lwc1
        // function
        0x01, 0x00, 0x1C, 0x3C, // lui
        0x64, 0x80, 0x9C, 0x27, // addiu
        0x21, 0xE0, 0x99, 0x03, // addu
        0x1C, 0x80, 0x84, 0x8F, // lw
        0x04, 0x00, 0x81, 0xC4, // lwc1
        0x08, 0x00, 0xE0, 0x03, // jr
        0x00, 0x00, 0x80, 0xC4, // lwc1
        // function
        0x01, 0x00, 0x1C, 0x3C, // lui
        0x48, 0x80, 0x9C, 0x27, // addiu
        0x21, 0xE0, 0x99, 0x03, // addu
        0x18, 0x80, 0x84, 0x8F, // lw
        0xAC, 0x00, 0x81, 0xC4, // lwc1
        0x08, 0x00, 0xE0, 0x03, // jr
        0xA8, 0x00, 0x80, 0xC4, // lwc1
    ];
    let text_rom = Rom::new(0x0);
    let text_vram = Vram::new(0x80000000);

    let data_bytes: [u8; 0] = [];
    let data_rom = Rom::new(0x60);
    let data_vram = Vram::new(0x80000060);

    let rodata_bytes: [u8; 4 * 4] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x93, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x48, 0x93,
        0x40,
    ];
    let rodata_rom = Rom::new(0xA0);
    let rodata_vram = Vram::new(0x800000A0);

    let gcc_except_table_bytes: [u8; 0] = [];
    let gcc_except_table_rom = Rom::new(0xB0);
    let gcc_except_table_vram = Vram::new(0x800000B0);

    let bss_size = Size::new(0x0);
    let bss_vram = Vram::new(0x800000B0);

    let gp_config = Some(GpConfig::new_pic(GpValue::new(0x80008080)));
    let got_locals = vec![
        /* -0x7FF0($gp) */ GotLocalEntry::new(0x00000000), /* Lazy resolver */
        /* -0x7FEC($gp) */ GotLocalEntry::new(0x80000000), /* GNU extension */
        /* -0x7FE8($gp) */ GotLocalEntry::new(0x80000000), /* */
    ];
    let got_globals = vec![
        /* -0x7FE4($gp) */
        GotGlobalEntry::new(0x800000A0, 0x800000A0, false, "R_DBL_800000A0"), /* R_DBL_800000A0 */
    ];
    let global_offset_table = Some(GlobalOffsetTable::new(
        Vram::new(0x80000090),
        got_locals,
        got_globals,
    ));

    let executable_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let data_settings = DataSectionSettings::new(None);
    let noload_settings = NobitsSectionSettings::new(None);

    let sections = Sections::new(
        Endian::Little,
        (
            RawSectionInfo::new(&text_bytes, text_rom, text_vram),
            executable_settings,
        ),
        (
            RawSectionInfo::new(&data_bytes, data_rom, data_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(&rodata_bytes, rodata_rom, rodata_vram),
            data_settings,
        ),
        (
            RawSectionInfo::new(
                &gcc_except_table_bytes,
                gcc_except_table_rom,
                gcc_except_table_vram,
            ),
            data_settings,
        ),
        (
            RawNobitsSectionInfo::new(bss_size, bss_vram),
            noload_settings,
        ),
        gp_config,
        global_offset_table,
    );

    sections.print_global_segment_info();

    let instr_display_flags = InstructionDisplayFlags::default();
    let text_display_settings = FunctionDisplaySettings::new(instr_display_flags);

    let data_display_settings = SymDataDisplaySettings::new();
    let bss_display_settings = SymNobitsDisplaySettings::new();

    let disassembled_str = sections.display_to_string(
        &text_display_settings,
        &data_display_settings,
        &data_display_settings,
        &data_display_settings,
        &bss_display_settings,
    );

    println!("{}", disassembled_str);

    let expected_str = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 0080043C */  lui         $a0, %hi(R_DBL_800000A0)
    /* 000004 80000004 A00080C4 */  lwc1        $fv0, %lo(R_DBL_800000A0)($a0)
    /* 000008 80000008 0800E003 */  jr          $ra
    /* 00000C 8000000C A40081C4 */   lwc1       $fv0f, %lo(R_DBL_800000A0 + 0x4)($a0)
.size func_80000000, . - func_80000000

/* Automatically generated and unreferenced pad */
glabel func_80000010
    /* 000010 80000010 248081C7 */  lwc1        $fv0f, %gp_rel(R_DBL_800000A0 + 0x4)($gp)
    /* 000014 80000014 0800E003 */  jr          $ra
    /* 000018 80000018 208080C7 */   lwc1       $fv0, %gp_rel(R_DBL_800000A0)($gp)
.size func_80000010, . - func_80000010

/* Automatically generated and unreferenced pad */
glabel func_8000001C
    /* 00001C 8000001C 01001C3C */  lui         $gp, %hi(_gp_disp)
    /* 000020 80000020 64809C27 */  addiu       $gp, $gp, %lo(_gp_disp)
    /* 000024 80000024 21E09903 */  addu        $gp, $gp, $t9
    /* 000028 80000028 1C80848F */  lw          $a0, %got(R_DBL_800000A0)($gp)
    /* 00002C 8000002C 040081C4 */  lwc1        $fv0f, 0x4($a0)
    /* 000030 80000030 0800E003 */  jr          $ra
    /* 000034 80000034 000080C4 */   lwc1       $fv0, 0x0($a0)
.size func_8000001C, . - func_8000001C

/* Automatically generated and unreferenced pad */
glabel func_80000038
    /* 000038 80000038 01001C3C */  lui         $gp, %hi(_gp_disp)
    /* 00003C 8000003C 48809C27 */  addiu       $gp, $gp, %lo(_gp_disp)
    /* 000040 80000040 21E09903 */  addu        $gp, $gp, $t9
    /* 000044 80000044 1880848F */  lw          $a0, %got(R_DBL_800000A8)($gp)
    /* 000048 80000048 AC0081C4 */  lwc1        $fv0f, %lo(R_DBL_800000A8 + 0x4)($a0)
    /* 00004C 8000004C 0800E003 */  jr          $ra
    /* 000050 80000050 A80080C4 */   lwc1       $fv0, %lo(R_DBL_800000A8)($a0)
.size func_80000038, . - func_80000038

.section .rodata

dlabel R_DBL_800000A0
    /* 0000A0 800000A0 0000000000489340 */ .double 1234.0
.size R_DBL_800000A0, . - R_DBL_800000A0

dlabel R_DBL_800000A8
    /* 0000A8 800000A8 0000000000489340 */ .double 1234.0
.size R_DBL_800000A8, . - R_DBL_800000A8
";

    assert_eq!(disassembled_str, expected_str);
}
