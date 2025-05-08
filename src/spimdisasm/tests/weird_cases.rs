/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use std::collections::BTreeMap;

use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaExtension, IsaVersion};
use spimdisasm::{
    addresses::{AddressRange, GpValue, Rom, RomVramRange, Size, Vram},
    config::{Compiler, Endian, GlobalConfigBuilder, GpConfig},
    context::{builder::UserSegmentBuilder, ContextBuilder, GlobalSegmentBuilder},
    parent_segment_info::ParentSegmentInfo,
    sections::before_proc::ExecutableSectionSettings,
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings},
};

#[test]
fn oot_kaleido_scope_draw_world_map_1_0() {
    let bytes = [
        // 0x8081CE54
        0x27, 0xBD, 0xFD, 0xC0, // addiu
        0xAF, 0xB0, 0x00, 0x20, // sw
        0x00, 0xA0, 0x80, 0x25, // move
        0xAF, 0xBF, 0x00, 0x44, // sw
        0xAF, 0xBE, 0x00, 0x40, // sw
        0xAF, 0xB7, 0x00, 0x3C, // sw
        0xAF, 0xB6, 0x00, 0x38, // sw
        0xAF, 0xB5, 0x00, 0x34, // sw
        0xAF, 0xB4, 0x00, 0x30, // sw
        0xAF, 0xB3, 0x00, 0x2C, // sw
        0xAF, 0xB2, 0x00, 0x28, // sw
        0xAF, 0xB1, 0x00, 0x24, // sw
        0xAF, 0xA4, 0x02, 0x40, // sw
        0x8F, 0xAE, 0x02, 0x40, // lw
        0x3C, 0x01, 0x00, 0x01, // lui
        0x34, 0x21, 0x07, 0x60, // ori
        0x01, 0xC1, 0x90, 0x21, // addu
        0x02, 0x40, 0x18, 0x25, // move
        0x3C, 0x0F, 0x00, 0x01, // lui
        0x01, 0xEE, 0x78, 0x21, // addu
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x95, 0xEF, 0x09, 0x34, // lhu
        0x24, 0x01, 0x00, 0x06, // addiu
        0x55, 0xE1, 0x00, 0xC0, // bnel
        0x3C, 0x07, 0xE7, 0x00, //  lui
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x96, 0x58, 0x01, 0xE4, // lhu
        0x57, 0x00, 0x00, 0xBC, // bnel
        0x3C, 0x07, 0xE7, 0x00, //  lui
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x96, 0x59, 0x01, 0xE8, // lhu
        0x24, 0x01, 0x00, 0x01, // addiu
        0x57, 0x21, 0x00, 0xB7, // bnel
        0x3C, 0x07, 0xE7, 0x00, //  lui
        0x86, 0x42, 0x02, 0x20, // lh
        0x86, 0x44, 0x02, 0x38, // lh
        0xA6, 0x40, 0x02, 0x60, // sh
        0x00, 0x02, 0x9C, 0x00, // sll
        0x14, 0x80, 0x00, 0x40, // bnez
        0x00, 0x13, 0x9C, 0x03, //  sra
        0x86, 0x44, 0x02, 0x14, // lh
        0x24, 0x0F, 0x00, 0x0B, // addiu
        0x24, 0x05, 0x00, 0x0B, // addiu
        0x28, 0x81, 0x00, 0x1F, // slti
        0x14, 0x20, 0x00, 0x14, // bnez
        0x3C, 0x01, 0x80, 0x83, //  lui
        0xA4, 0x20, 0x99, 0xB0, // sh
        0x24, 0x4E, 0x00, 0x01, // addiu
        0xA6, 0x4E, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0x28, 0x41, 0x00, 0x0C, // slti
        0x54, 0x20, 0x00, 0x07, // bnel
        0x84, 0x78, 0x02, 0x20, //  lh
        0xA6, 0x4F, 0x02, 0x20, // sh
        0x0C, 0x20, 0x7C, 0x61, // jal
        0x8F, 0xA4, 0x02, 0x40, //  lw
        0x10, 0x00, 0x00, 0x06, // b
        0x86, 0x42, 0x02, 0x20, //  lh
        0x84, 0x78, 0x02, 0x20, // lh
        0x00, 0x78, 0xC8, 0x21, // addu
        0x93, 0x2E, 0x02, 0x66, // lbu
        0x51, 0xC0, 0xFF, 0xF2, // beql
        0x24, 0x4E, 0x00, 0x01, //  addiu
        0x10, 0x00, 0x00, 0x1D, // b
        0xAF, 0xB2, 0x00, 0x4C, //  sw
        0x28, 0x81, 0xFF, 0xE2, // slti
        0x10, 0x20, 0x00, 0x15, // beqz
        0x3C, 0x0F, 0x80, 0x83, //  lui
        0x3C, 0x01, 0x80, 0x83, // lui
        0xA4, 0x20, 0x99, 0xB0, // sh
        0x24, 0x4F, 0xFF, 0xFF, // addiu
        0xA6, 0x4F, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0x04, 0x43, 0x00, 0x08, // bgezl
        0x84, 0x78, 0x02, 0x20, //  lh
        0xA6, 0x40, 0x02, 0x20, // sh
        0x8F, 0xA4, 0x02, 0x40, // lw
        0x0C, 0x20, 0x7C, 0x61, // jal
        0x24, 0x05, 0x00, 0x0A, //  addiu
        0x10, 0x00, 0x00, 0x06, // b
        0x86, 0x42, 0x02, 0x20, //  lh
        0x84, 0x78, 0x02, 0x20, // lh
        0x00, 0x78, 0xC8, 0x21, // addu
        0x93, 0x2E, 0x02, 0x66, // lbu
        0x51, 0xC0, 0xFF, 0xF2, // beql
        0x24, 0x4F, 0xFF, 0xFF, //  addiu
        0x10, 0x00, 0x00, 0x06, // b
        0xAF, 0xB2, 0x00, 0x4C, //  sw
        0x95, 0xEF, 0x99, 0xB0, // lhu
        0x3C, 0x01, 0x80, 0x83, // lui
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x25, 0xF8, 0x00, 0x01, // addiu
        0xA4, 0x38, 0x99, 0xB0, // sh
        0x8F, 0xA4, 0x00, 0x4C, // lw
        0x24, 0x45, 0x00, 0x1F, // addiu
        0xA4, 0x85, 0x02, 0x48, // sh
        0x00, 0x05, 0x28, 0x80, // sll
        0x30, 0xA5, 0xFF, 0xFF, // andi
        0xA4, 0x82, 0x02, 0x40, // sh
        0x0C, 0x20, 0x66, 0xFD, // jal
        0x8C, 0x86, 0x01, 0x4C, //  lw
        0x10, 0x00, 0x00, 0x5D, // b
        0x8F, 0xB1, 0x00, 0x4C, //  lw
        0x3C, 0x0E, 0x80, 0x12, // lui
        0x85, 0xCE, 0xB9, 0xF0, // lh
        0x24, 0x01, 0x00, 0x0A, // addiu
        0x25, 0xCF, 0x00, 0x18, // addiu
        0x14, 0x81, 0x00, 0x2C, // bne
        0xA6, 0x4F, 0x02, 0x40, //  sh
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x86, 0x58, 0x02, 0x14, // lh
        0x2B, 0x01, 0x00, 0x1F, // slti
        0x14, 0x20, 0x00, 0x25, // bnez
        0x00, 0x00, 0x00, 0x00, //  no
        0xA6, 0x40, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0xA6, 0x40, 0x02, 0x38, // sh
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x02, 0x42, 0xC8, 0x21, // addu
        0x93, 0x2E, 0x02, 0x66, // lbu
        0x15, 0xC0, 0x00, 0x08, // bnez
        0x24, 0x4F, 0x00, 0x01, //  addiu
        0xA6, 0x4F, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0x02, 0x42, 0xC0, 0x21, // addu
        0x93, 0x19, 0x02, 0x66, // lbu
        0x53, 0x20, 0xFF, 0xFB, // beql
        0x24, 0x4F, 0x00, 0x01, //  addiu
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x8F, 0xA4, 0x00, 0x4C, // lw
        0x24, 0x45, 0x00, 0x1F, // addiu
        0xA4, 0x85, 0x02, 0x48, // sh
        0x00, 0x05, 0x28, 0x80, // sll
        0x30, 0xA5, 0xFF, 0xFF, // andi
        0xA4, 0x82, 0x02, 0x40, // sh
        0x0C, 0x20, 0x66, 0xFD, // jal
        0x8C, 0x86, 0x01, 0x4C, //  lw
        0x3C, 0x12, 0x80, 0x10, // lui
        0x3C, 0x0F, 0x80, 0x10, // lui
        0x26, 0x47, 0x43, 0xA0, // addiu
        0x25, 0xEF, 0x43, 0xA8, // addiu
        0x3C, 0x05, 0x80, 0x10, // lui
        0x24, 0xA5, 0x43, 0x94, // addiu
        0xAF, 0xAF, 0x00, 0x14, // sw
        0xAF, 0xA7, 0x00, 0x10, // sw
        0x24, 0x04, 0x48, 0x09, // addiu
        0x0C, 0x03, 0x20, 0x1B, // jal
        0x24, 0x06, 0x00, 0x04, //  addiu
        0x3C, 0x01, 0x80, 0x83, // lui
        0xA4, 0x20, 0x99, 0xB0, // sh
        0x10, 0x00, 0x00, 0x2C, // b
        0x8F, 0xB1, 0x00, 0x4C, //  lw
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x86, 0x58, 0x02, 0x14, // lh
        0x24, 0x19, 0x00, 0x0B, // addiu
        0x2B, 0x01, 0xFF, 0xE2, // slti
        0x50, 0x20, 0x00, 0x26, // beql
        0x8F, 0xB1, 0x00, 0x4C, //  lw
        0xA6, 0x59, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0xA6, 0x40, 0x02, 0x38, // sh
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x02, 0x42, 0x70, 0x21, // addu
        0x91, 0xCF, 0x02, 0x66, // lbu
        0x15, 0xE0, 0x00, 0x08, // bnez
        0x24, 0x58, 0xFF, 0xFF, //  addiu
        0xA6, 0x58, 0x02, 0x20, // sh
        0x86, 0x42, 0x02, 0x20, // lh
        0x02, 0x42, 0xC8, 0x21, // addu
        0x93, 0x2E, 0x02, 0x66, // lbu
        0x51, 0xC0, 0xFF, 0xFB, // beql
        0x24, 0x58, 0xFF, 0xFF, //  addiu
        0xAF, 0xB2, 0x00, 0x4C, // sw
        0x8F, 0xA4, 0x00, 0x4C, // lw
        0x24, 0x45, 0x00, 0x1F, // addiu
        0xA4, 0x85, 0x02, 0x48, // sh
        0x00, 0x05, 0x28, 0x80, // sll
        0x30, 0xA5, 0xFF, 0xFF, // andi
        0xA4, 0x82, 0x02, 0x40, // sh
        0x0C, 0x20, 0x66, 0xFD, // jal
        0x8C, 0x86, 0x01, 0x4C, //  lw
        0x3C, 0x12, 0x80, 0x10, // lui
        0x3C, 0x18, 0x80, 0x10, // lui
        0x26, 0x47, 0x43, 0xA0, // addiu
        0x27, 0x18, 0x43, 0xA8, // addiu
        0x3C, 0x05, 0x80, 0x10, // lui
        0x24, 0xA5, 0x43, 0x94, // addiu
        0xAF, 0xB8, 0x00, 0x14, // sw
        0xAF, 0xA7, 0x00, 0x10, // sw
        0x24, 0x04, 0x48, 0x09, // addiu
        0x0C, 0x03, 0x20, 0x1B, // jal
        0x24, 0x06, 0x00, 0x04, //  addiu
        0x3C, 0x01, 0x80, 0x83, // lui
        0xA4, 0x20, 0x99, 0xB0, // sh
        0x8F, 0xB1, 0x00, 0x4C, // lw
        0x3C, 0x12, 0x80, 0x10, // lui
        0x3C, 0x05, 0x80, 0x10, // lui
        0x86, 0x22, 0x02, 0x20, // lh
        0x26, 0x52, 0x43, 0xA0, // addiu
        0x24, 0x0F, 0x03, 0xE7, // addiu
        0x02, 0x22, 0xC8, 0x21, // addu
        0x93, 0x2E, 0x02, 0x66, // lbu
        0x24, 0xA5, 0x43, 0x94, // addiu
        0x24, 0x04, 0x48, 0x09, // addiu
        0x15, 0xC0, 0x00, 0x03, // bnez
        0x24, 0x06, 0x00, 0x04, //  addiu
        0xA6, 0x2F, 0x02, 0x40, // sh
        0x86, 0x22, 0x02, 0x20, // lh
        0x12, 0x62, 0x00, 0x06, // beq
        0x02, 0x40, 0x38, 0x25, //  move
        0x3C, 0x18, 0x80, 0x10, // lui
        0x27, 0x18, 0x43, 0xA8, // addiu
        0xAF, 0xB8, 0x00, 0x14, // sw
        0x0C, 0x03, 0x20, 0x1B, // jal
        0xAF, 0xB2, 0x00, 0x10, //  sw
        0x8F, 0xBF, 0x00, 0x44, // lw
        0x8F, 0xB0, 0x00, 0x20, // lw
        0x8F, 0xB1, 0x00, 0x24, // lw
        0x8F, 0xB2, 0x00, 0x28, // lw
        0x8F, 0xB3, 0x00, 0x2C, // lw
        0x8F, 0xB4, 0x00, 0x30, // lw
        0x8F, 0xB5, 0x00, 0x34, // lw
        0x8F, 0xB6, 0x00, 0x38, // lw
        0x8F, 0xB7, 0x00, 0x3C, // lw
        0x8F, 0xBE, 0x00, 0x40, // lw
        0x03, 0xE0, 0x00, 0x08, // jr
        0x27, 0xBD, 0x02, 0x40, //  addiu
    ];
    let rom = Rom::new(0x001050);
    let vram = Vram::new(0x8081CE54);
    let size = Size::new(bytes.len() as u32);

    let global_config = GlobalConfigBuilder::new(Endian::Big).build();
    let global_ranges = RomVramRange::new(
        AddressRange::new(rom, rom + size),
        AddressRange::new(vram, vram + size),
    );
    let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    global_segment
        .preheat_text(&global_config, &text_settings, "test", &bytes, rom, vram)
        .unwrap();

    let mut platform_segment = UserSegmentBuilder::new();
    platform_segment.n64_libultra_symbols().unwrap();
    platform_segment.n64_hardware_registers(true, true).unwrap();

    let mut context = ContextBuilder::new(global_segment, platform_segment)
        .build(global_config)
        .unwrap();

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

    assert_eq!(section_text.symbols().len(), 1);

    let symbols = context.global_segment().symbols();
    for s in symbols {
        println!("{:?}", s.1);
    }
    assert_eq!(symbols.len(), 1);

    let labels = context.global_segment().labels();
    for s in labels {
        println!("{:?}", s.1);
    }
    assert_eq!(labels.len(), 20);

    let mut disassembled = ".section .text\n".to_string();
    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    let data_display_settings = SymDataDisplaySettings::new();
    for func in section_text.symbols() {
        let func_display =
            func.display(&context, &function_display_settings, &data_display_settings);
        disassembled.push('\n');
        disassembled.push_str(&func_display.unwrap().to_string());
    }

    println!();
    println!();
    println!();

    println!("{}", disassembled);

    let expected_str = "\
.section .text

/* Automatically generated and unreferenced pad */
glabel func_8081CE54
    /* 001050 8081CE54 27BDFDC0 */  addiu       $sp, $sp, -0x240
    /* 001054 8081CE58 AFB00020 */  sw          $s0, 0x20($sp)
    /* 001058 8081CE5C 00A08025 */  or          $s0, $a1, $zero
    /* 00105C 8081CE60 AFBF0044 */  sw          $ra, 0x44($sp)
    /* 001060 8081CE64 AFBE0040 */  sw          $s8, 0x40($sp)
    /* 001064 8081CE68 AFB7003C */  sw          $s7, 0x3C($sp)
    /* 001068 8081CE6C AFB60038 */  sw          $s6, 0x38($sp)
    /* 00106C 8081CE70 AFB50034 */  sw          $s5, 0x34($sp)
    /* 001070 8081CE74 AFB40030 */  sw          $s4, 0x30($sp)
    /* 001074 8081CE78 AFB3002C */  sw          $s3, 0x2C($sp)
    /* 001078 8081CE7C AFB20028 */  sw          $s2, 0x28($sp)
    /* 00107C 8081CE80 AFB10024 */  sw          $s1, 0x24($sp)
    /* 001080 8081CE84 AFA40240 */  sw          $a0, 0x240($sp)
    /* 001084 8081CE88 8FAE0240 */  lw          $t6, 0x240($sp)
    /* 001088 8081CE8C 3C010001 */  lui         $at, (0x10760 >> 16)
    /* 00108C 8081CE90 34210760 */  ori         $at, $at, (0x10760 & 0xFFFF)
    /* 001090 8081CE94 01C19021 */  addu        $s2, $t6, $at
    /* 001094 8081CE98 02401825 */  or          $v1, $s2, $zero
    /* 001098 8081CE9C 3C0F0001 */  lui         $t7, %hi(UNK_00010934)
    /* 00109C 8081CEA0 01EE7821 */  addu        $t7, $t7, $t6
    /* 0010A0 8081CEA4 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0010A4 8081CEA8 95EF0934 */  lhu         $t7, %lo(UNK_00010934)($t7)
    /* 0010A8 8081CEAC 24010006 */  addiu       $at, $zero, 0x6
    /* 0010AC 8081CEB0 55E100C0 */  bnel        $t7, $at, .L8081D1B4
    /* 0010B0 8081CEB4 3C07E700 */   lui        $a3, (0xE7000000 >> 16)
    /* 0010B4 8081CEB8 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0010B8 8081CEBC 965801E4 */  lhu         $t8, 0x1E4($s2)
    /* 0010BC 8081CEC0 570000BC */  bnezl       $t8, .L8081D1B4
    /* 0010C0 8081CEC4 3C07E700 */   lui        $a3, (0xE7000000 >> 16)
    /* 0010C4 8081CEC8 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0010C8 8081CECC 965901E8 */  lhu         $t9, 0x1E8($s2)
    /* 0010CC 8081CED0 24010001 */  addiu       $at, $zero, 0x1
    /* 0010D0 8081CED4 572100B7 */  bnel        $t9, $at, .L8081D1B4
    /* 0010D4 8081CED8 3C07E700 */   lui        $a3, (0xE7000000 >> 16)
    /* 0010D8 8081CEDC 86420220 */  lh          $v0, 0x220($s2)
    /* 0010DC 8081CEE0 86440238 */  lh          $a0, 0x238($s2)
    /* 0010E0 8081CEE4 A6400260 */  sh          $zero, 0x260($s2)
    /* 0010E4 8081CEE8 00029C00 */  sll         $s3, $v0, 16
    /* 0010E8 8081CEEC 14800040 */  bnez        $a0, .L8081CFF0
    /* 0010EC 8081CEF0 00139C03 */   sra        $s3, $s3, 16
    /* 0010F0 8081CEF4 86440214 */  lh          $a0, 0x214($s2)
    /* 0010F4 8081CEF8 240F000B */  addiu       $t7, $zero, 0xB
    /* 0010F8 8081CEFC 2405000B */  addiu       $a1, $zero, 0xB
    /* 0010FC 8081CF00 2881001F */  slti        $at, $a0, 0x1F
    /* 001100 8081CF04 14200014 */  bnez        $at, .L8081CF58
    /* 001104 8081CF08 3C018083 */   lui        $at, %hi(UNK_808299B0)
    /* 001108 8081CF0C A42099B0 */  sh          $zero, %lo(UNK_808299B0)($at)
    /* 00110C 8081CF10 244E0001 */  addiu       $t6, $v0, 0x1
  .L8081CF14:
    /* 001110 8081CF14 A64E0220 */  sh          $t6, 0x220($s2)
    /* 001114 8081CF18 86420220 */  lh          $v0, 0x220($s2)
    /* 001118 8081CF1C 2841000C */  slti        $at, $v0, 0xC
    /* 00111C 8081CF20 54200007 */  bnezl       $at, .L8081CF40
    /* 001120 8081CF24 84780220 */   lh         $t8, 0x220($v1)
    /* 001124 8081CF28 A64F0220 */  sh          $t7, 0x220($s2)
    /* 001128 8081CF2C 0C207C61 */  jal         UNK_func_8081F184
    /* 00112C 8081CF30 8FA40240 */   lw         $a0, 0x240($sp)
    /* 001130 8081CF34 10000006 */  b           .L8081CF50
    /* 001134 8081CF38 86420220 */   lh         $v0, 0x220($s2)
    /* 001138 8081CF3C 84780220 */  lh          $t8, 0x220($v1)
  .L8081CF40:
    /* 00113C 8081CF40 0078C821 */  addu        $t9, $v1, $t8
    /* 001140 8081CF44 932E0266 */  lbu         $t6, 0x266($t9)
    /* 001144 8081CF48 51C0FFF2 */  beqzl       $t6, .L8081CF14
    /* 001148 8081CF4C 244E0001 */   addiu      $t6, $v0, 0x1
  .L8081CF50:
    /* 00114C 8081CF50 1000001D */  b           .L8081CFC8
    /* 001150 8081CF54 AFB2004C */   sw         $s2, 0x4C($sp)
  .L8081CF58:
    /* 001154 8081CF58 2881FFE2 */  slti        $at, $a0, -0x1E
    /* 001158 8081CF5C 10200015 */  beqz        $at, .L8081CFB4
    /* 00115C 8081CF60 3C0F8083 */   lui        $t7, %hi(UNK_808299B0)
    /* 001160 8081CF64 3C018083 */  lui         $at, %hi(UNK_808299B0)
    /* 001164 8081CF68 A42099B0 */  sh          $zero, %lo(UNK_808299B0)($at)
    /* 001168 8081CF6C 244FFFFF */  addiu       $t7, $v0, -0x1
  .L8081CF70:
    /* 00116C 8081CF70 A64F0220 */  sh          $t7, 0x220($s2)
    /* 001170 8081CF74 86420220 */  lh          $v0, 0x220($s2)
    /* 001174 8081CF78 04430008 */  bgezl       $v0, .L8081CF9C
    /* 001178 8081CF7C 84780220 */   lh         $t8, 0x220($v1)
    /* 00117C 8081CF80 A6400220 */  sh          $zero, 0x220($s2)
    /* 001180 8081CF84 8FA40240 */  lw          $a0, 0x240($sp)
    /* 001184 8081CF88 0C207C61 */  jal         UNK_func_8081F184
    /* 001188 8081CF8C 2405000A */   addiu      $a1, $zero, 0xA
    /* 00118C 8081CF90 10000006 */  b           .L8081CFAC
    /* 001190 8081CF94 86420220 */   lh         $v0, 0x220($s2)
    /* 001194 8081CF98 84780220 */  lh          $t8, 0x220($v1)
  .L8081CF9C:
    /* 001198 8081CF9C 0078C821 */  addu        $t9, $v1, $t8
    /* 00119C 8081CFA0 932E0266 */  lbu         $t6, 0x266($t9)
    /* 0011A0 8081CFA4 51C0FFF2 */  beqzl       $t6, .L8081CF70
    /* 0011A4 8081CFA8 244FFFFF */   addiu      $t7, $v0, -0x1
  .L8081CFAC:
    /* 0011A8 8081CFAC 10000006 */  b           .L8081CFC8
    /* 0011AC 8081CFB0 AFB2004C */   sw         $s2, 0x4C($sp)
  .L8081CFB4:
    /* 0011B0 8081CFB4 95EF99B0 */  lhu         $t7, %lo(UNK_808299B0)($t7)
    /* 0011B4 8081CFB8 3C018083 */  lui         $at, %hi(UNK_808299B0)
    /* 0011B8 8081CFBC AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0011BC 8081CFC0 25F80001 */  addiu       $t8, $t7, 0x1
    /* 0011C0 8081CFC4 A43899B0 */  sh          $t8, %lo(UNK_808299B0)($at)
  .L8081CFC8:
    /* 0011C4 8081CFC8 8FA4004C */  lw          $a0, 0x4C($sp)
    /* 0011C8 8081CFCC 2445001F */  addiu       $a1, $v0, 0x1F
    /* 0011CC 8081CFD0 A4850248 */  sh          $a1, 0x248($a0)
    /* 0011D0 8081CFD4 00052880 */  sll         $a1, $a1, 2
    /* 0011D4 8081CFD8 30A5FFFF */  andi        $a1, $a1, 0xFFFF
    /* 0011D8 8081CFDC A4820240 */  sh          $v0, 0x240($a0)
    /* 0011DC 8081CFE0 0C2066FD */  jal         UNK_func_80819BF4
    /* 0011E0 8081CFE4 8C86014C */   lw         $a2, 0x14C($a0)
    /* 0011E4 8081CFE8 1000005D */  b           .L8081D160
    /* 0011E8 8081CFEC 8FB1004C */   lw         $s1, 0x4C($sp)
  .L8081CFF0:
    /* 0011EC 8081CFF0 3C0E8012 */  lui         $t6, %hi(UNK_8011B9F0)
    /* 0011F0 8081CFF4 85CEB9F0 */  lh          $t6, %lo(UNK_8011B9F0)($t6)
    /* 0011F4 8081CFF8 2401000A */  addiu       $at, $zero, 0xA
    /* 0011F8 8081CFFC 25CF0018 */  addiu       $t7, $t6, 0x18
    /* 0011FC 8081D000 1481002C */  bne         $a0, $at, .L8081D0B4
    /* 001200 8081D004 A64F0240 */   sh         $t7, 0x240($s2)
    /* 001204 8081D008 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 001208 8081D00C 86580214 */  lh          $t8, 0x214($s2)
    /* 00120C 8081D010 2B01001F */  slti        $at, $t8, 0x1F
    /* 001210 8081D014 14200025 */  bnez        $at, .L8081D0AC
    /* 001214 8081D018 00000000 */   nop
    /* 001218 8081D01C A6400220 */  sh          $zero, 0x220($s2)
    /* 00121C 8081D020 86420220 */  lh          $v0, 0x220($s2)
    /* 001220 8081D024 A6400238 */  sh          $zero, 0x238($s2)
    /* 001224 8081D028 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 001228 8081D02C 0242C821 */  addu        $t9, $s2, $v0
    /* 00122C 8081D030 932E0266 */  lbu         $t6, 0x266($t9)
    /* 001230 8081D034 15C00008 */  bnez        $t6, .L8081D058
    /* 001234 8081D038 244F0001 */   addiu      $t7, $v0, 0x1
  .L8081D03C:
    /* 001238 8081D03C A64F0220 */  sh          $t7, 0x220($s2)
    /* 00123C 8081D040 86420220 */  lh          $v0, 0x220($s2)
    /* 001240 8081D044 0242C021 */  addu        $t8, $s2, $v0
    /* 001244 8081D048 93190266 */  lbu         $t9, 0x266($t8)
    /* 001248 8081D04C 5320FFFB */  beqzl       $t9, .L8081D03C
    /* 00124C 8081D050 244F0001 */   addiu      $t7, $v0, 0x1
    /* 001250 8081D054 AFB2004C */  sw          $s2, 0x4C($sp)
  .L8081D058:
    /* 001254 8081D058 8FA4004C */  lw          $a0, 0x4C($sp)
    /* 001258 8081D05C 2445001F */  addiu       $a1, $v0, 0x1F
    /* 00125C 8081D060 A4850248 */  sh          $a1, 0x248($a0)
    /* 001260 8081D064 00052880 */  sll         $a1, $a1, 2
    /* 001264 8081D068 30A5FFFF */  andi        $a1, $a1, 0xFFFF
    /* 001268 8081D06C A4820240 */  sh          $v0, 0x240($a0)
    /* 00126C 8081D070 0C2066FD */  jal         UNK_func_80819BF4
    /* 001270 8081D074 8C86014C */   lw         $a2, 0x14C($a0)
    /* 001274 8081D078 3C128010 */  lui         $s2, %hi(UNK_801043A0)
    /* 001278 8081D07C 3C0F8010 */  lui         $t7, %hi(UNK_801043A8)
    /* 00127C 8081D080 264743A0 */  addiu       $a3, $s2, %lo(UNK_801043A0)
    /* 001280 8081D084 25EF43A8 */  addiu       $t7, $t7, %lo(UNK_801043A8)
    /* 001284 8081D088 3C058010 */  lui         $a1, %hi(UNK_80104394)
    /* 001288 8081D08C 24A54394 */  addiu       $a1, $a1, %lo(UNK_80104394)
    /* 00128C 8081D090 AFAF0014 */  sw          $t7, 0x14($sp)
    /* 001290 8081D094 AFA70010 */  sw          $a3, 0x10($sp)
    /* 001294 8081D098 24044809 */  addiu       $a0, $zero, 0x4809
    /* 001298 8081D09C 0C03201B */  jal         UNK_func_800C806C
    /* 00129C 8081D0A0 24060004 */   addiu      $a2, $zero, 0x4
    /* 0012A0 8081D0A4 3C018083 */  lui         $at, %hi(UNK_808299B0)
    /* 0012A4 8081D0A8 A42099B0 */  sh          $zero, %lo(UNK_808299B0)($at)
  .L8081D0AC:
    /* 0012A8 8081D0AC 1000002C */  b           .L8081D160
    /* 0012AC 8081D0B0 8FB1004C */   lw         $s1, 0x4C($sp)
  .L8081D0B4:
    /* 0012B0 8081D0B4 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0012B4 8081D0B8 86580214 */  lh          $t8, 0x214($s2)
    /* 0012B8 8081D0BC 2419000B */  addiu       $t9, $zero, 0xB
    /* 0012BC 8081D0C0 2B01FFE2 */  slti        $at, $t8, -0x1E
    /* 0012C0 8081D0C4 50200026 */  beqzl       $at, .L8081D160
    /* 0012C4 8081D0C8 8FB1004C */   lw         $s1, 0x4C($sp)
    /* 0012C8 8081D0CC A6590220 */  sh          $t9, 0x220($s2)
    /* 0012CC 8081D0D0 86420220 */  lh          $v0, 0x220($s2)
    /* 0012D0 8081D0D4 A6400238 */  sh          $zero, 0x238($s2)
    /* 0012D4 8081D0D8 AFB2004C */  sw          $s2, 0x4C($sp)
    /* 0012D8 8081D0DC 02427021 */  addu        $t6, $s2, $v0
    /* 0012DC 8081D0E0 91CF0266 */  lbu         $t7, 0x266($t6)
    /* 0012E0 8081D0E4 15E00008 */  bnez        $t7, .L8081D108
    /* 0012E4 8081D0E8 2458FFFF */   addiu      $t8, $v0, -0x1
  .L8081D0EC:
    /* 0012E8 8081D0EC A6580220 */  sh          $t8, 0x220($s2)
    /* 0012EC 8081D0F0 86420220 */  lh          $v0, 0x220($s2)
    /* 0012F0 8081D0F4 0242C821 */  addu        $t9, $s2, $v0
    /* 0012F4 8081D0F8 932E0266 */  lbu         $t6, 0x266($t9)
    /* 0012F8 8081D0FC 51C0FFFB */  beqzl       $t6, .L8081D0EC
    /* 0012FC 8081D100 2458FFFF */   addiu      $t8, $v0, -0x1
    /* 001300 8081D104 AFB2004C */  sw          $s2, 0x4C($sp)
  .L8081D108:
    /* 001304 8081D108 8FA4004C */  lw          $a0, 0x4C($sp)
    /* 001308 8081D10C 2445001F */  addiu       $a1, $v0, 0x1F
    /* 00130C 8081D110 A4850248 */  sh          $a1, 0x248($a0)
    /* 001310 8081D114 00052880 */  sll         $a1, $a1, 2
    /* 001314 8081D118 30A5FFFF */  andi        $a1, $a1, 0xFFFF
    /* 001318 8081D11C A4820240 */  sh          $v0, 0x240($a0)
    /* 00131C 8081D120 0C2066FD */  jal         UNK_func_80819BF4
    /* 001320 8081D124 8C86014C */   lw         $a2, 0x14C($a0)
    /* 001324 8081D128 3C128010 */  lui         $s2, %hi(UNK_801043A0)
    /* 001328 8081D12C 3C188010 */  lui         $t8, %hi(UNK_801043A8)
    /* 00132C 8081D130 264743A0 */  addiu       $a3, $s2, %lo(UNK_801043A0)
    /* 001330 8081D134 271843A8 */  addiu       $t8, $t8, %lo(UNK_801043A8)
    /* 001334 8081D138 3C058010 */  lui         $a1, %hi(UNK_80104394)
    /* 001338 8081D13C 24A54394 */  addiu       $a1, $a1, %lo(UNK_80104394)
    /* 00133C 8081D140 AFB80014 */  sw          $t8, 0x14($sp)
    /* 001340 8081D144 AFA70010 */  sw          $a3, 0x10($sp)
    /* 001344 8081D148 24044809 */  addiu       $a0, $zero, 0x4809
    /* 001348 8081D14C 0C03201B */  jal         UNK_func_800C806C
    /* 00134C 8081D150 24060004 */   addiu      $a2, $zero, 0x4
    /* 001350 8081D154 3C018083 */  lui         $at, %hi(UNK_808299B0)
    /* 001354 8081D158 A42099B0 */  sh          $zero, %lo(UNK_808299B0)($at)
    /* 001358 8081D15C 8FB1004C */  lw          $s1, 0x4C($sp)
  .L8081D160:
    /* 00135C 8081D160 3C128010 */  lui         $s2, %hi(UNK_801043A0)
    /* 001360 8081D164 3C058010 */  lui         $a1, %hi(UNK_80104394)
    /* 001364 8081D168 86220220 */  lh          $v0, 0x220($s1)
    /* 001368 8081D16C 265243A0 */  addiu       $s2, $s2, %lo(UNK_801043A0)
    /* 00136C 8081D170 240F03E7 */  addiu       $t7, $zero, 0x3E7
    /* 001370 8081D174 0222C821 */  addu        $t9, $s1, $v0
    /* 001374 8081D178 932E0266 */  lbu         $t6, 0x266($t9)
    /* 001378 8081D17C 24A54394 */  addiu       $a1, $a1, %lo(UNK_80104394)
    /* 00137C 8081D180 24044809 */  addiu       $a0, $zero, 0x4809
    /* 001380 8081D184 15C00003 */  bnez        $t6, .L8081D194
    /* 001384 8081D188 24060004 */   addiu      $a2, $zero, 0x4
    /* 001388 8081D18C A62F0240 */  sh          $t7, 0x240($s1)
    /* 00138C 8081D190 86220220 */  lh          $v0, 0x220($s1)
  .L8081D194:
    /* 001390 8081D194 12620006 */  beq         $s3, $v0, .L8081D1B0
    /* 001394 8081D198 02403825 */   or         $a3, $s2, $zero
    /* 001398 8081D19C 3C188010 */  lui         $t8, %hi(UNK_801043A8)
    /* 00139C 8081D1A0 271843A8 */  addiu       $t8, $t8, %lo(UNK_801043A8)
    /* 0013A0 8081D1A4 AFB80014 */  sw          $t8, 0x14($sp)
    /* 0013A4 8081D1A8 0C03201B */  jal         UNK_func_800C806C
    /* 0013A8 8081D1AC AFB20010 */   sw         $s2, 0x10($sp)
  .L8081D1B0:
    /* 0013AC 8081D1B0 8FBF0044 */  lw          $ra, 0x44($sp)
  .L8081D1B4:
    /* 0013B0 8081D1B4 8FB00020 */  lw          $s0, 0x20($sp)
    /* 0013B4 8081D1B8 8FB10024 */  lw          $s1, 0x24($sp)
    /* 0013B8 8081D1BC 8FB20028 */  lw          $s2, 0x28($sp)
    /* 0013BC 8081D1C0 8FB3002C */  lw          $s3, 0x2C($sp)
    /* 0013C0 8081D1C4 8FB40030 */  lw          $s4, 0x30($sp)
    /* 0013C4 8081D1C8 8FB50034 */  lw          $s5, 0x34($sp)
    /* 0013C8 8081D1CC 8FB60038 */  lw          $s6, 0x38($sp)
    /* 0013CC 8081D1D0 8FB7003C */  lw          $s7, 0x3C($sp)
    /* 0013D0 8081D1D4 8FBE0040 */  lw          $s8, 0x40($sp)
    /* 0013D4 8081D1D8 03E00008 */  jr          $ra
    /* 0013D8 8081D1DC 27BD0240 */   addiu      $sp, $sp, 0x240
.size func_8081CE54, . - func_8081CE54
";

    assert_eq!(disassembled, expected_str);
    // None::<u32>.unwrap();
}

#[test]
fn weird_case_use_gp_as_temp() {
    static BYTES: [u8; 60] = [
        // func_80000000
        0x07, 0x80, 0x01, 0x3C, // lui     at,0x0
        0xD8, 0x7D, 0x21, 0x24, // addiu   at,at,0
        0x20, 0x00, 0x3C, 0xAC, // sw      gp,32(at)
        0x2C, 0x00, 0x3F, 0xAC, // sw      ra,44(at)
        0x00, 0x00, 0x9C, 0x20, // addi    gp,a0,0
        0x40, 0x00, 0x84, 0x20, // addi    a0,a0,64
        0x00, 0x00, 0x84, 0xAF, // sw      a0,0(gp)
        0x04, 0x00, 0x84, 0xAF, // sw      a0,4(gp)
        0x08, 0x00, 0x84, 0xAF, // sw      a0,8(gp)
        0x07, 0x80, 0x01, 0x3C, // lui     at,0x0
        0xD8, 0x7D, 0x21, 0x24, // addiu   at,at,0
        0x2C, 0x00, 0x3F, 0x8C, // lw      ra,44(at)
        0x20, 0x00, 0x3C, 0x8C, // lw      gp,32(at)
        0x08, 0x00, 0xE0, 0x03, // jr      ra
        0x00, 0x00, 0x00, 0x00, // nop
    ];

    let rom = Rom::new(0x0003EBC8);
    let vram = Vram::new(0x8004E3C8);

    let segment_rom = Rom::new(0x00000000);
    let segment_vram = Vram::new(0x80000000);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::PSYQ),
        InstructionFlags::new_extension(IsaExtension::R3000GTE),
    );

    let mut context = {
        let global_config = GlobalConfigBuilder::new(Endian::Little)
            .with_gp_config(Some(GpConfig::new_sdata(GpValue::new(0x80075264))))
            .build();

        let global_ranges = RomVramRange::new(
            AddressRange::new(segment_rom, Rom::new(0x0003F3A4)),
            AddressRange::new(segment_vram, Vram::new(0x8004EBA4)),
        );
        let mut global_segment = GlobalSegmentBuilder::new(global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, "text", &BYTES, rom, vram)
            .unwrap();

        let platform_segment = UserSegmentBuilder::new();

        let builder = ContextBuilder::new(global_segment, platform_segment);

        builder.build(global_config).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text",
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
    let data_display_settings = SymDataDisplaySettings::new();
    for sym in section_text.symbols() {
        disassembly.push('\n');
        disassembly.push_str(
            &sym.display(&context, &display_settings, &data_display_settings)
                .unwrap()
                .to_string(),
        );
    }

    println!("{}", disassembly);

    let expected_disassembly = "\
.section .text

/* Handwritten function */
/* Automatically generated and unreferenced pad */
glabel func_8004E3C8
    /* 03EBC8 8004E3C8 0780013C */  lui         $at, %hi(UNK_80077DD8)
    /* 03EBCC 8004E3CC D87D2124 */  addiu       $at, $at, %lo(UNK_80077DD8)
    /* 03EBD0 8004E3D0 20003CAC */  sw          $gp, 0x20($at)
    /* 03EBD4 8004E3D4 2C003FAC */  sw          $ra, 0x2C($at)
    /* 03EBD8 8004E3D8 00009C20 */  addi        $gp, $a0, 0x0 /* handwritten instruction */
    /* 03EBDC 8004E3DC 40008420 */  addi        $a0, $a0, 0x40 /* handwritten instruction */
    /* 03EBE0 8004E3E0 000084AF */  sw          $a0, 0x0($gp)
    /* 03EBE4 8004E3E4 040084AF */  sw          $a0, 0x4($gp)
    /* 03EBE8 8004E3E8 080084AF */  sw          $a0, 0x8($gp)
    /* 03EBEC 8004E3EC 0780013C */  lui         $at, %hi(UNK_80077DD8)
    /* 03EBF0 8004E3F0 D87D2124 */  addiu       $at, $at, %lo(UNK_80077DD8)
    /* 03EBF4 8004E3F4 2C003F8C */  lw          $ra, 0x2C($at)
    /* 03EBF8 8004E3F8 20003C8C */  lw          $gp, 0x20($at)
    /* 03EBFC 8004E3FC 0800E003 */  jr          $ra
    /* 03EC00 8004E400 00000000 */   nop
.size func_8004E3C8, . - func_8004E3C8
";

    assert_eq!(disassembly, expected_disassembly,);
}
