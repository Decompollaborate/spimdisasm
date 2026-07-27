/* SPDX-FileCopyrightText: © 2024-2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use address_space::{AddressRange, GpValue, Rom, RomVramRange, Size, Vram};
use pretty_assertions::assert_eq;
use rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaExtension, IsaVersion};
use spimdisasm::{
    config::{Compiler, Endian, GlobalConfigBuilder, GpConfig},
    context::{builder::AbsoluteSegmentBuilder, ContextBuilder, GlobalSegmentBuilder},
    relocation::UserRelocs,
    sections::before_proc::ExecutableSectionSettings,
    segments::ParentSegmentInfo,
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
    let global_ranges = RomVramRange::new_option(
        AddressRange::new_size(rom, size),
        AddressRange::new_size(vram, size),
        4,
    )
    .unwrap();
    let mut global_segment = GlobalSegmentBuilder::new("segment", global_ranges).finish_symbols();

    let text_settings =
        ExecutableSectionSettings::new(None, InstructionFlags::new(IsaVersion::MIPS_III));

    global_segment
        .preheat_text(&global_config, &text_settings, "test", &bytes, rom, vram)
        .unwrap();

    let mut context_builder = ContextBuilder::new();
    context_builder.add_global_segment(global_segment).unwrap();

    let mut absolute_segment = AbsoluteSegmentBuilder::new();
    absolute_segment.n64_libultra_symbols().unwrap();
    absolute_segment.n64_hardware_registers(true, true).unwrap();

    let mut context = context_builder
        .build(global_config, absolute_segment)
        .unwrap();

    let instr_display_flags = InstructionDisplayFlags::default();

    let section_text = context
        .create_section_text(
            &text_settings,
            "test",
            bytes.to_vec(),
            rom,
            vram,
            ParentSegmentInfo::new(rom, vram, None),
        )
        .unwrap();

    let user_relocs = UserRelocs::new();
    let section_text = section_text
        .post_process(&mut context, &user_relocs)
        .unwrap();

    assert_eq!(section_text.symbols().len(), 1);

    for segment in context.segments().global_segments() {
        let symbols = segment.symbols();
        for s in symbols {
            println!("{:?}", s.1);
        }
        assert_eq!(symbols.len(), 1);

        let labels = segment.labels();
        for s in labels {
            println!("{:?}", s.1);
        }
        assert_eq!(labels.len(), 20);
    }

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

    println!("{disassembled}");

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

        let global_ranges = RomVramRange::new_option(
            AddressRange::new(segment_rom, Rom::new(0x0003F3A4)),
            AddressRange::new(segment_vram, Vram::new(0x8004EBA4)),
            4,
        )
        .unwrap();
        let mut global_segment =
            GlobalSegmentBuilder::new("segment", global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, "text", &BYTES, rom, vram)
            .unwrap();

        let mut builder = ContextBuilder::new();
        builder.add_global_segment(global_segment).unwrap();

        let absolute_segment = AbsoluteSegmentBuilder::new();

        builder.build(global_config, absolute_segment).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text",
            BYTES.to_vec(),
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = UserRelocs::new();
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

    println!("{disassembly}");

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

#[test]
fn weird_case_load_address_at_gp() {
    static BYTES: [u8; 1264] = [
        // func_802AE370
        0x27, 0xBD, 0xFF, 0xC8, // addiu
        0xFF, 0xA9, 0x00, 0x10, // sd
        0x3C, 0x09, 0x80, 0x3F, // lui
        0xFF, 0xBF, 0x00, 0x00, // sd
        0xFF, 0xA8, 0x00, 0x08, // sd
        0xFF, 0xAA, 0x00, 0x18, // sd
        0xFF, 0xAB, 0x00, 0x20, // sd
        0xFF, 0xAC, 0x00, 0x28, // sd
        0xFF, 0xAD, 0x00, 0x30, // sd
        0x25, 0x29, 0xD8, 0x18, // addiu
        0x3C, 0x02, 0x80, 0x36, // lui
        0xAD, 0x32, 0x00, 0x00, // sw
        0x24, 0x42, 0x80, 0x70, // addiu
        0x8C, 0x43, 0x00, 0x00, // lw
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD8, 0x2C, // addiu
        0xAC, 0x83, 0x00, 0x00, // sw
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD8, 0x30, // addiu
        0x20, 0x63, 0x0C, 0x80, // addi
        0xAC, 0x83, 0x00, 0x00, // sw
        0x20, 0x63, 0x0C, 0x80, // addi
        0xAC, 0x43, 0x00, 0x00, // sw
        0x3C, 0x02, 0x80, 0x3F, // lui
        0x3C, 0x03, 0x80, 0x3F, // lui
        0x24, 0x42, 0xD8, 0x2C, // addiu
        0x24, 0x63, 0xD8, 0x30, // addiu
        0x8C, 0x42, 0x00, 0x00, // lw
        0x8C, 0x63, 0x00, 0x00, // lw
        0x24, 0x04, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x84, 0xE2, // jal
        0x24, 0x05, 0x00, 0x00, // addiu
        0x3C, 0x1C, 0x80, 0x3F, // lui
        0x0C, 0x0A, 0x9D, 0x53, // jal
        0x27, 0x9C, 0xD7, 0x60, // addiu
        0x24, 0x09, 0x00, 0x1E, // addiu
        0xA7, 0x89, 0x00, 0x52, // sh
        0xA7, 0x89, 0x00, 0x54, // sh
        0x24, 0x09, 0xFF, 0xE2, // addiu
        0xA7, 0x89, 0x00, 0x56, // sh
        0x24, 0x09, 0x00, 0x1E, // addiu
        0xA7, 0x89, 0x00, 0x58, // sh
        0xA7, 0x89, 0x00, 0x5A, // sh
        0x24, 0x09, 0xFF, 0xE2, // addiu
        0xA7, 0x89, 0x00, 0x5C, // sh
        0x24, 0x09, 0x00, 0x23, // addiu
        0xA7, 0x89, 0x00, 0x5E, // sh
        0xA7, 0x89, 0x00, 0x60, // sh
        0x24, 0x09, 0xFF, 0xDD, // addiu
        0xA7, 0x89, 0x00, 0x62, // sh
        0x24, 0x09, 0x00, 0x23, // addiu
        0xA7, 0x89, 0x00, 0x64, // sh
        0xA7, 0x89, 0x00, 0x66, // sh
        0x24, 0x09, 0xFF, 0xDD, // addiu
        0x3C, 0x08, 0x80, 0x3F, // lui
        0xA7, 0x89, 0x00, 0x68, // sh
        0x25, 0x08, 0xD8, 0x08, // addiu
        0xAD, 0x0F, 0x00, 0x00, // sw
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x25, 0x08, 0xD8, 0x0C, // addiu
        0xAD, 0x13, 0x00, 0x00, // sw
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x25, 0x08, 0xD8, 0x10, // addiu
        0xAD, 0x10, 0x00, 0x00, // sw
        0x23, 0x94, 0x00, 0x4C, // addi
        0xA6, 0x91, 0x00, 0x00, // sh
        0xA7, 0x91, 0x00, 0x4E, // sh
        0xA7, 0x91, 0x00, 0x74, // sh
        0x3C, 0x12, 0x80, 0x3F, // lui
        0x24, 0x11, 0x00, 0x01, // addiu
        0x26, 0x52, 0xD8, 0x25, // addiu
        0xA2, 0x51, 0x00, 0x00, // sb
        0x3C, 0x01, 0x80, 0x3F, // lui
        0xA0, 0x20, 0xD8, 0x26, // sb
        0x3C, 0x01, 0x80, 0x3F, // lui
        0xA0, 0x20, 0xD8, 0x27, // sb
        0x3C, 0x01, 0x80, 0x3F, // lui
        0xA0, 0x20, 0xD8, 0x24, // sb
        0x24, 0x11, 0x00, 0x00, // addiu
        0x3C, 0x12, 0x80, 0x3F, // lui
        0xA3, 0x91, 0x00, 0xA1, // sb
        0x26, 0x52, 0xD8, 0x0C, // addiu
        0x8E, 0x4A, 0x00, 0x00, // lw
        0x23, 0x91, 0x00, 0x04, // addi
        0x23, 0x83, 0x00, 0x52, // addi
        0x0C, 0x0A, 0xA6, 0x4B, // jal
        0x24, 0x18, 0x00, 0x00, // addiu
        0x3C, 0x0A, 0x80, 0x3F, // lui
        0x3C, 0x16, 0x80, 0x3F, // lui
        0x3C, 0x17, 0x80, 0x3F, // lui
        0x25, 0x4A, 0xD8, 0x18, // addiu
        0x26, 0xD6, 0xD8, 0x2C, // addiu
        0x26, 0xF7, 0xD8, 0x30, // addiu
        0x8D, 0x4A, 0x00, 0x00, // lw
        0x8E, 0xD6, 0x00, 0x00, // lw
        0x8E, 0xF7, 0x00, 0x00, // lw
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x25, 0x08, 0xD4, 0x60, // addiu
        0x01, 0x40, 0x48, 0x25, // or
        0x02, 0xC0, 0x18, 0x25, // or
        0x0C, 0x0A, 0x7E, 0x17, // jal
        0x02, 0xE0, 0x20, 0x25, // or
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x80, 0xE7, // jal
        0x24, 0x03, 0x00, 0x64, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x80, 0xF5, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x81, 0x03, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x44, 0x80, 0x00, 0x00, // mtc1
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x81, 0x20, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x80, 0xA4, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x25, 0x08, 0xD4, 0x60, // addiu
        0x02, 0xC0, 0x10, 0x25, // or
        0x0C, 0x0A, 0x79, 0x56, // jal
        0x02, 0xE0, 0x18, 0x25, // or
        0x3C, 0x03, 0x80, 0x3F, // lui
        0x24, 0x63, 0xD4, 0x60, // addiu
        0x0C, 0x0A, 0x80, 0xC8, // jal
        0x24, 0x02, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x00, // addiu
        0x0C, 0x0A, 0x80, 0xA4, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x25, 0x08, 0xD4, 0x60, // addiu
        0x02, 0xE0, 0x10, 0x25, // or
        0x0C, 0x0A, 0x79, 0x56, // jal
        0x02, 0xC0, 0x18, 0x25, // or
        0x24, 0x09, 0xFF, 0xD8, // addiu
        0xA7, 0x89, 0x00, 0x78, // sh
        0x24, 0x09, 0x00, 0x00, // addiu
        0xA7, 0x89, 0x00, 0x7A, // sh
        0x24, 0x09, 0x00, 0x02, // addiu
        0xA7, 0x89, 0x00, 0x7C, // sh
        0x24, 0x09, 0x00, 0x00, // addiu
        0xA7, 0x89, 0x00, 0x7E, // sh
        0x24, 0x09, 0x00, 0x28, // addiu
        0xA7, 0x89, 0x00, 0x80, // sh
        0x24, 0x09, 0x00, 0x02, // addiu
        0xA7, 0x89, 0x00, 0x82, // sh
        0x24, 0x09, 0x00, 0x28, // addiu
        0xA7, 0x89, 0x00, 0x84, // sh
        0x24, 0x09, 0x00, 0x3C, // addiu
        0xA7, 0x89, 0x00, 0x86, // sh
        0x24, 0x09, 0x00, 0x02, // addiu
        0xA7, 0x89, 0x00, 0x88, // sh
        0x24, 0x09, 0x00, 0x3C, // addiu
        0xA7, 0x89, 0x00, 0x8A, // sh
        0x24, 0x09, 0x00, 0x50, // addiu
        0xA7, 0x89, 0x00, 0x8C, // sh
        0x24, 0x09, 0x00, 0x02, // addiu
        0xA7, 0x89, 0x00, 0x8E, // sh
        0x24, 0x09, 0x00, 0x50, // addiu
        0xA7, 0x89, 0x00, 0x90, // sh
        0x24, 0x09, 0x00, 0x64, // addiu
        0xA7, 0x89, 0x00, 0x92, // sh
        0x24, 0x09, 0x00, 0x02, // addiu
        0x3C, 0x0B, 0x80, 0x3F, // lui
        0xA7, 0x89, 0x00, 0x94, // sh
        0x25, 0x6B, 0xD8, 0x18, // addiu
        0x8D, 0x6B, 0x00, 0x00, // lw
        0x24, 0x08, 0x00, 0x00, // addiu
        0x8D, 0x69, 0x00, 0x04, // lw
        0x8D, 0x6A, 0x00, 0x08, // lw
        0x01, 0x2B, 0x48, 0x20, // add
        0x01, 0x4B, 0x50, 0x20, // add
        0x0C, 0x0A, 0x70, 0xD5, // jal
        0x24, 0x0B, 0x4E, 0x20, // addiu
        0x27, 0xBD, 0xFF, 0xC0, // addiu
        0x24, 0x04, 0x00, 0x00, // addiu
        0x24, 0x05, 0x00, 0x28, // addiu
        0x24, 0x06, 0x00, 0x0F, // addiu
        0x0C, 0x09, 0x60, 0x8C, // jal
        0x24, 0x07, 0x00, 0x0F, // addiu
        0x24, 0x09, 0x00, 0x01, // addiu
        0x23, 0xBD, 0x00, 0x40, // addi
        0x0C, 0x0A, 0xBB, 0xB2, // jal
        0xA3, 0x89, 0x00, 0x9A, // sb
        0x3C, 0x0B, 0x80, 0x3F, // lui
        0xA3, 0x80, 0x00, 0x9A, // sb
        0x25, 0x6B, 0xD8, 0x18, // addiu
        0x8D, 0x6B, 0x00, 0x00, // lw
        0x3C, 0x08, 0x80, 0x3F, // lui
        0x3C, 0x09, 0x80, 0x3F, // lui
        0x8D, 0x6A, 0x00, 0x18, // lw
        0x25, 0x08, 0xD8, 0x30, // addiu
        0x25, 0x29, 0xD8, 0x2C, // addiu
        0x01, 0x4B, 0x50, 0x20, // add
        0x8D, 0x08, 0x00, 0x00, // lw
        0x8D, 0x29, 0x00, 0x00, // lw
        0x0C, 0x0A, 0xAA, 0x0E, // jal
        0x8D, 0x4A, 0x00, 0x04, // lw
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x01, // addiu
        0x0C, 0x0A, 0x80, 0xE7, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x01, // addiu
        0x0C, 0x0A, 0x80, 0xF5, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x01, // addiu
        0x0C, 0x0A, 0x81, 0x03, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x01, 0x3F, 0x00, // lui
        0x44, 0x81, 0x00, 0x00, // mtc1
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x01, // addiu
        0x0C, 0x0A, 0x81, 0x20, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x02, // addiu
        0x0C, 0x0A, 0x80, 0xE7, // jal
        0x24, 0x03, 0x00, 0x02, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x02, // addiu
        0x0C, 0x0A, 0x80, 0xF5, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x02, // addiu
        0x0C, 0x0A, 0x81, 0x03, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x01, 0x3F, 0x00, // lui
        0x44, 0x81, 0x00, 0x00, // mtc1
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x02, // addiu
        0x0C, 0x0A, 0x81, 0x20, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x03, // addiu
        0x0C, 0x0A, 0x80, 0xE7, // jal
        0x24, 0x03, 0x00, 0x02, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x03, // addiu
        0x0C, 0x0A, 0x80, 0xF5, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x03, // addiu
        0x0C, 0x0A, 0x81, 0x03, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x01, 0x3F, 0x00, // lui
        0x44, 0x81, 0x00, 0x00, // mtc1
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x03, // addiu
        0x0C, 0x0A, 0x81, 0x20, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x04, // addiu
        0x0C, 0x0A, 0x80, 0xE7, // jal
        0x24, 0x03, 0x00, 0x02, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x04, // addiu
        0x0C, 0x0A, 0x80, 0xF5, // jal
        0x24, 0x03, 0x00, 0x00, // addiu
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x04, // addiu
        0x0C, 0x0A, 0x81, 0x03, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x01, 0x3F, 0x00, // lui
        0x44, 0x81, 0x00, 0x00, // mtc1
        0x3C, 0x04, 0x80, 0x3F, // lui
        0x24, 0x84, 0xD4, 0x60, // addiu
        0x24, 0x02, 0x00, 0x04, // addiu
        0x0C, 0x0A, 0x81, 0x20, // jal
        0x24, 0x03, 0x00, 0x01, // addiu
        0x3C, 0x08, 0x80, 0x36, // lui
        0x25, 0x08, 0x44, 0x4C, // addiu
        0x24, 0x09, 0x0D, 0x48, // addiu
        0xA5, 0x09, 0x00, 0x00, // sh
        0x3C, 0x08, 0x80, 0x36, // lui
        0x25, 0x08, 0x44, 0x50, // addiu
        0x24, 0x09, 0x00, 0x00, // addiu
        0xA5, 0x09, 0x00, 0x00, // sh
        0xDF, 0xBF, 0x00, 0x00, // ld
        0xDF, 0xAD, 0x00, 0x30, // ld
        0xDF, 0xAC, 0x00, 0x28, // ld
        0xDF, 0xAB, 0x00, 0x20, // ld
        0xDF, 0xAA, 0x00, 0x18, // ld
        0xDF, 0xA9, 0x00, 0x10, // ld
        0xDF, 0xA8, 0x00, 0x08, // ld
        0x03, 0xE0, 0x00, 0x08, // jr
        0x27, 0xBD, 0x00, 0x38, // addiu
    ];

    let rom = Rom::new(0x00069BB0);
    let vram = Vram::new(0x802AE370);

    let segment_rom = Rom::new(0x00000000);
    let segment_vram = Vram::new(0x802447C0);

    let text_settings = ExecutableSectionSettings::new(
        Some(Compiler::IDO),
        InstructionFlags::new(IsaVersion::MIPS_III),
    );

    let mut context = {
        let global_config = GlobalConfigBuilder::new(Endian::Big)
            .with_gp_config(None)
            .build();

        let global_ranges = RomVramRange::new_option(
            AddressRange::new(segment_rom, Rom::new(0x000CAEA0)),
            AddressRange::new(segment_vram, Vram::new(0x80400000)),
            4,
        )
        .unwrap();
        let mut global_segment =
            GlobalSegmentBuilder::new("segment", global_ranges).finish_symbols();

        global_segment
            .preheat_text(&global_config, &text_settings, "text", &BYTES, rom, vram)
            .unwrap();

        let mut builder = ContextBuilder::new();
        builder.add_global_segment(global_segment).unwrap();

        let absolute_segment = AbsoluteSegmentBuilder::new();

        builder.build(global_config, absolute_segment).unwrap()
    };

    let parent_segment_info = ParentSegmentInfo::new(segment_rom, segment_vram, None);
    let section_text = context
        .create_section_text(
            &text_settings,
            "text",
            BYTES.to_vec(),
            rom,
            vram,
            parent_segment_info,
        )
        .unwrap();

    let user_relocs = UserRelocs::new();
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

    println!("{disassembly}");

    let expected_disassembly = "\
.section .text

/* Handwritten function */
/* Automatically generated and unreferenced pad */
glabel func_802AE370
    /* 069BB0 802AE370 27BDFFC8 */  addiu       $sp, $sp, -0x38
    /* 069BB4 802AE374 FFA90010 */  sd          $t1, 0x10($sp)
    /* 069BB8 802AE378 3C09803F */  lui         $t1, %hi(UNK_803ED818)
    /* 069BBC 802AE37C FFBF0000 */  sd          $ra, 0x0($sp)
    /* 069BC0 802AE380 FFA80008 */  sd          $t0, 0x8($sp)
    /* 069BC4 802AE384 FFAA0018 */  sd          $t2, 0x18($sp)
    /* 069BC8 802AE388 FFAB0020 */  sd          $t3, 0x20($sp)
    /* 069BCC 802AE38C FFAC0028 */  sd          $t4, 0x28($sp)
    /* 069BD0 802AE390 FFAD0030 */  sd          $t5, 0x30($sp)
    /* 069BD4 802AE394 2529D818 */  addiu       $t1, $t1, %lo(UNK_803ED818)
    /* 069BD8 802AE398 3C028036 */  lui         $v0, %hi(UNK_80358070)
    /* 069BDC 802AE39C AD320000 */  sw          $s2, 0x0($t1)
    /* 069BE0 802AE3A0 24428070 */  addiu       $v0, $v0, %lo(UNK_80358070)
    /* 069BE4 802AE3A4 8C430000 */  lw          $v1, 0x0($v0)
    /* 069BE8 802AE3A8 3C04803F */  lui         $a0, %hi(UNK_803ED82C)
    /* 069BEC 802AE3AC 2484D82C */  addiu       $a0, $a0, %lo(UNK_803ED82C)
    /* 069BF0 802AE3B0 AC830000 */  sw          $v1, 0x0($a0)
    /* 069BF4 802AE3B4 3C04803F */  lui         $a0, %hi(UNK_803ED830)
    /* 069BF8 802AE3B8 2484D830 */  addiu       $a0, $a0, %lo(UNK_803ED830)
    /* 069BFC 802AE3BC 20630C80 */  addi        $v1, $v1, 0xC80 /* handwritten instruction */
    /* 069C00 802AE3C0 AC830000 */  sw          $v1, 0x0($a0)
    /* 069C04 802AE3C4 20630C80 */  addi        $v1, $v1, 0xC80 /* handwritten instruction */
    /* 069C08 802AE3C8 AC430000 */  sw          $v1, 0x0($v0)
    /* 069C0C 802AE3CC 3C02803F */  lui         $v0, %hi(UNK_803ED82C)
    /* 069C10 802AE3D0 3C03803F */  lui         $v1, %hi(UNK_803ED830)
    /* 069C14 802AE3D4 2442D82C */  addiu       $v0, $v0, %lo(UNK_803ED82C)
    /* 069C18 802AE3D8 2463D830 */  addiu       $v1, $v1, %lo(UNK_803ED830)
    /* 069C1C 802AE3DC 8C420000 */  lw          $v0, 0x0($v0)
    /* 069C20 802AE3E0 8C630000 */  lw          $v1, 0x0($v1)
    /* 069C24 802AE3E4 24040000 */  addiu       $a0, $zero, 0x0
    /* 069C28 802AE3E8 0C0A84E2 */  jal         UNK_func_802A1388
    /* 069C2C 802AE3EC 24050000 */   addiu      $a1, $zero, 0x0
    /* 069C30 802AE3F0 3C1C803F */  lui         $gp, %hi(UNK_803ED760)
    /* 069C34 802AE3F4 0C0A9D53 */  jal         UNK_func_802A754C
    /* 069C38 802AE3F8 279CD760 */   addiu      $gp, $gp, %lo(UNK_803ED760)
    /* 069C3C 802AE3FC 2409001E */  addiu       $t1, $zero, 0x1E
    /* 069C40 802AE400 A7890052 */  sh          $t1, 0x52($gp)
    /* 069C44 802AE404 A7890054 */  sh          $t1, 0x54($gp)
    /* 069C48 802AE408 2409FFE2 */  addiu       $t1, $zero, -0x1E
    /* 069C4C 802AE40C A7890056 */  sh          $t1, 0x56($gp)
    /* 069C50 802AE410 2409001E */  addiu       $t1, $zero, 0x1E
    /* 069C54 802AE414 A7890058 */  sh          $t1, 0x58($gp)
    /* 069C58 802AE418 A789005A */  sh          $t1, 0x5A($gp)
    /* 069C5C 802AE41C 2409FFE2 */  addiu       $t1, $zero, -0x1E
    /* 069C60 802AE420 A789005C */  sh          $t1, 0x5C($gp)
    /* 069C64 802AE424 24090023 */  addiu       $t1, $zero, 0x23
    /* 069C68 802AE428 A789005E */  sh          $t1, 0x5E($gp)
    /* 069C6C 802AE42C A7890060 */  sh          $t1, 0x60($gp)
    /* 069C70 802AE430 2409FFDD */  addiu       $t1, $zero, -0x23
    /* 069C74 802AE434 A7890062 */  sh          $t1, 0x62($gp)
    /* 069C78 802AE438 24090023 */  addiu       $t1, $zero, 0x23
    /* 069C7C 802AE43C A7890064 */  sh          $t1, 0x64($gp)
    /* 069C80 802AE440 A7890066 */  sh          $t1, 0x66($gp)
    /* 069C84 802AE444 2409FFDD */  addiu       $t1, $zero, -0x23
    /* 069C88 802AE448 3C08803F */  lui         $t0, %hi(UNK_803ED808)
    /* 069C8C 802AE44C A7890068 */  sh          $t1, 0x68($gp)
    /* 069C90 802AE450 2508D808 */  addiu       $t0, $t0, %lo(UNK_803ED808)
    /* 069C94 802AE454 AD0F0000 */  sw          $t7, 0x0($t0)
    /* 069C98 802AE458 3C08803F */  lui         $t0, %hi(UNK_803ED80C)
    /* 069C9C 802AE45C 2508D80C */  addiu       $t0, $t0, %lo(UNK_803ED80C)
    /* 069CA0 802AE460 AD130000 */  sw          $s3, 0x0($t0)
    /* 069CA4 802AE464 3C08803F */  lui         $t0, %hi(UNK_803ED810)
    /* 069CA8 802AE468 2508D810 */  addiu       $t0, $t0, %lo(UNK_803ED810)
    /* 069CAC 802AE46C AD100000 */  sw          $s0, 0x0($t0)
    /* 069CB0 802AE470 2394004C */  addi        $s4, $gp, 0x4C /* handwritten instruction */
    /* 069CB4 802AE474 A6910000 */  sh          $s1, 0x0($s4)
    /* 069CB8 802AE478 A791004E */  sh          $s1, 0x4E($gp)
    /* 069CBC 802AE47C A7910074 */  sh          $s1, 0x74($gp)
    /* 069CC0 802AE480 3C12803F */  lui         $s2, %hi(UNK_803ED825)
    /* 069CC4 802AE484 24110001 */  addiu       $s1, $zero, 0x1
    /* 069CC8 802AE488 2652D825 */  addiu       $s2, $s2, %lo(UNK_803ED825)
    /* 069CCC 802AE48C A2510000 */  sb          $s1, 0x0($s2)
    /* 069CD0 802AE490 3C01803F */  lui         $at, %hi(UNK_803ED826)
    /* 069CD4 802AE494 A020D826 */  sb          $zero, %lo(UNK_803ED826)($at)
    /* 069CD8 802AE498 3C01803F */  lui         $at, %hi(UNK_803ED827)
    /* 069CDC 802AE49C A020D827 */  sb          $zero, %lo(UNK_803ED827)($at)
    /* 069CE0 802AE4A0 3C01803F */  lui         $at, %hi(UNK_803ED824)
    /* 069CE4 802AE4A4 A020D824 */  sb          $zero, %lo(UNK_803ED824)($at)
    /* 069CE8 802AE4A8 24110000 */  addiu       $s1, $zero, 0x0
    /* 069CEC 802AE4AC 3C12803F */  lui         $s2, %hi(UNK_803ED80C)
    /* 069CF0 802AE4B0 A39100A1 */  sb          $s1, 0xA1($gp)
    /* 069CF4 802AE4B4 2652D80C */  addiu       $s2, $s2, %lo(UNK_803ED80C)
    /* 069CF8 802AE4B8 8E4A0000 */  lw          $t2, 0x0($s2)
    /* 069CFC 802AE4BC 23910004 */  addi        $s1, $gp, 0x4 /* handwritten instruction */
    /* 069D00 802AE4C0 23830052 */  addi        $v1, $gp, 0x52 /* handwritten instruction */
    /* 069D04 802AE4C4 0C0AA64B */  jal         UNK_func_802A992C
    /* 069D08 802AE4C8 24180000 */   addiu      $t8, $zero, 0x0
    /* 069D0C 802AE4CC 3C0A803F */  lui         $t2, %hi(UNK_803ED818)
    /* 069D10 802AE4D0 3C16803F */  lui         $s6, %hi(UNK_803ED82C)
    /* 069D14 802AE4D4 3C17803F */  lui         $s7, %hi(UNK_803ED830)
    /* 069D18 802AE4D8 254AD818 */  addiu       $t2, $t2, %lo(UNK_803ED818)
    /* 069D1C 802AE4DC 26D6D82C */  addiu       $s6, $s6, %lo(UNK_803ED82C)
    /* 069D20 802AE4E0 26F7D830 */  addiu       $s7, $s7, %lo(UNK_803ED830)
    /* 069D24 802AE4E4 8D4A0000 */  lw          $t2, 0x0($t2)
    /* 069D28 802AE4E8 8ED60000 */  lw          $s6, 0x0($s6)
    /* 069D2C 802AE4EC 8EF70000 */  lw          $s7, 0x0($s7)
    /* 069D30 802AE4F0 3C08803F */  lui         $t0, %hi(UNK_803ED460)
    /* 069D34 802AE4F4 2508D460 */  addiu       $t0, $t0, %lo(UNK_803ED460)
    /* 069D38 802AE4F8 01404825 */  or          $t1, $t2, $zero
    /* 069D3C 802AE4FC 02C01825 */  or          $v1, $s6, $zero
    /* 069D40 802AE500 0C0A7E17 */  jal         UNK_func_8029F85C
    /* 069D44 802AE504 02E02025 */   or         $a0, $s7, $zero
    /* 069D48 802AE508 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069D4C 802AE50C 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069D50 802AE510 24020000 */  addiu       $v0, $zero, 0x0
    /* 069D54 802AE514 0C0A80E7 */  jal         UNK_func_802A039C
    /* 069D58 802AE518 24030064 */   addiu      $v1, $zero, 0x64
    /* 069D5C 802AE51C 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069D60 802AE520 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069D64 802AE524 24020000 */  addiu       $v0, $zero, 0x0
    /* 069D68 802AE528 0C0A80F5 */  jal         UNK_func_802A03D4
    /* 069D6C 802AE52C 24030000 */   addiu      $v1, $zero, 0x0
    /* 069D70 802AE530 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069D74 802AE534 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069D78 802AE538 24020000 */  addiu       $v0, $zero, 0x0
    /* 069D7C 802AE53C 0C0A8103 */  jal         UNK_func_802A040C
    /* 069D80 802AE540 24030000 */   addiu      $v1, $zero, 0x0
    /* 069D84 802AE544 44800000 */  mtc1        $zero, $fv0
    /* 069D88 802AE548 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069D8C 802AE54C 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069D90 802AE550 24020000 */  addiu       $v0, $zero, 0x0
    /* 069D94 802AE554 0C0A8120 */  jal         UNK_func_802A0480
    /* 069D98 802AE558 24030000 */   addiu      $v1, $zero, 0x0
    /* 069D9C 802AE55C 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069DA0 802AE560 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069DA4 802AE564 24020000 */  addiu       $v0, $zero, 0x0
    /* 069DA8 802AE568 0C0A80A4 */  jal         UNK_func_802A0290
    /* 069DAC 802AE56C 24030001 */   addiu      $v1, $zero, 0x1
    /* 069DB0 802AE570 3C08803F */  lui         $t0, %hi(UNK_803ED460)
    /* 069DB4 802AE574 2508D460 */  addiu       $t0, $t0, %lo(UNK_803ED460)
    /* 069DB8 802AE578 02C01025 */  or          $v0, $s6, $zero
    /* 069DBC 802AE57C 0C0A7956 */  jal         UNK_func_8029E558
    /* 069DC0 802AE580 02E01825 */   or         $v1, $s7, $zero
    /* 069DC4 802AE584 3C03803F */  lui         $v1, %hi(UNK_803ED460)
    /* 069DC8 802AE588 2463D460 */  addiu       $v1, $v1, %lo(UNK_803ED460)
    /* 069DCC 802AE58C 0C0A80C8 */  jal         UNK_func_802A0320
    /* 069DD0 802AE590 24020000 */   addiu      $v0, $zero, 0x0
    /* 069DD4 802AE594 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069DD8 802AE598 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069DDC 802AE59C 24020000 */  addiu       $v0, $zero, 0x0
    /* 069DE0 802AE5A0 0C0A80A4 */  jal         UNK_func_802A0290
    /* 069DE4 802AE5A4 24030001 */   addiu      $v1, $zero, 0x1
    /* 069DE8 802AE5A8 3C08803F */  lui         $t0, %hi(UNK_803ED460)
    /* 069DEC 802AE5AC 2508D460 */  addiu       $t0, $t0, %lo(UNK_803ED460)
    /* 069DF0 802AE5B0 02E01025 */  or          $v0, $s7, $zero
    /* 069DF4 802AE5B4 0C0A7956 */  jal         UNK_func_8029E558
    /* 069DF8 802AE5B8 02C01825 */   or         $v1, $s6, $zero
    /* 069DFC 802AE5BC 2409FFD8 */  addiu       $t1, $zero, -0x28
    /* 069E00 802AE5C0 A7890078 */  sh          $t1, 0x78($gp)
    /* 069E04 802AE5C4 24090000 */  addiu       $t1, $zero, 0x0
    /* 069E08 802AE5C8 A789007A */  sh          $t1, 0x7A($gp)
    /* 069E0C 802AE5CC 24090002 */  addiu       $t1, $zero, 0x2
    /* 069E10 802AE5D0 A789007C */  sh          $t1, 0x7C($gp)
    /* 069E14 802AE5D4 24090000 */  addiu       $t1, $zero, 0x0
    /* 069E18 802AE5D8 A789007E */  sh          $t1, 0x7E($gp)
    /* 069E1C 802AE5DC 24090028 */  addiu       $t1, $zero, 0x28
    /* 069E20 802AE5E0 A7890080 */  sh          $t1, 0x80($gp)
    /* 069E24 802AE5E4 24090002 */  addiu       $t1, $zero, 0x2
    /* 069E28 802AE5E8 A7890082 */  sh          $t1, 0x82($gp)
    /* 069E2C 802AE5EC 24090028 */  addiu       $t1, $zero, 0x28
    /* 069E30 802AE5F0 A7890084 */  sh          $t1, 0x84($gp)
    /* 069E34 802AE5F4 2409003C */  addiu       $t1, $zero, 0x3C
    /* 069E38 802AE5F8 A7890086 */  sh          $t1, 0x86($gp)
    /* 069E3C 802AE5FC 24090002 */  addiu       $t1, $zero, 0x2
    /* 069E40 802AE600 A7890088 */  sh          $t1, 0x88($gp)
    /* 069E44 802AE604 2409003C */  addiu       $t1, $zero, 0x3C
    /* 069E48 802AE608 A789008A */  sh          $t1, 0x8A($gp)
    /* 069E4C 802AE60C 24090050 */  addiu       $t1, $zero, 0x50
    /* 069E50 802AE610 A789008C */  sh          $t1, 0x8C($gp)
    /* 069E54 802AE614 24090002 */  addiu       $t1, $zero, 0x2
    /* 069E58 802AE618 A789008E */  sh          $t1, 0x8E($gp)
    /* 069E5C 802AE61C 24090050 */  addiu       $t1, $zero, 0x50
    /* 069E60 802AE620 A7890090 */  sh          $t1, 0x90($gp)
    /* 069E64 802AE624 24090064 */  addiu       $t1, $zero, 0x64
    /* 069E68 802AE628 A7890092 */  sh          $t1, 0x92($gp)
    /* 069E6C 802AE62C 24090002 */  addiu       $t1, $zero, 0x2
    /* 069E70 802AE630 3C0B803F */  lui         $t3, %hi(UNK_803ED818)
    /* 069E74 802AE634 A7890094 */  sh          $t1, 0x94($gp)
    /* 069E78 802AE638 256BD818 */  addiu       $t3, $t3, %lo(UNK_803ED818)
    /* 069E7C 802AE63C 8D6B0000 */  lw          $t3, 0x0($t3)
    /* 069E80 802AE640 24080000 */  addiu       $t0, $zero, 0x0
    /* 069E84 802AE644 8D690004 */  lw          $t1, 0x4($t3)
    /* 069E88 802AE648 8D6A0008 */  lw          $t2, 0x8($t3)
    /* 069E8C 802AE64C 012B4820 */  add         $t1, $t1, $t3 /* handwritten instruction */
    /* 069E90 802AE650 014B5020 */  add         $t2, $t2, $t3 /* handwritten instruction */
    /* 069E94 802AE654 0C0A70D5 */  jal         UNK_func_8029C354
    /* 069E98 802AE658 240B4E20 */   addiu      $t3, $zero, 0x4E20
    /* 069E9C 802AE65C 27BDFFC0 */  addiu       $sp, $sp, -0x40
    /* 069EA0 802AE660 24040000 */  addiu       $a0, $zero, 0x0
    /* 069EA4 802AE664 24050028 */  addiu       $a1, $zero, 0x28
    /* 069EA8 802AE668 2406000F */  addiu       $a2, $zero, 0xF
    /* 069EAC 802AE66C 0C09608C */  jal         UNK_func_80258230
    /* 069EB0 802AE670 2407000F */   addiu      $a3, $zero, 0xF
    /* 069EB4 802AE674 24090001 */  addiu       $t1, $zero, 0x1
    /* 069EB8 802AE678 23BD0040 */  addi        $sp, $sp, 0x40 /* handwritten instruction */
    /* 069EBC 802AE67C 0C0ABBB2 */  jal         UNK_func_802AEEC8
    /* 069EC0 802AE680 A389009A */   sb         $t1, 0x9A($gp)
    /* 069EC4 802AE684 3C0B803F */  lui         $t3, %hi(UNK_803ED818)
    /* 069EC8 802AE688 A380009A */  sb          $zero, 0x9A($gp)
    /* 069ECC 802AE68C 256BD818 */  addiu       $t3, $t3, %lo(UNK_803ED818)
    /* 069ED0 802AE690 8D6B0000 */  lw          $t3, 0x0($t3)
    /* 069ED4 802AE694 3C08803F */  lui         $t0, %hi(UNK_803ED830)
    /* 069ED8 802AE698 3C09803F */  lui         $t1, %hi(UNK_803ED82C)
    /* 069EDC 802AE69C 8D6A0018 */  lw          $t2, 0x18($t3)
    /* 069EE0 802AE6A0 2508D830 */  addiu       $t0, $t0, %lo(UNK_803ED830)
    /* 069EE4 802AE6A4 2529D82C */  addiu       $t1, $t1, %lo(UNK_803ED82C)
    /* 069EE8 802AE6A8 014B5020 */  add         $t2, $t2, $t3 /* handwritten instruction */
    /* 069EEC 802AE6AC 8D080000 */  lw          $t0, 0x0($t0)
    /* 069EF0 802AE6B0 8D290000 */  lw          $t1, 0x0($t1)
    /* 069EF4 802AE6B4 0C0AAA0E */  jal         UNK_func_802AA838
    /* 069EF8 802AE6B8 8D4A0004 */   lw         $t2, 0x4($t2)
    /* 069EFC 802AE6BC 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F00 802AE6C0 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F04 802AE6C4 24020001 */  addiu       $v0, $zero, 0x1
    /* 069F08 802AE6C8 0C0A80E7 */  jal         UNK_func_802A039C
    /* 069F0C 802AE6CC 24030000 */   addiu      $v1, $zero, 0x0
    /* 069F10 802AE6D0 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F14 802AE6D4 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F18 802AE6D8 24020001 */  addiu       $v0, $zero, 0x1
    /* 069F1C 802AE6DC 0C0A80F5 */  jal         UNK_func_802A03D4
    /* 069F20 802AE6E0 24030000 */   addiu      $v1, $zero, 0x0
    /* 069F24 802AE6E4 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F28 802AE6E8 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F2C 802AE6EC 24020001 */  addiu       $v0, $zero, 0x1
    /* 069F30 802AE6F0 0C0A8103 */  jal         UNK_func_802A040C
    /* 069F34 802AE6F4 24030000 */   addiu      $v1, $zero, 0x0
    /* 069F38 802AE6F8 3C013F00 */  lui         $at, (0x3F000000 >> 16)
    /* 069F3C 802AE6FC 44810000 */  mtc1        $at, $fv0
    /* 069F40 802AE700 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F44 802AE704 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F48 802AE708 24020001 */  addiu       $v0, $zero, 0x1
    /* 069F4C 802AE70C 0C0A8120 */  jal         UNK_func_802A0480
    /* 069F50 802AE710 24030001 */   addiu      $v1, $zero, 0x1
    /* 069F54 802AE714 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F58 802AE718 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F5C 802AE71C 24020002 */  addiu       $v0, $zero, 0x2
    /* 069F60 802AE720 0C0A80E7 */  jal         UNK_func_802A039C
    /* 069F64 802AE724 24030002 */   addiu      $v1, $zero, 0x2
    /* 069F68 802AE728 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F6C 802AE72C 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F70 802AE730 24020002 */  addiu       $v0, $zero, 0x2
    /* 069F74 802AE734 0C0A80F5 */  jal         UNK_func_802A03D4
    /* 069F78 802AE738 24030000 */   addiu      $v1, $zero, 0x0
    /* 069F7C 802AE73C 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F80 802AE740 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069F84 802AE744 24020002 */  addiu       $v0, $zero, 0x2
    /* 069F88 802AE748 0C0A8103 */  jal         UNK_func_802A040C
    /* 069F8C 802AE74C 24030001 */   addiu      $v1, $zero, 0x1
    /* 069F90 802AE750 3C013F00 */  lui         $at, (0x3F000000 >> 16)
    /* 069F94 802AE754 44810000 */  mtc1        $at, $fv0
    /* 069F98 802AE758 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069F9C 802AE75C 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069FA0 802AE760 24020002 */  addiu       $v0, $zero, 0x2
    /* 069FA4 802AE764 0C0A8120 */  jal         UNK_func_802A0480
    /* 069FA8 802AE768 24030001 */   addiu      $v1, $zero, 0x1
    /* 069FAC 802AE76C 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069FB0 802AE770 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069FB4 802AE774 24020003 */  addiu       $v0, $zero, 0x3
    /* 069FB8 802AE778 0C0A80E7 */  jal         UNK_func_802A039C
    /* 069FBC 802AE77C 24030002 */   addiu      $v1, $zero, 0x2
    /* 069FC0 802AE780 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069FC4 802AE784 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069FC8 802AE788 24020003 */  addiu       $v0, $zero, 0x3
    /* 069FCC 802AE78C 0C0A80F5 */  jal         UNK_func_802A03D4
    /* 069FD0 802AE790 24030000 */   addiu      $v1, $zero, 0x0
    /* 069FD4 802AE794 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069FD8 802AE798 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069FDC 802AE79C 24020003 */  addiu       $v0, $zero, 0x3
    /* 069FE0 802AE7A0 0C0A8103 */  jal         UNK_func_802A040C
    /* 069FE4 802AE7A4 24030001 */   addiu      $v1, $zero, 0x1
    /* 069FE8 802AE7A8 3C013F00 */  lui         $at, (0x3F000000 >> 16)
    /* 069FEC 802AE7AC 44810000 */  mtc1        $at, $fv0
    /* 069FF0 802AE7B0 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 069FF4 802AE7B4 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 069FF8 802AE7B8 24020003 */  addiu       $v0, $zero, 0x3
    /* 069FFC 802AE7BC 0C0A8120 */  jal         UNK_func_802A0480
    /* 06A000 802AE7C0 24030001 */   addiu      $v1, $zero, 0x1
    /* 06A004 802AE7C4 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 06A008 802AE7C8 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 06A00C 802AE7CC 24020004 */  addiu       $v0, $zero, 0x4
    /* 06A010 802AE7D0 0C0A80E7 */  jal         UNK_func_802A039C
    /* 06A014 802AE7D4 24030002 */   addiu      $v1, $zero, 0x2
    /* 06A018 802AE7D8 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 06A01C 802AE7DC 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 06A020 802AE7E0 24020004 */  addiu       $v0, $zero, 0x4
    /* 06A024 802AE7E4 0C0A80F5 */  jal         UNK_func_802A03D4
    /* 06A028 802AE7E8 24030000 */   addiu      $v1, $zero, 0x0
    /* 06A02C 802AE7EC 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 06A030 802AE7F0 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 06A034 802AE7F4 24020004 */  addiu       $v0, $zero, 0x4
    /* 06A038 802AE7F8 0C0A8103 */  jal         UNK_func_802A040C
    /* 06A03C 802AE7FC 24030001 */   addiu      $v1, $zero, 0x1
    /* 06A040 802AE800 3C013F00 */  lui         $at, (0x3F000000 >> 16)
    /* 06A044 802AE804 44810000 */  mtc1        $at, $fv0
    /* 06A048 802AE808 3C04803F */  lui         $a0, %hi(UNK_803ED460)
    /* 06A04C 802AE80C 2484D460 */  addiu       $a0, $a0, %lo(UNK_803ED460)
    /* 06A050 802AE810 24020004 */  addiu       $v0, $zero, 0x4
    /* 06A054 802AE814 0C0A8120 */  jal         UNK_func_802A0480
    /* 06A058 802AE818 24030001 */   addiu      $v1, $zero, 0x1
    /* 06A05C 802AE81C 3C088036 */  lui         $t0, %hi(UNK_8036444C)
    /* 06A060 802AE820 2508444C */  addiu       $t0, $t0, %lo(UNK_8036444C)
    /* 06A064 802AE824 24090D48 */  addiu       $t1, $zero, 0xD48
    /* 06A068 802AE828 A5090000 */  sh          $t1, 0x0($t0)
    /* 06A06C 802AE82C 3C088036 */  lui         $t0, %hi(UNK_80364450)
    /* 06A070 802AE830 25084450 */  addiu       $t0, $t0, %lo(UNK_80364450)
    /* 06A074 802AE834 24090000 */  addiu       $t1, $zero, 0x0
    /* 06A078 802AE838 A5090000 */  sh          $t1, 0x0($t0)
    /* 06A07C 802AE83C DFBF0000 */  ld          $ra, 0x0($sp)
    /* 06A080 802AE840 DFAD0030 */  ld          $t5, 0x30($sp)
    /* 06A084 802AE844 DFAC0028 */  ld          $t4, 0x28($sp)
    /* 06A088 802AE848 DFAB0020 */  ld          $t3, 0x20($sp)
    /* 06A08C 802AE84C DFAA0018 */  ld          $t2, 0x18($sp)
    /* 06A090 802AE850 DFA90010 */  ld          $t1, 0x10($sp)
    /* 06A094 802AE854 DFA80008 */  ld          $t0, 0x8($sp)
    /* 06A098 802AE858 03E00008 */  jr          $ra
    /* 06A09C 802AE85C 27BD0038 */   addiu      $sp, $sp, 0x38
.size func_802AE370, . - func_802AE370
";

    assert_eq!(disassembly, expected_disassembly,);
}
