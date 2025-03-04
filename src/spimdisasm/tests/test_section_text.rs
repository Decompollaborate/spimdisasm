/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::BTreeMap;

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaExtension, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, GpValue, Rom, RomVramRange, Size, Vram},
    collections::addended_ordered_map::FindSettings,
    config::{Compiler, Endian, GlobalConfig, GpConfig},
    context::{
        builder::UserSegmentBuilder, ContextBuilder, GlobalSegmentBuilder, OverlaySegmentBuilder,
    },
    metadata::OverlayCategoryName,
    parent_segment_info::ParentSegmentInfo,
    sections::before_proc::ExecutableSectionSettings,
    symbols::display::FunctionDisplaySettings,
};

#[test]
fn test_section_text_1() {
    let bytes = [
        // 0x80000400
        0x27, 0xBD, 0xFF, 0xE8, // addiu
        0xAF, 0xB0, 0x00, 0x10, // sw
        0xAF, 0xBF, 0x00, 0x14, // sw
        0x0C, 0x00, 0x01, 0x3F, // jal
        0x00, 0x80, 0x80, 0x21, //  addu
        0x0C, 0x00, 0x01, 0x3F, // jal
        0x02, 0x00, 0x20, 0x21, //  addu
        0x3C, 0x03, 0x80, 0x1A, // lui
        0x8C, 0x63, 0x6D, 0x7C, // lw
        0x24, 0x02, 0x00, 0x02, // addiu
        0x14, 0x62, 0x00, 0x08, // bne
        0x00, 0x00, 0x00, 0x00, //  nop
        0x0C, 0x00, 0x01, 0x3F, // jal
        0x02, 0x00, 0x20, 0x21, //  addu
        0x8F, 0xBF, 0x00, 0x14, // lw
        0x8F, 0xB0, 0x00, 0x10, // lw
        0x27, 0xBD, 0x00, 0x18, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, //  nop
        0x8F, 0xBF, 0x00, 0x14, // lw
        0x8F, 0xB0, 0x00, 0x10, // lw
        0x27, 0xBD, 0x00, 0x18, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, //  nop
        // 0x80000460
        0x27, 0xBD, 0xFF, 0xD0, // addiu
        0x3C, 0x04, 0x01, 0x07, // lui
        0x24, 0x84, 0x7C, 0x60, // addiu
        0xAF, 0xB1, 0x00, 0x24, // sw
        0x3C, 0x11, 0x80, 0x1C, // lui
        0x8E, 0x31, 0x70, 0xB0, // lw
        0x3C, 0x05, 0x01, 0x08, // lui
        0x24, 0xA5, 0x35, 0xA0, // addiu
        0x00, 0xA4, 0x28, 0x23, // subu
        0xAF, 0xBF, 0x00, 0x28, // sw
        0x0C, 0x02, 0x66, 0x00, // jal
        0xAF, 0xB0, 0x00, 0x20, //  sw
        0x3C, 0x04, 0x00, 0x58, // lui
        0x24, 0x84, 0x1D, 0xF0, // addiu
        0x3C, 0x10, 0x00, 0x59, // lui
        0x26, 0x10, 0xD7, 0x30, // addiu
        0x02, 0x04, 0x80, 0x23, // subu
        0x02, 0x00, 0x30, 0x21, // addu
        0x0C, 0x00, 0x04, 0xC4, // jal
        0x02, 0x20, 0x28, 0x21, //  addu
        0x02, 0x20, 0x20, 0x21, // addu
        0x3C, 0x02, 0x80, 0x1C, // lui
        0x8C, 0x42, 0x70, 0xB0, // lw
        0x24, 0x05, 0xFF, 0xFF, // addiu
        0x00, 0x00, 0x30, 0x21, // addu
        0xAF, 0xA0, 0x00, 0x10, // sw
        0xAF, 0xA0, 0x00, 0x14, // sw
        0xAF, 0xA0, 0x00, 0x18, // sw
        0x00, 0x50, 0x10, 0x21, // addu
        0x3C, 0x01, 0x80, 0x1C, // lui
        0xAC, 0x22, 0x70, 0xB0, // sw
        0x0C, 0x01, 0x39, 0x39, // jal
        0x24, 0x07, 0x00, 0x02, //  addiu
        0x8F, 0xBF, 0x00, 0x28, // lw
        0x8F, 0xB1, 0x00, 0x24, // lw
        0x8F, 0xB0, 0x00, 0x20, // lw
        0x27, 0xBD, 0x00, 0x30, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, //  nop
        // 0x800004FC
        0x27, 0xBD, 0xFF, 0xE8, // addiu
        0xAF, 0xBF, 0x00, 0x10, // sw
        0x3C, 0x01, 0x80, 0x22, // lui
        0xAC, 0x20, 0xAD, 0x28, // sw
        0x0C, 0x01, 0xD2, 0xBB, // jal
        0x00, 0x00, 0x00, 0x00, //  nop
        0x8F, 0xBF, 0x00, 0x10, // lw
        0x27, 0xBD, 0x00, 0x18, // addiu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, //  nop
    ];
    let rom = Rom::new(0x001050);
    let vram = Vram::new(0x80000400);
    let size = Size::new(0x21FC00);

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    let global_config = GlobalConfig::new(Endian::Big);
    let mut context = {
        let global_ranges = RomVramRange::new(
            AddressRange::new(rom, rom + size),
            AddressRange::new(vram, vram + size),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &bytes, rom, vram)
            .unwrap();

        let mut platform_segment = UserSegmentBuilder::new();
        platform_segment.n64_libultra_symbols().unwrap();
        platform_segment.n64_hardware_registers(true, true).unwrap();

        let mut builder = ContextBuilder::new(global_segment, platform_segment);

        for i in 0x0..=0xF {
            let segment_name = format!("segment_0{:X}", i);
            let category_name = OverlayCategoryName::new(segment_name.clone());

            let magic_number = 0x01000000;
            let segment_size = Size::new(magic_number);
            let segment_vram = Vram::new(i * magic_number);
            let vram_range = AddressRange::new(segment_vram, segment_vram + segment_size);
            let arbitrary_number = 128 * 1024 * 1024; // 128MiB, no rom should be that big, right?
            let segment_rom = Rom::new(arbitrary_number + i * magic_number);
            let rom_range = AddressRange::new(segment_rom, segment_rom + segment_size);

            println!(
                "Adding overlay '{:?}': {:?} {:?}",
                category_name, rom_range, vram_range
            );

            let ranges = RomVramRange::new(rom_range, vram_range);

            let overlay_builder = OverlaySegmentBuilder::new(ranges, category_name, segment_name);

            builder
                .add_overlay(overlay_builder.finish_symbols())
                .unwrap();
        }

        builder.build(global_config).unwrap()
    };

    let instr_display_flags = InstructionDisplayFlags::default();

    let section_text = context
        .create_section_text(
            &text_settings,
            "test",
            &bytes,
            rom,
            vram,
            ParentSegmentInfo::new(rom, vram, None),
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    for func in section_text.functions() {
        let func_display = func.display(&context, &function_display_settings).unwrap();
        println!("{}", func_display);
    }

    assert_eq!(section_text.functions().len(), 3);

    let symbols = context.global_segment().symbols();
    for s in symbols {
        println!("{:?}", s.1);
    }
    assert_eq!(symbols.len(), 11);

    /*
    println!();
    let overlays_data = context
        .overlay_segments()
        .get(&OverlayCategoryName::new("segment_01".into()))
        .unwrap();
    println!("placeholder:");
    for sym in overlays_data.placeholder_segment().symbols() {
        println!("{:?}", sym);
    }
    println!();
    println!("other:");
    for (segment_rom, segment_metadata) in &overlays_data.segments() {
        println!("  {:?}", segment_rom,);
        for sym in segment_metadata.symbols() {
            println!("    {:?}", sym);
        }
    }
    */

    // None::<u32>.unwrap();
}

#[test]
fn test_section_text_lui_delay_slot() {
    let bytes = [
        0x94, 0xA3, 0x00, 0x9A, // lhu
        0x24, 0x02, 0x7F, 0xFF, // addiu
        0x10, 0x62, 0x00, 0x0D, // beq
        0x3C, 0x03, 0x7F, 0xFF, //  lui
        0x94, 0xA3, 0x00, 0xB2, // lhu
        0x10, 0x60, 0x00, 0x04, // beqz
        0x00, 0x03, 0x1A, 0x00, //  sll
        0x8C, 0xA2, 0x00, 0x40, // lw
        0x08, 0x02, 0x05, 0xE1, // j
        0x00, 0x43, 0x10, 0x21, //  addu
        0x94, 0xA2, 0x00, 0xB4, // lhu
        0x8C, 0xA3, 0x00, 0x3C, // lw
        0x00, 0x02, 0x12, 0x00, // sll
        0x00, 0x62, 0x18, 0x23, // subu
        0x08, 0x02, 0x05, 0xE1, // j
        0xAC, 0xA3, 0x00, 0x54, //  sw
        0x8C, 0xA2, 0x00, 0x40, // lw
        0x34, 0x63, 0xFF, 0xFF, // ori
        0x00, 0x43, 0x10, 0x21, // addu
        0x03, 0xE0, 0x00, 0x08, // jr
        0xAC, 0xA2, 0x00, 0x54, //  sw
    ];
    let rom = Rom::new(0x069558);
    let vram = Vram::new(0x80081738);
    let size = Size::new(0x1000);

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    let global_config = GlobalConfig::new(Endian::Big);
    let mut context = {
        let global_ranges = RomVramRange::new(
            AddressRange::new(rom, rom + size),
            AddressRange::new(vram, vram + size),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &bytes, rom, vram)
            .unwrap();

        let mut platform_segment = UserSegmentBuilder::new();
        platform_segment.n64_libultra_symbols().unwrap();
        platform_segment.n64_hardware_registers(true, true).unwrap();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let instr_display_flags = InstructionDisplayFlags::default();

    let section_text = context
        .create_section_text(
            &text_settings,
            "test",
            &bytes,
            rom,
            vram,
            ParentSegmentInfo::new(rom, vram, None),
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    for func in section_text.functions() {
        let func_display = func.display(&context, &function_display_settings);
        println!("{}", func_display.unwrap());
    }

    let expected_str = "\
glabel func_80081738
    /* 069558 80081738 94A3009A */  lhu         $v1, 0x9A($a1)
    /* 06955C 8008173C 24027FFF */  addiu       $v0, $zero, 0x7FFF
    /* 069560 80081740 1062000D */  beq         $v1, $v0, .L80081778
    /* 069564 80081744 3C037FFF */   lui        $v1, (0x7FFFFFFF >> 16)
    /* 069568 80081748 94A300B2 */  lhu         $v1, 0xB2($a1)
    /* 06956C 8008174C 10600004 */  beqz        $v1, .L80081760
    /* 069570 80081750 00031A00 */   sll        $v1, $v1, 8
    /* 069574 80081754 8CA20040 */  lw          $v0, 0x40($a1)
    /* 069578 80081758 080205E1 */  j           .L80081784
    /* 06957C 8008175C 00431021 */   addu       $v0, $v0, $v1
  .L80081760:
    /* 069580 80081760 94A200B4 */  lhu         $v0, 0xB4($a1)
    /* 069584 80081764 8CA3003C */  lw          $v1, 0x3C($a1)
    /* 069588 80081768 00021200 */  sll         $v0, $v0, 8
    /* 06958C 8008176C 00621823 */  subu        $v1, $v1, $v0
    /* 069590 80081770 080205E1 */  j           .L80081784
    /* 069594 80081774 ACA30054 */   sw         $v1, 0x54($a1)
  .L80081778:
    /* 069598 80081778 8CA20040 */  lw          $v0, 0x40($a1)
    /* 06959C 8008177C 3463FFFF */  ori         $v1, $v1, (0x7FFFFFFF & 0xFFFF)
    /* 0695A0 80081780 00431021 */  addu        $v0, $v0, $v1
  .L80081784:
    /* 0695A4 80081784 03E00008 */  jr          $ra
    /* 0695A8 80081788 ACA20054 */   sw         $v0, 0x54($a1)
.size func_80081738, . - func_80081738
";

    assert_eq!(
        section_text.functions()[0]
            .display(&context, &function_display_settings)
            .unwrap()
            .to_string(),
        expected_str
    );
}

#[test]
fn test_section_text_pairing_on_delay_slot() {
    // from pokemon snap, based on scMain
    static BYTES: [u8; 208] = [
        0x27, 0xBD, 0xFF, 0x80, //  addiu
        0xAF, 0xBF, 0x00, 0x1C, //  sw
        0xAF, 0xB0, 0x00, 0x18, //  sw
        0x3C, 0x10, 0x80, 0x05, //  lui
        0x26, 0x10, 0x88, 0x68, //  addiu
        0x02, 0x00, 0x20, 0x25, //  or
        0x27, 0xA5, 0x00, 0x7C, //  addiu
        0x0C, 0x00, 0xCA, 0x10, //  jal
        0x24, 0x06, 0x00, 0x01, //   addiu
        0x8F, 0xA4, 0x00, 0x7C, //  lw
        0x24, 0x01, 0x00, 0x01, //  addiu
        0x10, 0x81, 0x00, 0x09, //  beq
        0x24, 0x01, 0x00, 0x02, //   addiu
        0x10, 0x81, 0x00, 0x0B, //  beq
        0x24, 0x01, 0x00, 0x03, //   addiu
        0x10, 0x81, 0x00, 0x0D, //  beq
        0x24, 0x01, 0x00, 0x63, //   addiu
        0x10, 0x81, 0x00, 0x0F, //  beq
        0x3C, 0x02, 0x80, 0x05, //   lui
        0x10, 0x00, 0x00, 0x14, //  b
        0x3C, 0x18, 0x80, 0x05, //   lui
        0x0C, 0x00, 0x07, 0xE3, //  jal
        0x00, 0x00, 0x00, 0x00, //   nop
        0x10, 0x00, 0xFF, 0xEE, //  b
        0x02, 0x00, 0x20, 0x25, //   or
        0x0C, 0x00, 0x07, 0xF9, //  jal
        0x00, 0x00, 0x00, 0x00, //   nop
        0x10, 0x00, 0xFF, 0xEA, //  b
        0x02, 0x00, 0x20, 0x25, //   or
        0x0C, 0x00, 0x08, 0x95, //  jal
        0x00, 0x00, 0x00, 0x00, //   nop
        0x10, 0x00, 0xFF, 0xE6, //  b
        0x02, 0x00, 0x20, 0x25, //   or
        0x8C, 0x42, 0x88, 0x88, //  lw
        0x50, 0x40, 0xFF, 0xE3, //  beql
        0x02, 0x00, 0x20, 0x25, //   or
        0x00, 0x40, 0xF8, 0x09, //  jalr
        0x00, 0x00, 0x00, 0x00, //   nop
        0x10, 0x00, 0xFF, 0xDF, //  b
        0x02, 0x00, 0x20, 0x25, //   or
        0x8F, 0x18, 0x88, 0x8C, //  lw
        0x57, 0x00, 0xFF, 0xDC, //  bnel
        0x02, 0x00, 0x20, 0x25, //   or
        0x0C, 0x00, 0x09, 0x39, //  jal
        0x00, 0x00, 0x00, 0x00, //   nop
        0x10, 0x00, 0xFF, 0xD8, //  b
        0x02, 0x00, 0x20, 0x25, //   or
        0x8F, 0xBF, 0x00, 0x1C, //  lw
        0x8F, 0xB0, 0x00, 0x18, //  lw
        0x27, 0xBD, 0x00, 0x80, //  addiu
        0x03, 0xE0, 0x00, 0x08, //  jr
        0x00, 0x00, 0x00, 0x00, //   nop
    ];

    let rom = Rom::new(0x3118);
    let vram = Vram::new(0x80002518);

    let segment_rom = Rom::new(0x1000);
    let segment_vram = Vram::new(0x80000400);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );

    let mut context = {
        let global_config = GlobalConfig::new(Endian::Big);

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x46270)),
            AddressRange::new(segment_vram, Vram::new(0x8009A8C0)),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &BYTES, rom, vram)
            .unwrap();

        let mut platform_segment = UserSegmentBuilder::new();
        platform_segment.n64_libultra_symbols().unwrap();
        platform_segment.n64_hardware_registers(true, true).unwrap();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text".to_string(),
            &BYTES,
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let mut disassembly = ".section .text\n".to_string();
    let display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    for sym in section_text.functions() {
        disassembly.push('\n');
        disassembly.push_str(
            &sym.display(&context, &display_settings)
                .unwrap()
                .to_string(),
        );
    }

    println!("{}", disassembly);

    let expected_disassembly = "\
.section .text

glabel func_80002518
    /* 003118 80002518 27BDFF80 */  addiu       $sp, $sp, -0x80
    /* 00311C 8000251C AFBF001C */  sw          $ra, 0x1C($sp)
    /* 003120 80002520 AFB00018 */  sw          $s0, 0x18($sp)
    /* 003124 80002524 3C108005 */  lui         $s0, %hi(UNK_80048868)
    /* 003128 80002528 26108868 */  addiu       $s0, $s0, %lo(UNK_80048868)
    /* 00312C 8000252C 02002025 */  or          $a0, $s0, $zero
  .L80002530:
    /* 003130 80002530 27A5007C */  addiu       $a1, $sp, 0x7C
    /* 003134 80002534 0C00CA10 */  jal         UNK_func_80032840
    /* 003138 80002538 24060001 */   addiu      $a2, $zero, 0x1
    /* 00313C 8000253C 8FA4007C */  lw          $a0, 0x7C($sp)
    /* 003140 80002540 24010001 */  addiu       $at, $zero, 0x1
    /* 003144 80002544 10810009 */  beq         $a0, $at, .L8000256C
    /* 003148 80002548 24010002 */   addiu      $at, $zero, 0x2
    /* 00314C 8000254C 1081000B */  beq         $a0, $at, .L8000257C
    /* 003150 80002550 24010003 */   addiu      $at, $zero, 0x3
    /* 003154 80002554 1081000D */  beq         $a0, $at, .L8000258C
    /* 003158 80002558 24010063 */   addiu      $at, $zero, 0x63
    /* 00315C 8000255C 1081000F */  beq         $a0, $at, .L8000259C
    /* 003160 80002560 3C028005 */   lui        $v0, %hi(UNK_80048888)
    /* 003164 80002564 10000014 */  b           .L800025B8
    /* 003168 80002568 3C188005 */   lui        $t8, %hi(UNK_8004888C)
  .L8000256C:
    /* 00316C 8000256C 0C0007E3 */  jal         UNK_func_80001F8C
    /* 003170 80002570 00000000 */   nop
    /* 003174 80002574 1000FFEE */  b           .L80002530
    /* 003178 80002578 02002025 */   or         $a0, $s0, $zero
  .L8000257C:
    /* 00317C 8000257C 0C0007F9 */  jal         UNK_func_80001FE4
    /* 003180 80002580 00000000 */   nop
    /* 003184 80002584 1000FFEA */  b           .L80002530
    /* 003188 80002588 02002025 */   or         $a0, $s0, $zero
  .L8000258C:
    /* 00318C 8000258C 0C000895 */  jal         UNK_func_80002254
    /* 003190 80002590 00000000 */   nop
    /* 003194 80002594 1000FFE6 */  b           .L80002530
    /* 003198 80002598 02002025 */   or         $a0, $s0, $zero
  .L8000259C:
    /* 00319C 8000259C 8C428888 */  lw          $v0, %lo(UNK_80048888)($v0)
    /* 0031A0 800025A0 5040FFE3 */  beqzl       $v0, .L80002530
    /* 0031A4 800025A4 02002025 */   or         $a0, $s0, $zero
    /* 0031A8 800025A8 0040F809 */  jalr        $v0
    /* 0031AC 800025AC 00000000 */   nop
    /* 0031B0 800025B0 1000FFDF */  b           .L80002530
    /* 0031B4 800025B4 02002025 */   or         $a0, $s0, $zero
  .L800025B8:
    /* 0031B8 800025B8 8F18888C */  lw          $t8, %lo(UNK_8004888C)($t8)
    /* 0031BC 800025BC 5700FFDC */  bnezl       $t8, .L80002530
    /* 0031C0 800025C0 02002025 */   or         $a0, $s0, $zero
    /* 0031C4 800025C4 0C000939 */  jal         UNK_func_800024E4
    /* 0031C8 800025C8 00000000 */   nop
    /* 0031CC 800025CC 1000FFD8 */  b           .L80002530
    /* 0031D0 800025D0 02002025 */   or         $a0, $s0, $zero
    /* 0031D4 800025D4 8FBF001C */  lw          $ra, 0x1C($sp)
    /* 0031D8 800025D8 8FB00018 */  lw          $s0, 0x18($sp)
    /* 0031DC 800025DC 27BD0080 */  addiu       $sp, $sp, 0x80
    /* 0031E0 800025E0 03E00008 */  jr          $ra
    /* 0031E4 800025E4 00000000 */   nop
.size func_80002518, . - func_80002518
";

    assert_eq!(disassembly, expected_disassembly,);
}

#[test]
fn test_section_text_lui_paired_with_lw_and_ori() {
    // __osSiRawStartDma, but minimized to the point the code doesn't do anything
    static BYTES: [u8; 64] = [
        0x3C, 0x03, 0xA4, 0x80, // lui
        0xAC, 0x62, 0x00, 0x00, // sw
        0x56, 0x00, 0x00, 0x03, // bnel
        0x3C, 0x03, 0xA4, 0x80, //  lui
        0x08, 0x00, 0x1E, 0x1F, // j
        0x34, 0x63, 0x00, 0x04, //  ori
        0x34, 0x63, 0x00, 0x10, // ori
        0x3C, 0x02, 0x1F, 0xC0, // lui
        0x34, 0x42, 0x07, 0xC0, // ori
        0xAC, 0x62, 0x00, 0x00, // sw
        0x16, 0x00, 0x00, 0x03, // bnez
        0x00, 0x00, 0x10, 0x21, //  addu
        0x02, 0x20, 0x20, 0x21, // addu
        0x00, 0x00, 0x10, 0x21, // addu
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, // nop
    ];

    let rom = Rom::new(0x008460);
    let vram = Vram::new(0x80007860);

    let segment_rom = Rom::new(0x1000);
    let segment_vram = Vram::new(0x80000400);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::KMC),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );

    let mut context = {
        let global_config = GlobalConfig::new(Endian::Big);

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x46270)),
            AddressRange::new(segment_vram, Vram::new(0x8009A8C0)),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &BYTES, rom, vram)
            .unwrap();

        let mut platform_segment = UserSegmentBuilder::new();
        platform_segment.n64_libultra_symbols().unwrap();
        platform_segment.n64_hardware_registers(true, true).unwrap();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text".to_string(),
            &BYTES,
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let mut disassembly = ".section .text\n".to_string();
    let display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    for sym in section_text.functions() {
        disassembly.push('\n');
        disassembly.push_str(
            &sym.display(&context, &display_settings)
                .unwrap()
                .to_string(),
        );
    }

    println!("{}", disassembly);

    let expected_disassembly = "\
.section .text

glabel func_80007860
    /* 008460 80007860 3C03A480 */  lui         $v1, %hi(SI_DRAM_ADDR_REG)
    /* 008464 80007864 AC620000 */  sw          $v0, %lo(SI_DRAM_ADDR_REG)($v1)
    /* 008468 80007868 56000003 */  bnezl       $s0, .L80007878
    /* 00846C 8000786C 3C03A480 */   lui        $v1, (0xA4800010 >> 16)
    /* 008470 80007870 08001E1F */  j           .L8000787C
    /* 008474 80007874 34630004 */   ori        $v1, $v1, (0xA4800004 & 0xFFFF)
  .L80007878:
    /* 008478 80007878 34630010 */  ori         $v1, $v1, (0xA4800010 & 0xFFFF)
  .L8000787C:
    /* 00847C 8000787C 3C021FC0 */  lui         $v0, (0x1FC007C0 >> 16)
    /* 008480 80007880 344207C0 */  ori         $v0, $v0, (0x1FC007C0 & 0xFFFF)
    /* 008484 80007884 AC620000 */  sw          $v0, 0x0($v1)
    /* 008488 80007888 16000003 */  bnez        $s0, .L80007898
    /* 00848C 8000788C 00001021 */   addu       $v0, $zero, $zero
    /* 008490 80007890 02202021 */  addu        $a0, $s1, $zero
    /* 008494 80007894 00001021 */  addu        $v0, $zero, $zero
  .L80007898:
    /* 008498 80007898 03E00008 */  jr          $ra
    /* 00849C 8000789C 00000000 */   nop
.size func_80007860, . - func_80007860
";

    assert_eq!(disassembly, expected_disassembly,);
}

#[test]
fn test_section_text_gp_rels() {
    static BYTES: [u8; 36] = [
        // func_80000000
        0x01, 0x80, 0x1C, 0x3C, // lui
        0x14, 0x80, 0x9C, 0x27, // addiu
        0x21, 0xF0, 0xA0, 0x03, // addu
        0x06, 0x00, 0x00, 0x0C, // jal
        0x00, 0x00, 0x00, 0x00, // nop
        0x4D, 0x00, 0x00, 0x00, // break
        // func_80000018
        0x10, 0x80, 0x82, 0x8F, // lw
        0x08, 0x00, 0xE0, 0x03, // jr
        0x00, 0x00, 0x00, 0x00, // nop
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let segment_rom = Rom::new(0x00000000);
    let segment_vram = Vram::new(0x80000000);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::PSYQ),
        InstructionFlags::new_extension(IsaExtension::R3000GTE),
    );

    let mut context = {
        let global_config = GlobalConfig::new(Endian::Little)
            .with_gp_config(Some(GpConfig::new_sdata(GpValue::new(0x80008014))));

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x00008000)),
            AddressRange::new(segment_vram, Vram::new(0x80008000)),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &BYTES, rom, vram)
            .unwrap();

        let platform_segment = UserSegmentBuilder::new();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text".to_string(),
            &BYTES,
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let mut disassembly = ".section .text\n".to_string();
    let display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    for sym in section_text.functions() {
        disassembly.push('\n');
        disassembly.push_str(
            &sym.display(&context, &display_settings)
                .unwrap()
                .to_string(),
        );
    }

    println!("{}", disassembly);

    let expected_disassembly = "\
.section .text

glabel func_80000000
    /* 000000 80000000 01801C3C */  lui         $gp, %hi(_gp)
    /* 000004 80000004 14809C27 */  addiu       $gp, $gp, %lo(_gp)
    /* 000008 80000008 21F0A003 */  addu        $s8, $sp, $zero
    /* 00000C 8000000C 0600000C */  jal         func_80000018
    /* 000010 80000010 00000000 */   nop
    /* 000014 80000014 4D000000 */  break       0, 1
.size func_80000000, . - func_80000000

glabel func_80000018
    /* 000018 80000018 1080828F */  lw          $v0, %gp_rel(UNK_80000024)($gp)
    /* 00001C 8000001C 0800E003 */  jr          $ra
    /* 000020 80000020 00000000 */   nop
.size func_80000018, . - func_80000018
";

    assert_eq!(disassembly, expected_disassembly,);
}

#[test]
fn test_section_text_type_inference_on_complex_control_flow() {
    static BYTES: [u8; 292] = [
        // func_8080010C
        0x27, 0xBD, 0xFF, 0xC8, // addiu
        0xAF, 0xB7, 0x00, 0x30, // sw
        0xAF, 0xB0, 0x00, 0x14, // sw
        0xAF, 0xB6, 0x00, 0x2C, // sw
        0xAF, 0xB5, 0x00, 0x28, // sw
        0xAF, 0xB4, 0x00, 0x24, // sw
        0xAF, 0xB3, 0x00, 0x20, // sw
        0xAF, 0xB2, 0x00, 0x1C, // sw
        0xAF, 0xB1, 0x00, 0x18, // sw
        0x3C, 0x10, 0x80, 0x80, // lui
        0x3C, 0x17, 0x80, 0x80, // lui
        0x00, 0x80, 0x88, 0x25, // or
        0x00, 0xC0, 0x90, 0x25, // or
        0x00, 0xA0, 0x98, 0x25, // or
        0xAF, 0xBF, 0x00, 0x34, // sw
        0x26, 0xF7, 0x1A, 0x4C, // addiu
        0x26, 0x10, 0x14, 0xA4, // addiu
        0x24, 0x14, 0xFF, 0xFC, // addiu
        0x24, 0x15, 0xFF, 0xFD, // addiu
        0x24, 0x16, 0xFF, 0xFE, // addiu
        0x96, 0x0E, 0x00, 0x00, // lhu
        0x56, 0x6E, 0x00, 0x26, // bnel
        0x26, 0x10, 0x00, 0x08, //  addiu
        0x12, 0x54, 0x00, 0x17, // beq
        0x00, 0x00, 0x00, 0x00, //  nop
        0x12, 0x55, 0x00, 0x05, // beq
        0x00, 0x00, 0x00, 0x00, //  nop
        0x12, 0x56, 0x00, 0x0F, // beq
        0x02, 0x20, 0x20, 0x25, //  or
        0x10, 0x00, 0x00, 0x1A, // b
        0x02, 0x20, 0x20, 0x25, //  or
        0x0C, 0x04, 0x19, 0xE4, // jal
        0x02, 0x20, 0x20, 0x25, //  or
        0x8C, 0x4F, 0x00, 0x7C, // lw
        0x02, 0x20, 0x20, 0x25, // or
        0x02, 0xC0, 0x28, 0x25, // or
        0x00, 0x0F, 0xCC, 0xC0, // sll
        0x07, 0x23, 0x00, 0x16, // bgezl
        0x26, 0x10, 0x00, 0x08, //  addiu
        0x0C, 0x20, 0x00, 0x00, // jal
        0x8E, 0x06, 0x00, 0x04, //  lw
        0x10, 0x00, 0x00, 0x12, // b
        0x26, 0x10, 0x00, 0x08, //  addiu
        0x0C, 0x20, 0x00, 0x2F, // jal
        0x8E, 0x05, 0x00, 0x04, //  lw
        0x10, 0x00, 0x00, 0x0E, // b
        0x26, 0x10, 0x00, 0x08, //  addiu
        0x0C, 0x03, 0x68, 0xA6, // jal
        0x24, 0x04, 0x00, 0x0E, //  addiu
        0x10, 0x40, 0x00, 0x09, // beqz
        0x02, 0x20, 0x20, 0x25, //  or
        0x24, 0x05, 0xFF, 0xFF, // addiu
        0x0C, 0x20, 0x00, 0x00, // jal
        0x8E, 0x06, 0x00, 0x04, //  lw
        0x10, 0x00, 0x00, 0x05, // b
        0x26, 0x10, 0x00, 0x08, //  addiu
        0x02, 0x40, 0x28, 0x25, // or
        0x0C, 0x20, 0x00, 0x00, // jal
        0x8E, 0x06, 0x00, 0x04, //  lw
        0x26, 0x10, 0x00, 0x08, // addiu
        0x56, 0x17, 0xFF, 0xD8, // bnel
        0x96, 0x0E, 0x00, 0x00, //  lhu
        0x8F, 0xBF, 0x00, 0x34, // lw
        0x8F, 0xB0, 0x00, 0x14, // lw
        0x8F, 0xB1, 0x00, 0x18, // lw
        0x8F, 0xB2, 0x00, 0x1C, // lw
        0x8F, 0xB3, 0x00, 0x20, // lw
        0x8F, 0xB4, 0x00, 0x24, // lw
        0x8F, 0xB5, 0x00, 0x28, // lw
        0x8F, 0xB6, 0x00, 0x2C, // lw
        0x8F, 0xB7, 0x00, 0x30, // lw
        0x03, 0xE0, 0x00, 0x08, // jr
        0x27, 0xBD, 0x00, 0x38, //  addiu
    ];

    let rom = Rom::new(0x02139F0C);
    let vram = Vram::new(0x8080010C);

    let segment_rom = Rom::new(0x00000000);
    let segment_vram = Vram::new(0x80000000);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new_isa(IsaVersion::MIPS_III, None),
    );

    let mut context = {
        let global_config = GlobalConfig::new(Endian::Big);

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x04000000)),
            AddressRange::new(segment_vram, Vram::new(0x84000000)),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, &BYTES, rom, vram)
            .unwrap();

        let platform_segment = UserSegmentBuilder::new();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text".to_string(),
            &BYTES,
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = BTreeMap::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    let mut disassembly = ".section .text\n".to_string();
    let display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    for sym in section_text.functions() {
        disassembly.push('\n');
        disassembly.push_str(
            &sym.display(&context, &display_settings)
                .unwrap()
                .to_string(),
        );
    }

    println!("{}", disassembly);

    let expected_disassembly = "\
.section .text

glabel func_8080010C
    /* 2139F0C 8080010C 27BDFFC8 */  addiu       $sp, $sp, -0x38
    /* 2139F10 80800110 AFB70030 */  sw          $s7, 0x30($sp)
    /* 2139F14 80800114 AFB00014 */  sw          $s0, 0x14($sp)
    /* 2139F18 80800118 AFB6002C */  sw          $s6, 0x2C($sp)
    /* 2139F1C 8080011C AFB50028 */  sw          $s5, 0x28($sp)
    /* 2139F20 80800120 AFB40024 */  sw          $s4, 0x24($sp)
    /* 2139F24 80800124 AFB30020 */  sw          $s3, 0x20($sp)
    /* 2139F28 80800128 AFB2001C */  sw          $s2, 0x1C($sp)
    /* 2139F2C 8080012C AFB10018 */  sw          $s1, 0x18($sp)
    /* 2139F30 80800130 3C108080 */  lui         $s0, %hi(UNK_808014A4)
    /* 2139F34 80800134 3C178080 */  lui         $s7, %hi(UNK_80801A4C)
    /* 2139F38 80800138 00808825 */  or          $s1, $a0, $zero
    /* 2139F3C 8080013C 00C09025 */  or          $s2, $a2, $zero
    /* 2139F40 80800140 00A09825 */  or          $s3, $a1, $zero
    /* 2139F44 80800144 AFBF0034 */  sw          $ra, 0x34($sp)
    /* 2139F48 80800148 26F71A4C */  addiu       $s7, $s7, %lo(UNK_80801A4C)
    /* 2139F4C 8080014C 261014A4 */  addiu       $s0, $s0, %lo(UNK_808014A4)
    /* 2139F50 80800150 2414FFFC */  addiu       $s4, $zero, -0x4
    /* 2139F54 80800154 2415FFFD */  addiu       $s5, $zero, -0x3
    /* 2139F58 80800158 2416FFFE */  addiu       $s6, $zero, -0x2
    /* 2139F5C 8080015C 960E0000 */  lhu         $t6, 0x0($s0)
  .L80800160:
    /* 2139F60 80800160 566E0026 */  bnel        $s3, $t6, .L808001FC
    /* 2139F64 80800164 26100008 */   addiu      $s0, $s0, 0x8
    /* 2139F68 80800168 12540017 */  beq         $s2, $s4, .L808001C8
    /* 2139F6C 8080016C 00000000 */   nop
    /* 2139F70 80800170 12550005 */  beq         $s2, $s5, .L80800188
    /* 2139F74 80800174 00000000 */   nop
    /* 2139F78 80800178 1256000F */  beq         $s2, $s6, .L808001B8
    /* 2139F7C 8080017C 02202025 */   or         $a0, $s1, $zero
    /* 2139F80 80800180 1000001A */  b           .L808001EC
    /* 2139F84 80800184 02202025 */   or         $a0, $s1, $zero
  .L80800188:
    /* 2139F88 80800188 0C0419E4 */  jal         UNK_func_80106790
    /* 2139F8C 8080018C 02202025 */   or         $a0, $s1, $zero
    /* 2139F90 80800190 8C4F007C */  lw          $t7, 0x7C($v0)
    /* 2139F94 80800194 02202025 */  or          $a0, $s1, $zero
    /* 2139F98 80800198 02C02825 */  or          $a1, $s6, $zero
    /* 2139F9C 8080019C 000FCCC0 */  sll         $t9, $t7, 19
    /* 2139FA0 808001A0 07230016 */  bgezl       $t9, .L808001FC
    /* 2139FA4 808001A4 26100008 */   addiu      $s0, $s0, 0x8
    /* 2139FA8 808001A8 0C200000 */  jal         UNK_func_80800000
    /* 2139FAC 808001AC 8E060004 */   lw         $a2, 0x4($s0)
    /* 2139FB0 808001B0 10000012 */  b           .L808001FC
    /* 2139FB4 808001B4 26100008 */   addiu      $s0, $s0, 0x8
  .L808001B8:
    /* 2139FB8 808001B8 0C20002F */  jal         UNK_func_808000BC
    /* 2139FBC 808001BC 8E050004 */   lw         $a1, 0x4($s0)
    /* 2139FC0 808001C0 1000000E */  b           .L808001FC
    /* 2139FC4 808001C4 26100008 */   addiu      $s0, $s0, 0x8
  .L808001C8:
    /* 2139FC8 808001C8 0C0368A6 */  jal         UNK_func_800DA298
    /* 2139FCC 808001CC 2404000E */   addiu      $a0, $zero, 0xE
    /* 2139FD0 808001D0 10400009 */  beqz        $v0, .L808001F8
    /* 2139FD4 808001D4 02202025 */   or         $a0, $s1, $zero
    /* 2139FD8 808001D8 2405FFFF */  addiu       $a1, $zero, -0x1
    /* 2139FDC 808001DC 0C200000 */  jal         UNK_func_80800000
    /* 2139FE0 808001E0 8E060004 */   lw         $a2, 0x4($s0)
    /* 2139FE4 808001E4 10000005 */  b           .L808001FC
    /* 2139FE8 808001E8 26100008 */   addiu      $s0, $s0, 0x8
  .L808001EC:
    /* 2139FEC 808001EC 02402825 */  or          $a1, $s2, $zero
    /* 2139FF0 808001F0 0C200000 */  jal         UNK_func_80800000
    /* 2139FF4 808001F4 8E060004 */   lw         $a2, 0x4($s0)
  .L808001F8:
    /* 2139FF8 808001F8 26100008 */  addiu       $s0, $s0, 0x8
  .L808001FC:
    /* 2139FFC 808001FC 5617FFD8 */  bnel        $s0, $s7, .L80800160
    /* 213A000 80800200 960E0000 */   lhu        $t6, 0x0($s0)
    /* 213A004 80800204 8FBF0034 */  lw          $ra, 0x34($sp)
    /* 213A008 80800208 8FB00014 */  lw          $s0, 0x14($sp)
    /* 213A00C 8080020C 8FB10018 */  lw          $s1, 0x18($sp)
    /* 213A010 80800210 8FB2001C */  lw          $s2, 0x1C($sp)
    /* 213A014 80800214 8FB30020 */  lw          $s3, 0x20($sp)
    /* 213A018 80800218 8FB40024 */  lw          $s4, 0x24($sp)
    /* 213A01C 8080021C 8FB50028 */  lw          $s5, 0x28($sp)
    /* 213A020 80800220 8FB6002C */  lw          $s6, 0x2C($sp)
    /* 213A024 80800224 8FB70030 */  lw          $s7, 0x30($sp)
    /* 213A028 80800228 03E00008 */  jr          $ra
    /* 213A02C 8080022C 27BD0038 */   addiu      $sp, $sp, 0x38
.size func_8080010C, . - func_8080010C
";

    assert_eq!(disassembly, expected_disassembly);

    let silly_symbol = context
        .global_segment()
        .symbols()
        .find(&Vram::new(0x808014A4), FindSettings::new(false))
        .unwrap();

    println!("{:?} {:?}", silly_symbol, silly_symbol.owner_segment_kind());
    assert_eq!(silly_symbol.all_access_types().len(), 2);
}
