/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::BTreeMap;

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    config::{Endian, GlobalConfig},
    context::{builder::UserSegmentBuilder, Context, ContextBuilder, GlobalSegmentBuilder},
    metadata::SegmentMetadata,
    parent_segment_info::ParentSegmentInfo,
    sections::{
        before_proc::{DataSectionSettings, ExecutableSectionSettings, NoloadSectionSettings},
        processed::{DataSectionProcessed, ExecutableSectionProcessed, NoloadSectionProcessed},
    },
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNoloadDisplaySettings},
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
struct RawNoloadSectionInfo {
    size: Size,
    vram: Vram,
}
impl RawNoloadSectionInfo {
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
    bss: Option<NoloadSectionProcessed>,
}
impl Sections {
    pub fn new(
        endian: Endian,
        text_info: (RawSectionInfo, ExecutableSectionSettings),
        data_info: (RawSectionInfo, DataSectionSettings),
        rodata_info: (RawSectionInfo, DataSectionSettings),
        gcc_except_table_info: (RawSectionInfo, DataSectionSettings),
        bss_info: (RawNoloadSectionInfo, NoloadSectionSettings),
    ) -> Self {
        let mut global_ranges = text_info.0.ranges();

        global_ranges.expand_ranges(&data_info.0.ranges());
        global_ranges.expand_ranges(&rodata_info.0.ranges());
        global_ranges.expand_ranges(&gcc_except_table_info.0.ranges());
        global_ranges.expand_ranges(&bss_info.0.ranges());

        let global_config = GlobalConfig::new(endian);
        let mut context = {
            let mut global_heater = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

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
        bss_display_settings: &SymNoloadDisplaySettings,
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

    let executable_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let data_settings = DataSectionSettings::new(None);
    let noload_settings = NoloadSectionSettings::new(None);

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
            RawNoloadSectionInfo::new(bss_size, bss_vram),
            noload_settings,
        ),
    );

    sections.print_global_segment_info();

    let instr_display_flags = InstructionDisplayFlags::default();
    let text_display_settings = FunctionDisplaySettings::new(instr_display_flags);

    let data_display_settings = SymDataDisplaySettings::new();
    let bss_display_settings = SymNoloadDisplaySettings::new();

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
