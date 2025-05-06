/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::BTreeMap;

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaExtension, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, GpValue, Rom, RomVramRange, Size, Vram},
    collections::addended_ordered_map::FindSettings,
    config::{Compiler, Endian, GlobalConfigBuilder, GpConfig},
    context::{
        builder::UserSegmentBuilder, Context, ContextBuilder, GlobalSegmentBuilder,
        OverlaySegmentBuilder,
    },
    metadata::{OverlayCategoryName, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    sections::{before_proc::ExecutableSectionSettings, processed::ExecutableSectionProcessed},
    symbols::display::FunctionDisplaySettings,
};

#[expect(clippy::too_many_arguments)]
fn disassemble_text(
    raw_bytes: &[u8],
    rom: Rom,
    vram: Vram,
    endian: Endian,
    gp_config: Option<GpConfig>,
    text_settings: ExecutableSectionSettings,
    fill_n64_symbols: bool,
    add_segmented_assets: bool,
    user_symbols: Vec<(String, Vram, Option<Rom>, Option<Size>, Option<SymbolType>)>,
) -> (String, Context, ExecutableSectionProcessed) {
    let segment_rom = Rom::new(0x00000000);
    let segment_vram = Vram::new(0x80000000);

    let section_name = "text";

    let mut context = {
        let global_config = GlobalConfigBuilder::new(endian)
            .with_gp_config(gp_config)
            .build();

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x04000000)),
            AddressRange::new(segment_vram, Vram::new(0x84000000)),
        );
        let mut global_segment_builder = GlobalSegmentBuilder::new(global_ranges);

        for (name, vram, rom, size, sym_type) in user_symbols {
            global_segment_builder
                .add_user_symbol(name, vram, rom, size, sym_type)
                .unwrap();
        }

        let mut global_segment_heater = global_segment_builder.finish_symbols();

        global_segment_heater
            .preheat_text(
                &global_config,
                &text_settings,
                section_name,
                raw_bytes,
                rom,
                vram,
            )
            .unwrap();

        let mut user_segment = UserSegmentBuilder::new();

        if fill_n64_symbols {
            matches!(
                text_settings.compiler(),
                Some(Compiler::IDO | Compiler::KMC | Compiler::SN64 | Compiler::EGCS)
            );
            user_segment.n64_libultra_symbols().unwrap();
            user_segment.n64_hardware_registers(true, true).unwrap();
        }

        let mut builder = ContextBuilder::new(global_segment_heater, user_segment);

        if add_segmented_assets {
            matches!(
                text_settings.compiler(),
                Some(Compiler::IDO | Compiler::KMC | Compiler::SN64 | Compiler::EGCS)
            );

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

                let overlay_builder =
                    OverlaySegmentBuilder::new(ranges, category_name, segment_name);

                builder
                    .add_overlay(overlay_builder.finish_symbols())
                    .unwrap();
            }
        }

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            section_name,
            raw_bytes,
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

    (disassembly, context, section_text)
}

#[test]
fn test_section_text_1() {
    static BYTES: [u8; 73 * 4] = [
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

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::KMC),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );
    let user_symbols = Vec::new();

    let (disassembly, context, section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        true,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000400
    /* 001050 80000400 27BDFFE8 */  addiu       $sp, $sp, -0x18
    /* 001054 80000404 AFB00010 */  sw          $s0, 0x10($sp)
    /* 001058 80000408 AFBF0014 */  sw          $ra, 0x14($sp)
    /* 00105C 8000040C 0C00013F */  jal         func_800004FC
    /* 001060 80000410 00808021 */   addu       $s0, $a0, $zero
    /* 001064 80000414 0C00013F */  jal         func_800004FC
    /* 001068 80000418 02002021 */   addu       $a0, $s0, $zero
    /* 00106C 8000041C 3C03801A */  lui         $v1, %hi(UNK_801A6D7C)
    /* 001070 80000420 8C636D7C */  lw          $v1, %lo(UNK_801A6D7C)($v1)
    /* 001074 80000424 24020002 */  addiu       $v0, $zero, 0x2
    /* 001078 80000428 14620008 */  bne         $v1, $v0, .L8000044C
    /* 00107C 8000042C 00000000 */   nop
    /* 001080 80000430 0C00013F */  jal         func_800004FC
    /* 001084 80000434 02002021 */   addu       $a0, $s0, $zero
    /* 001088 80000438 8FBF0014 */  lw          $ra, 0x14($sp)
    /* 00108C 8000043C 8FB00010 */  lw          $s0, 0x10($sp)
    /* 001090 80000440 27BD0018 */  addiu       $sp, $sp, 0x18
    /* 001094 80000444 03E00008 */  jr          $ra
    /* 001098 80000448 00000000 */   nop
  .L8000044C:
    /* 00109C 8000044C 8FBF0014 */  lw          $ra, 0x14($sp)
    /* 0010A0 80000450 8FB00010 */  lw          $s0, 0x10($sp)
    /* 0010A4 80000454 27BD0018 */  addiu       $sp, $sp, 0x18
    /* 0010A8 80000458 03E00008 */  jr          $ra
    /* 0010AC 8000045C 00000000 */   nop
.size func_80000400, . - func_80000400

/* Automatically generated and unreferenced pad */
glabel func_80000460
    /* 0010B0 80000460 27BDFFD0 */  addiu       $sp, $sp, -0x30
    /* 0010B4 80000464 3C040107 */  lui         $a0, %hi(UNK_01077C60)
    /* 0010B8 80000468 24847C60 */  addiu       $a0, $a0, %lo(UNK_01077C60)
    /* 0010BC 8000046C AFB10024 */  sw          $s1, 0x24($sp)
    /* 0010C0 80000470 3C11801C */  lui         $s1, %hi(UNK_801C70B0)
    /* 0010C4 80000474 8E3170B0 */  lw          $s1, %lo(UNK_801C70B0)($s1)
    /* 0010C8 80000478 3C050108 */  lui         $a1, %hi(UNK_010835A0)
    /* 0010CC 8000047C 24A535A0 */  addiu       $a1, $a1, %lo(UNK_010835A0)
    /* 0010D0 80000480 00A42823 */  subu        $a1, $a1, $a0
    /* 0010D4 80000484 AFBF0028 */  sw          $ra, 0x28($sp)
    /* 0010D8 80000488 0C026600 */  jal         UNK_func_80099800
    /* 0010DC 8000048C AFB00020 */   sw         $s0, 0x20($sp)
    /* 0010E0 80000490 3C040058 */  lui         $a0, %hi(UNK_00581DF0)
    /* 0010E4 80000494 24841DF0 */  addiu       $a0, $a0, %lo(UNK_00581DF0)
    /* 0010E8 80000498 3C100059 */  lui         $s0, %hi(UNK_0058D730)
    /* 0010EC 8000049C 2610D730 */  addiu       $s0, $s0, %lo(UNK_0058D730)
    /* 0010F0 800004A0 02048023 */  subu        $s0, $s0, $a0
    /* 0010F4 800004A4 02003021 */  addu        $a2, $s0, $zero
    /* 0010F8 800004A8 0C0004C4 */  jal         UNK_func_80001310
    /* 0010FC 800004AC 02202821 */   addu       $a1, $s1, $zero
    /* 001100 800004B0 02202021 */  addu        $a0, $s1, $zero
    /* 001104 800004B4 3C02801C */  lui         $v0, %hi(UNK_801C70B0)
    /* 001108 800004B8 8C4270B0 */  lw          $v0, %lo(UNK_801C70B0)($v0)
    /* 00110C 800004BC 2405FFFF */  addiu       $a1, $zero, -0x1
    /* 001110 800004C0 00003021 */  addu        $a2, $zero, $zero
    /* 001114 800004C4 AFA00010 */  sw          $zero, 0x10($sp)
    /* 001118 800004C8 AFA00014 */  sw          $zero, 0x14($sp)
    /* 00111C 800004CC AFA00018 */  sw          $zero, 0x18($sp)
    /* 001120 800004D0 00501021 */  addu        $v0, $v0, $s0
    /* 001124 800004D4 3C01801C */  lui         $at, %hi(UNK_801C70B0)
    /* 001128 800004D8 AC2270B0 */  sw          $v0, %lo(UNK_801C70B0)($at)
    /* 00112C 800004DC 0C013939 */  jal         UNK_func_8004E4E4
    /* 001130 800004E0 24070002 */   addiu      $a3, $zero, 0x2
    /* 001134 800004E4 8FBF0028 */  lw          $ra, 0x28($sp)
    /* 001138 800004E8 8FB10024 */  lw          $s1, 0x24($sp)
    /* 00113C 800004EC 8FB00020 */  lw          $s0, 0x20($sp)
    /* 001140 800004F0 27BD0030 */  addiu       $sp, $sp, 0x30
    /* 001144 800004F4 03E00008 */  jr          $ra
    /* 001148 800004F8 00000000 */   nop
.size func_80000460, . - func_80000460

glabel func_800004FC
    /* 00114C 800004FC 27BDFFE8 */  addiu       $sp, $sp, -0x18
    /* 001150 80000500 AFBF0010 */  sw          $ra, 0x10($sp)
    /* 001154 80000504 3C018022 */  lui         $at, %hi(UNK_8021AD28)
    /* 001158 80000508 AC20AD28 */  sw          $zero, %lo(UNK_8021AD28)($at)
    /* 00115C 8000050C 0C01D2BB */  jal         UNK_func_80074AEC
    /* 001160 80000510 00000000 */   nop
    /* 001164 80000514 8FBF0010 */  lw          $ra, 0x10($sp)
    /* 001168 80000518 27BD0018 */  addiu       $sp, $sp, 0x18
    /* 00116C 8000051C 03E00008 */  jr          $ra
    /* 001170 80000520 00000000 */   nop
.size func_800004FC, . - func_800004FC
";

    assert_eq!(disassembly, expected_disassembly,);

    assert_eq!(section_text.functions().len(), 3);

    let symbols = context.global_segment().symbols();
    for s in symbols {
        println!("{:?}", s.1);
    }
    assert_eq!(symbols.len(), 10);

    let labels = context.global_segment().labels();
    for s in labels {
        println!("{:?}", s.1);
    }
    assert_eq!(labels.len(), 4);
}

#[test]
fn test_section_text_lui_delay_slot() {
    static BYTES: [u8; 21 * 4] = [
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

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
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

    assert_eq!(disassembly, expected_disassembly);
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

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
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

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::KMC),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        true,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
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

    let endian = Endian::Little;
    let gp_config = Some(GpConfig::new_sdata(GpValue::new(0x80008014)));

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::PSYQ),
        InstructionFlags::new_extension(IsaExtension::R3000GTE),
    );
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
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

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new_isa(IsaVersion::MIPS_III, None),
    );
    let user_symbols = Vec::new();

    let (disassembly, context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
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

#[test]
fn test_section_text_exception_control_flow() {
    static BYTES: [u8; 14 * 4] = [
        // function
        0x00, 0x80, 0x1D, 0x3C, // lui
        0x6C, 0x1E, 0x00, 0x0C, // jal
        0xE0, 0x64, 0xBD, 0x27, // addiu
        0x0D, 0x00, 0x00, 0x00, // break
        // function
        0x80, 0x1F, 0x00, 0x0C, // jal
        0x00, 0x00, 0x00, 0x00, // nop
        0x0D, 0x00, 0x00, 0x00, // break
        // function
        0xC0, 0x1F, 0x00, 0x0C, // jal
        0x00, 0x00, 0x00, 0x00, // nop
        0x18, 0x00, 0x00, 0x42, // eret
        // function
        0x00, 0x00, 0x03, 0x24, // addiu
        0x0C, 0x00, 0x00, 0x00, // syscall
        0x08, 0x00, 0xE0, 0x03, // jr
        0x00, 0x00, 0x00, 0x00, // nop
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Little;
    let gp_config = Some(GpConfig::new_sdata(GpValue::new(0x80008014)));

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::EEGCC),
        InstructionFlags::new_extension(IsaExtension::R5900EE),
    );
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
.align 3
glabel func_80000000
    /* 000000 80000000 00801D3C */  lui         $sp, %hi(UNK_800064E0)
    /* 000004 80000004 6C1E000C */  jal         UNK_func_800079B0
    /* 000008 80000008 E064BD27 */   addiu      $sp, $sp, %lo(UNK_800064E0)
    /* 00000C 8000000C 0D000000 */  break       0
.size func_80000000, . - func_80000000

/* Automatically generated and unreferenced pad */
.align 3
glabel func_80000010
    /* 000010 80000010 801F000C */  jal         UNK_func_80007E00
    /* 000014 80000014 00000000 */   nop
    /* 000018 80000018 0D000000 */  break       0
.size func_80000010, . - func_80000010

/* Handwritten function */
/* Automatically generated and unreferenced pad */
glabel func_8000001C
    /* 00001C 8000001C C01F000C */  jal         UNK_func_80007F00
    /* 000020 80000020 00000000 */   nop
    /* 000024 80000024 18000042 */  eret /* handwritten instruction */
.size func_8000001C, . - func_8000001C

/* Handwritten function */
/* Automatically generated and unreferenced pad */
.align 3
glabel func_80000028
    /* 000028 80000028 00000324 */  addiu       $v1, $zero, 0x0
    /* 00002C 8000002C 0C000000 */  syscall     0 /* handwritten instruction */
    /* 000030 80000030 0800E003 */  jr          $ra
    /* 000034 80000034 00000000 */   nop
.size func_80000028, . - func_80000028
";

    assert_eq!(disassembly, expected_disassembly);
}

#[test]
fn test_section_text_forward_bal() {
    static BYTES: [u8; 10 * 4] = [
        // function
        0x27, 0xBD, 0xFF, 0xE0, // addiu
        0xAF, 0xBF, 0x00, 0x10, // sw
        0x04, 0x11, 0x00, 0x05, // bal
        0x24, 0x05, 0x00, 0x10, // addiu
        0x24, 0x42, 0x00, 0x10, // addiu
        0x8F, 0xBF, 0x00, 0x10, // lw
        0x03, 0xE0, 0x00, 0x08, // jr
        0x27, 0xBD, 0x00, 0x20, // addiu
        // function
        0x03, 0xE0, 0x00, 0x08, // jr
        0x00, 0x85, 0x10, 0x21, // addu
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Handwritten function */
/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 27BDFFE0 */  addiu       $sp, $sp, -0x20
    /* 000004 80000004 AFBF0010 */  sw          $ra, 0x10($sp)
    /* 000008 80000008 04110005 */  bal         func_80000020 /* handwritten instruction */
    /* 00000C 8000000C 24050010 */   addiu      $a1, $zero, 0x10
    /* 000010 80000010 24420010 */  addiu       $v0, $v0, 0x10
    /* 000014 80000014 8FBF0010 */  lw          $ra, 0x10($sp)
    /* 000018 80000018 03E00008 */  jr          $ra
    /* 00001C 8000001C 27BD0020 */   addiu      $sp, $sp, 0x20
.size func_80000000, . - func_80000000

glabel func_80000020
    /* 000020 80000020 03E00008 */  jr          $ra
    /* 000024 80000024 00851021 */   addu       $v0, $a0, $a1
.size func_80000020, . - func_80000020
";

    assert_eq!(disassembly, expected_disassembly);
}

#[test]
fn test_section_text_negative_branch_end() {
    static BYTES: [u8; 20 * 4] = [
        // function
        0x00, 0x00, 0x10, 0x25, // or
        0x3c, 0x03, 0x80, 0x00, // lui
        0x8c, 0x64, 0x01, 0x00, // lw
        0x14, 0x80, 0x00, 0x03, // bnez
        0x00, 0x00, 0x00, 0x00, // nop
        0x03, 0xe0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, // nop
        0x10, 0x00, 0xff, 0xfa, // b
        0x24, 0x42, 0x00, 0x01, // addiu
        // function
        0x00, 0x00, 0x10, 0x25, // or
        0x3c, 0x03, 0x80, 0x00, // lui
        0x8c, 0x64, 0x01, 0x04, // lw
        0x14, 0x80, 0x00, 0x03, // bnez
        0x00, 0x00, 0x00, 0x00, // nop
        0x03, 0xe0, 0x00, 0x08, // jr
        0x00, 0x00, 0x00, 0x00, // nop
        0x08, 0x00, 0x00, 0x0B, // j // TODO
        0x24, 0x42, 0x00, 0x01, // addiu
        // function
        0x03, 0xE0, 0x00, 0x08, //jr
        0x00, 0x00, 0x00, 0x00, //nop
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III))
            .with_negative_branch_as_end(true);
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 00001025 */  or          $v0, $zero, $zero
    /* 000004 80000004 3C038000 */  lui         $v1, %hi(UNK_80000100)
  .L80000008:
    /* 000008 80000008 8C640100 */  lw          $a0, %lo(UNK_80000100)($v1)
    /* 00000C 8000000C 14800003 */  bnez        $a0, .L8000001C
    /* 000010 80000010 00000000 */   nop
    /* 000014 80000014 03E00008 */  jr          $ra
    /* 000018 80000018 00000000 */   nop
  .L8000001C:
    /* 00001C 8000001C 1000FFFA */  b           .L80000008
    /* 000020 80000020 24420001 */   addiu      $v0, $v0, 0x1
.size func_80000000, . - func_80000000

/* Automatically generated and unreferenced pad */
glabel func_80000024
    /* 000024 80000024 00001025 */  or          $v0, $zero, $zero
    /* 000028 80000028 3C038000 */  lui         $v1, %hi(UNK_80000104)
  .L8000002C:
    /* 00002C 8000002C 8C640104 */  lw          $a0, %lo(UNK_80000104)($v1)
    /* 000030 80000030 14800003 */  bnez        $a0, .L80000040
    /* 000034 80000034 00000000 */   nop
    /* 000038 80000038 03E00008 */  jr          $ra
    /* 00003C 8000003C 00000000 */   nop
  .L80000040:
    /* 000040 80000040 0800000B */  j           .L8000002C
    /* 000044 80000044 24420001 */   addiu      $v0, $v0, 0x1
.size func_80000024, . - func_80000024

/* Automatically generated and unreferenced pad */
glabel func_80000048
    /* 000048 80000048 03E00008 */  jr          $ra
    /* 00004C 8000004C 00000000 */   nop
.size func_80000048, . - func_80000048
";

    assert_eq!(disassembly, expected_disassembly);
}

// KMC-generated code (N64)
#[test]
fn test_section_text_lwl_lwr_individual() {
    static BYTES: [u8; 15 * 4] = [
        0x3C, 0x02, 0x80, 0x00, // lui
        0x8C, 0x42, 0x00, 0x90, // lw
        0x27, 0xBD, 0xFF, 0xF8, // addiu
        0x00, 0x02, 0x10, 0x80, // sll
        0x3C, 0x01, 0x80, 0x00, // lui
        0x00, 0x22, 0x08, 0x21, // addu
        0x88, 0x23, 0x00, 0x60, // lwl
        0x3C, 0x01, 0x80, 0x00, // lui
        0x00, 0x22, 0x08, 0x21, // addu
        0x98, 0x23, 0x00, 0x63, // lwr
        0xAB, 0xA3, 0x00, 0x00, // swl
        0xBB, 0xA3, 0x00, 0x03, // swr
        0x83, 0xA2, 0x00, 0x00, // lb
        0x03, 0xE0, 0x00, 0x08, // jr
        0x27, 0xBD, 0x00, 0x08, // addiu
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 3C028000 */  lui         $v0, %hi(UNK_80000090)
    /* 000004 80000004 8C420090 */  lw          $v0, %lo(UNK_80000090)($v0)
    /* 000008 80000008 27BDFFF8 */  addiu       $sp, $sp, -0x8
    /* 00000C 8000000C 00021080 */  sll         $v0, $v0, 2
    /* 000010 80000010 3C018000 */  lui         $at, %hi(UNK_80000060)
    /* 000014 80000014 00220821 */  addu        $at, $at, $v0
    /* 000018 80000018 88230060 */  lwl         $v1, %lo(UNK_80000060)($at)
    /* 00001C 8000001C 3C018000 */  lui         $at, %hi(UNK_80000060 + 0x3)
    /* 000020 80000020 00220821 */  addu        $at, $at, $v0
    /* 000024 80000024 98230063 */  lwr         $v1, %lo(UNK_80000060 + 0x3)($at)
    /* 000028 80000028 ABA30000 */  swl         $v1, 0x0($sp)
    /* 00002C 8000002C BBA30003 */  swr         $v1, 0x3($sp)
    /* 000030 80000030 83A20000 */  lb          $v0, 0x0($sp)
    /* 000034 80000034 03E00008 */  jr          $ra
    /* 000038 80000038 27BD0008 */   addiu      $sp, $sp, 0x8
.size func_80000000, . - func_80000000
";

    assert_eq!(disassembly, expected_disassembly);
}

// GCCEE-SN-generated code (PS2)
#[test]
fn test_section_text_ldl_ldr_mixed() {
    static BYTES: [u8; 14 * 4] = [
        0xF0, 0xFF, 0xBD, 0x27, // addiu
        0x2D, 0x10, 0x80, 0x00, // move
        0x00, 0x00, 0xBF, 0x7F, // sq
        0x00, 0x80, 0x03, 0x3C, // lui
        0x2D, 0x20, 0xA0, 0x00, // move
        0xEC, 0x00, 0x49, 0x24, // addiu
        0xF0, 0x00, 0x47, 0x8C, // lw
        0x77, 0x00, 0x65, 0x68, // ldl
        0x70, 0x00, 0x65, 0x6C, // ldr
        0x0E, 0x00, 0x00, 0x0C, // jal
        0xE8, 0x00, 0x48, 0x8C, // lw
        0x00, 0x00, 0xBF, 0x7B, // lq
        0x08, 0x00, 0xE0, 0x03, // jr
        0x10, 0x00, 0xBD, 0x27, // addiu
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Little;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        None,
        InstructionFlags::new_extension(IsaExtension::R5900EE),
    );
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80000000
    /* 000000 80000000 F0FFBD27 */  addiu       $sp, $sp, -0x10
    /* 000004 80000004 2D108000 */  daddu       $v0, $a0, $zero
    /* 000008 80000008 0000BF7F */  sq          $ra, 0x0($sp)
    /* 00000C 8000000C 0080033C */  lui         $v1, %hi(UNK_80000070 + 0x7)
    /* 000010 80000010 2D20A000 */  daddu       $a0, $a1, $zero
    /* 000014 80000014 EC004924 */  addiu       $t1, $v0, 0xEC
    /* 000018 80000018 F000478C */  lw          $a3, 0xF0($v0)
    /* 00001C 8000001C 77006568 */  ldl         $a1, %lo(UNK_80000070 + 0x7)($v1)
    /* 000020 80000020 7000656C */  ldr         $a1, %lo(UNK_80000070)($v1)
    /* 000024 80000024 0E00000C */  jal         UNK_func_80000038
    /* 000028 80000028 E800488C */   lw         $t0, 0xE8($v0)
    /* 00002C 8000002C 0000BF7B */  lq          $ra, 0x0($sp)
    /* 000030 80000030 0800E003 */  jr          $ra
    /* 000034 80000034 1000BD27 */   addiu      $sp, $sp, 0x10
.size func_80000000, . - func_80000000
";

    assert_eq!(disassembly, expected_disassembly);
}

// Minified code taken from IDO 7.1 ld. Vram: 0x10070018
#[test]
fn test_section_text_ldl_ldr_gp_rel() {
    static BYTES: [u8; 7 * 4] = [
        0x6B, 0x88, 0xDF, 0x64, // ldl
        0x6F, 0x88, 0xDF, 0x6B, // ldr
        0x00, 0x08, 0x44, 0x3E, // dsrl32
        0x8F, 0x89, 0xDF, 0x74, // lw
        0x01, 0x28, 0x48, 0x21, // addu
        0x03, 0xE0, 0x00, 0x08, // jr
        0xAF, 0x89, 0xDF, 0x74, // sw
    ];

    let rom = Rom::new(0x00000000);
    let vram = Vram::new(0x80070018);

    let endian = Endian::Big;
    let gp_config = Some(GpConfig::new_pic(GpValue::new(0x100A89F8)));

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));
    let user_symbols = Vec::new();

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_80070018
    /* 000000 80070018 6B88DF64 */  ldl         $t0, %gp_rel(UNK_100A695C)($gp)
    /* 000004 8007001C 6F88DF6B */  ldr         $t0, %gp_rel(UNK_100A695C + 0x7)($gp)
    /* 000008 80070020 0008443E */  dsrl32      $t0, $t0, 16
    /* 00000C 80070024 8F89DF74 */  lw          $t1, %gp_rel(UNK_100A696C)($gp)
    /* 000010 80070028 01284821 */  addu        $t1, $t1, $t0
    /* 000014 8007002C 03E00008 */  jr          $ra
    /* 000018 80070030 AF89DF74 */   sw         $t1, %gp_rel(UNK_100A696C)($gp)
.size func_80070018, . - func_80070018
";

    assert_eq!(disassembly, expected_disassembly);
}

#[test]
fn test_section_text_1_instr_function() {
    // From IDO's 5.3 libc.so.1
    static BYTES: [u8; 4 * 10] = [
        // func
        0x8D, 0x49, 0x00, 0x00, // lw
        // func
        0x00, 0x00, 0x00, 0x00, // nop
        0x31, 0x29, 0x00, 0x01, // andi
        0x15, 0x20, 0xFF, 0xFC, // bnez
        0x00, 0x00, 0x00, 0x00, // nop
        // func
        0x8C, 0x88, 0x00, 0x00, // lw
        0x00, 0x00, 0x00, 0x00, // nop
        0x15, 0x05, 0x00, 0x05, // bne
        0x24, 0x02, 0x00, 0x01, // addiu
        0xAC, 0x86, 0x00, 0x00, // sw
    ];

    let rom = Rom::new(0x0);
    let vram = Vram::new(0x80000000);

    let endian = Endian::Big;
    let gp_config = None;

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new(IsaVersion::MIPS_I),
    )
    .with_negative_branch_as_end(true);

    let mut user_symbols = Vec::new();
    user_symbols.push((
        "_hcasstart".to_string(),
        Vram::new(0x80000000),
        None,
        None,
        None,
    ));
    user_symbols.push((
        "_hcasmayhave".to_string(),
        Vram::new(0x80000004),
        None,
        None,
        None,
    ));
    user_symbols.push((
        "_hcashavelock".to_string(),
        Vram::new(0x80000014),
        None,
        None,
        None,
    ));

    let (disassembly, _context, _section_text) = disassemble_text(
        &BYTES,
        rom,
        vram,
        endian,
        gp_config,
        text_settings,
        false,
        false,
        user_symbols,
    );

    let expected_disassembly = "\
.section .text

glabel _hcasstart
    /* 000000 80000000 8D490000 */  lw          $t1, 0x0($t2)
.size _hcasstart, . - _hcasstart

glabel _hcasmayhave
    /* 000004 80000004 00000000 */  nop
    /* 000008 80000008 31290001 */  andi        $t1, $t1, 0x1
    /* 00000C 8000000C 1520FFFC */  bnez        $t1, _hcasstart
    /* 000010 80000010 00000000 */   nop
.size _hcasmayhave, . - _hcasmayhave

glabel _hcashavelock
    /* 000014 80000014 8C880000 */  lw          $t0, 0x0($a0)
    /* 000018 80000018 00000000 */  nop
    /* 00001C 8000001C 15050005 */  bne         $t0, $a1, UNK_aent_80000034
    /* 000020 80000020 24020001 */   addiu      $v0, $zero, 0x1
    /* 000024 80000024 AC860000 */  sw          $a2, 0x0($a0)
.size _hcashavelock, . - _hcashavelock
";

    assert_eq!(disassembly, expected_disassembly,);
}
