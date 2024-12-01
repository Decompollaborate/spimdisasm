/* SPDX-FileCopyrightText: © 2024 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use rabbitizer::{DisplayFlags, InstructionFlags, Vram};
use spimdisasm::{
    address_range::AddressRange,
    config::{Endian, GlobalConfig},
    context::ContextBuilder,
    parent_segment_info::ParentSegmentInfo,
    rom_address::RomAddress,
    sections::{SectionText, SectionTextSettings},
    size::Size,
    symbols::display::FunctionDisplaySettings,
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
    let rom = RomAddress::new(0x001050);
    let vram = Vram::new(0x8081CE54);
    let size = Size::new(bytes.len() as u32);

    let global_config = GlobalConfig::new(Endian::Big);
    let mut context = ContextBuilder::new(
        global_config,
        AddressRange::new(rom, rom + size),
        AddressRange::new(vram, vram + size),
    )
    .process()
    .process()
    .process()
    .build();

    let text_settings = SectionTextSettings::new(InstructionFlags::new());
    let instr_display_flags = DisplayFlags::default();

    let section_text = SectionText::new(
        &mut context,
        &text_settings,
        "test".into(),
        &bytes,
        rom,
        vram,
        ParentSegmentInfo::new(rom, None),
    )
    .unwrap();

    let function_display_settings = FunctionDisplaySettings::new(instr_display_flags);
    for func in section_text.functions() {
        let func_display = func.display(&context, &function_display_settings);
        println!("{}", func_display);
    }

    assert_eq!(section_text.functions().len(), 1);

    let symbols = context.global_segment().symbols();
    for s in symbols {
        println!("{:?}", s.1);
    }
    assert_eq!(symbols.len(), 21);

    // None::<u32>.unwrap();
}