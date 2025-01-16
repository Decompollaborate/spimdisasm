/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    config::{Endian, GlobalConfig},
    context::{ContextBuilder, GlobalSegmentBuilder, OverlaySegmentBuilder},
    metadata::OverlayCategoryName,
    parent_segment_info::ParentSegmentInfo,
    sections::SectionExecutableSettings,
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
        SectionExecutableSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    let global_config = GlobalConfig::new(Endian::Big);
    let mut context = {
        let global_ranges = RomVramRange::new(
            AddressRange::new(rom, rom + size),
            AddressRange::new(vram, vram + size),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment.preanalyze_text(&global_config, &text_settings, &bytes, rom, vram);

        let mut builder = ContextBuilder::new(global_segment);

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

            builder.add_overlay(overlay_builder.finish_symbols());
        }

        builder.build(global_config)
    };

    let instr_display_flags = InstructionDisplayFlags::default();

    let mut section_text = context
        .create_section_text(
            &text_settings,
            "test".into(),
            &bytes,
            rom,
            vram,
            ParentSegmentInfo::new(rom, vram, None),
        )
        .unwrap();

    section_text.post_process(&context).unwrap();

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
        SectionExecutableSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    let global_config = GlobalConfig::new(Endian::Big);
    let mut context = {
        let global_ranges = RomVramRange::new(
            AddressRange::new(rom, rom + size),
            AddressRange::new(vram, vram + size),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment.preanalyze_text(&global_config, &text_settings, &bytes, rom, vram);

        let builder = ContextBuilder::new(global_segment);

        builder.build(global_config)
    };

    let instr_display_flags = InstructionDisplayFlags::default();

    let mut section_text = context
        .create_section_text(
            &text_settings,
            "test".into(),
            &bytes,
            rom,
            vram,
            ParentSegmentInfo::new(rom, vram, None),
        )
        .unwrap();

    section_text.post_process(&context).unwrap();

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
