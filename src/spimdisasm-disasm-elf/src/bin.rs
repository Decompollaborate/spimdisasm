/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use clap::{error::Result, Parser};
use object::{
    self, elf,
    read::elf::{ElfFile32, FileHeader, Sym},
    Object, ObjectSection, ObjectSymbol,
};
use spimdisasm::{
    self,
    addresses::{AddressRange, Rom, RomVramRange, Size, Vram},
    analysis::StringGuesserFlags,
    config::{GlobalConfig, GpConfig},
    context::{
        builder::{GlobalSegmentHeater, UserSegmentBuilder},
        Context, ContextBuilder, GlobalSegmentBuilder,
    },
    metadata::{GotAccessKind, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion},
    relocation::RelocationInfo,
    sections::{
        before_proc::{
            DataSection, DataSectionSettings, ExecutableSection, ExecutableSectionSettings,
            NoloadSection, NoloadSectionSettings,
        },
        processed::{DataSectionProcessed, ExecutableSectionProcessed, NoloadSectionProcessed},
        Section, SectionPostProcessError,
    },
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNoloadDisplaySettings},
};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

mod dynamic_section;
mod elf_section_type;
mod global_offset_table;
mod mips_reginfo;
mod utils;

use dynamic_section::DynamicSection;
use elf_section_type::{ElfSectionType, ProgbitsType};
use global_offset_table::parse_got;
use mips_reginfo::MipsReginfo;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
#[allow(non_camel_case_types)]
pub enum ArgCompiler {
    #[clap(aliases=&["ido"])]
    IDO,
}

impl From<ArgCompiler> for spimdisasm::config::Compiler {
    fn from(value: ArgCompiler) -> Self {
        match value {
            ArgCompiler::IDO => Self::IDO,
        }
    }
}

/// disasm-elf: CLI tool to disassemble an elf file using spimdisasm
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    input_path: PathBuf,
    output_dir: PathBuf,

    #[clap(long)]
    compiler: Option<ArgCompiler>,
}

#[allow(dead_code)]
fn print_elf_stuff(elf_file: &ElfFile32) {
    let elf_header = elf_file.elf_header();

    match elf_header.e_type(elf_file.endian()) {
        elf::ET_REL => println!("Relocatable file"),
        elf::ET_EXEC => println!("Executable file"),
        elf::ET_DYN => println!("Shared object file"),
        _ => panic!("Unsupported e_type"),
    }

    match elf_header.e_machine(elf_file.endian()) {
        elf::EM_MIPS => println!("MIPS R3000 big-endian"),
        elf::EM_MIPS_RS3_LE => println!("MIPS R3000 little-endian"),
        _ => panic!("Unsupported e_machine"),
    }

    println!(
        "Entrypoint: 0x{:08X}",
        elf_header.e_entry(elf_file.endian())
    );

    {
        let mut e_flags = elf_header.e_flags(elf_file.endian());
        println!("Flags: 0x{:08X}", e_flags);

        print!("    ");

        let arch = e_flags & elf::EF_MIPS_ARCH;
        e_flags &= !elf::EF_MIPS_ARCH;

        print!(
            "{}",
            match arch {
                elf::EF_MIPS_ARCH_1 => "mips1",
                elf::EF_MIPS_ARCH_2 => "mips2",
                elf::EF_MIPS_ARCH_3 => "mips3",
                elf::EF_MIPS_ARCH_4 => "mips4",
                elf::EF_MIPS_ARCH_5 => "mips5",
                elf::EF_MIPS_ARCH_32 => "mips32",
                elf::EF_MIPS_ARCH_64 => "mips64",
                elf::EF_MIPS_ARCH_32R2 => "mips32R2",
                elf::EF_MIPS_ARCH_64R2 => "mips64R2",
                elf::EF_MIPS_ARCH_32R6 => "mips32R6",
                elf::EF_MIPS_ARCH_64R6 => "mips64R6",
                _ => "UNKNOWN ARCH",
            }
        );

        if e_flags & elf::EF_MIPS_NOREORDER != 0 {
            print!(", noreorder");
            e_flags &= !elf::EF_MIPS_NOREORDER;
        }

        if e_flags & elf::EF_MIPS_PIC != 0 {
            print!(", pic");
            e_flags &= !elf::EF_MIPS_PIC;
        }
        if e_flags & elf::EF_MIPS_CPIC != 0 {
            print!(", cpic");
            e_flags &= !elf::EF_MIPS_CPIC;
        }

        if e_flags != 0 {
            print!(", ");
            print!("Unknown flags: 0x{:08X}", e_flags);
        }

        println!();
    }

    println!();

    println!("dynamic_symbols:");
    for sym in elf_file.dynamic_symbols() {
        println!("  {}", utils::pretty_unwrap(sym.name()));
    }
    println!();

    for section in elf_file.sections() {
        println!("{}", utils::pretty_unwrap(section.name()));
        println!(
            "    address: 0x{:08X}, size: 0x{:08X}, align: 0x{:02X}",
            section.address(),
            section.size(),
            section.align()
        );
        println!(
            "    kind: {:?}, flags: {:?}",
            section.kind(),
            section.flags()
        );

        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());
        println!("    sh_flags: {}", sh_flags);
        println!("    sh_type: {}", sh_type);
    }
}

fn gather_raw_got(elf_file: &ElfFile32) -> Option<(Vram, Vec<u32>)> {
    let endian = utils::endian_to_endian(elf_file.endian());

    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        if let Some(ElfSectionType::Progbits(ProgbitsType::Got)) = section_type {
            let vram = Vram::new(section.address() as u32);
            let data = utils::pretty_unwrap(section.data());
            let raw = data
                .chunks_exact(4)
                .map(|w| endian.word_from_bytes(w))
                .collect();

            return Some((vram, raw));
        }
    }

    None
}

fn create_global_ranges(elf_file: &ElfFile32) -> RomVramRange {
    let mut rom_start = None;
    let mut rom_end = None;
    let mut vram_start = None;
    let mut vram_end = None;

    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        if let Some(ElfSectionType::Progbits(_) | ElfSectionType::Nobits) = section_type {
            let mut vram = section.address() as u32;
            if vram == 0 {
                // hack to avoid having issues with relocatable files
                vram = section
                    .elf_section_header()
                    .sh_offset
                    .get(elf_file.endian());
            }
            let end = vram + section.size() as u32;

            if let Some(x) = vram_start {
                vram_start = Some(vram.min(x));
            } else {
                vram_start = Some(vram);
            }
            if let Some(x) = vram_end {
                vram_end = Some(end.max(x));
            } else {
                vram_end = Some(end);
            }

            if matches!(section_type, Some(ElfSectionType::Progbits(_))) {
                let rom = section
                    .elf_section_header()
                    .sh_offset
                    .get(elf_file.endian());
                let end = rom + section.size() as u32;

                if let Some(x) = rom_start {
                    rom_start = Some(rom.min(x));
                } else {
                    rom_start = Some(rom);
                }
                if let Some(x) = rom_end {
                    rom_end = Some(end.max(x));
                } else {
                    rom_end = Some(end);
                }
            }
        }
    }

    let rom = AddressRange::new(Rom::new(rom_start.unwrap()), Rom::new(rom_end.unwrap()));
    let vram = AddressRange::new(Vram::new(vram_start.unwrap()), Vram::new(vram_end.unwrap()));

    RomVramRange::new(rom, vram)
}

fn fill_dyn_symbols(
    elf_file: &ElfFile32,
    global_ranges: RomVramRange,
    global_config: &GlobalConfig,
    dynamic: &DynamicSection,
    got_vram: Vram,
    raw_got: &[u32],
) -> (GlobalSegmentHeater, UserSegmentBuilder) {
    let mut global_segment = GlobalSegmentBuilder::new(global_ranges);
    let mut user_segment = UserSegmentBuilder::new();

    let gotsym = dynamic.gotsym() as usize;
    let elf_endian = elf_file.endian();

    let global_offset_table = parse_got(elf_file, dynamic, got_vram, raw_got);
    let mut global_got = global_offset_table.globals().iter();

    let mut addresses = HashSet::new();
    let mut initials = HashSet::new();
    let mut local_syms = Vec::new();
    let mut weak_syms = Vec::new();
    let mut remaining_symbols = Vec::new();

    let dynsym = elf_file.elf_dynamic_symbol_table();
    let dynstr = dynsym.strings();

    // Silly hack to allow strings starting with `0x0A` (\n) or `0x09` (\t) to be detected as strings.
    // We need to do this because otherwise spimdisasm will think those values look like addresses,
    // because they happen to be in the middle of the non contiguous address space of the elf.
    utils::pretty_unwrap(
        global_segment.add_ignored_address_range(Vram::new(0x09000000), Size::new(0x00800000)),
    );
    utils::pretty_unwrap(
        global_segment.add_ignored_address_range(Vram::new(0x0A000000), Size::new(0x00800000)),
    );

    for (i, sym) in dynsym.enumerate() {
        let st_type = sym.st_type();
        let st_shndx = sym.st_shndx(elf_endian);

        let got_entry = if i.0 >= gotsym {
            global_got.next()
        } else {
            None
        };

        let raw_name = utils::pretty_unwrap(sym.name(elf_endian, dynstr));
        let name = utils::pretty_unwrap(String::from_utf8(raw_name.into()));

        if let Some(got_entry) = got_entry {
            let initial = got_entry.initial();
            if initial != 0 {
                let got_entry = Vram::new(initial);
                if !initials.contains(&got_entry) && initial >= 0x10 {
                    let sym_type = match st_type {
                        elf::STT_FUNC => Some(SymbolType::Function),
                        _ => None,
                    };

                    let size = {
                        let s = sym.st_size(elf_endian);
                        // This size seems to only be valid for `initial` for `UNDEF` and `COM` symbols.
                        // TODO: investigate COM symbols
                        if s == 0
                            || (st_shndx != elf::SHN_UNDEF && st_shndx != elf::SHN_COMMON)
                            || st_type == elf::STT_FUNC
                        {
                            Size::new(1)
                        } else {
                            Size::new(s)
                        }
                    };

                    let sym_metadata = utils::pretty_unwrap(user_segment.add_user_symbol(
                        got_entry,
                        name.clone(),
                        size,
                        sym_type,
                    ));
                    sym_metadata.set_got_access_kind(GotAccessKind::Global);

                    // TODO: set visibiility?
                    initials.insert(got_entry);
                }
            }
        }

        let sym_type = match st_type {
            elf::STT_FUNC => Some(SymbolType::Function),
            elf::STT_OBJECT => None,
            _ => continue,
        };

        let vram = Vram::new(sym.st_value(elf_endian));
        let rom = None;
        if vram < Vram::new(0x10) {
            continue;
        }

        let size = {
            let s = sym.st_size(elf_endian);
            if s == 0 || st_shndx == elf::SHN_UNDEF {
                None
            } else {
                Some(Size::new(s))
            }
        };

        match sym.st_visibility() {
            elf::STB_LOCAL => {
                local_syms.push((name, vram, rom, size, sym_type, got_entry));
                continue;
            }
            elf::STB_GLOBAL => {}
            elf::STB_WEAK => {
                weak_syms.push((name, vram, rom, size, sym_type, got_entry));
                continue;
            }
            x => panic!("Unhandled st_visibility: {}", x),
        }

        if global_ranges.in_vram_range(vram) {
            let sym_metadata = utils::pretty_unwrap(
                global_segment.add_user_symbol(name, vram, rom, size, sym_type),
            );
            if got_entry.is_some() {
                sym_metadata.set_got_access_kind(GotAccessKind::Global);
            }
            addresses.insert(vram);
        } else {
            remaining_symbols.push((name, vram, rom, size, sym_type, elf::STB_GLOBAL, got_entry));
        }
    }

    for (name, vram, rom, size, sym_type, got_entry) in local_syms {
        if addresses.contains(&vram) {
            continue;
        }

        if global_ranges.in_vram_range(vram) {
            let sym_metadata = utils::pretty_unwrap(
                global_segment.add_user_symbol(name, vram, rom, size, sym_type),
            );
            if got_entry.is_some() {
                sym_metadata.set_got_access_kind(GotAccessKind::Global);
            }
            // TODO: set local visibiility
            addresses.insert(vram);
        } else {
            remaining_symbols.push((name, vram, rom, size, sym_type, elf::STB_LOCAL, got_entry));
        }
    }

    for (name, vram, rom, size, sym_type, got_entry) in weak_syms {
        if addresses.contains(&vram) {
            continue;
        }

        if global_ranges.in_vram_range(vram) {
            let sym_metadata = utils::pretty_unwrap(
                global_segment.add_user_symbol(name, vram, rom, size, sym_type),
            );
            if got_entry.is_some() {
                sym_metadata.set_got_access_kind(GotAccessKind::Global);
            }
            // TODO: set weak visibiility
            addresses.insert(vram);
        } else {
            remaining_symbols.push((name, vram, rom, size, sym_type, elf::STB_WEAK, got_entry));
        }
    }

    for (name, vram, rom, size, sym_type, st_visibility, got_entry) in remaining_symbols {
        if let Some(got_entry) = got_entry {
            // Check for `COMMON` those symbols do not contain an address on their value, they contain an alignment instead.
            if vram == Vram::new(0) || got_entry.undef_com_or_abs() {
                let initial = got_entry.initial();
                if initial != 0 {
                    let got_entry = Vram::new(initial);
                    if initials.contains(&got_entry) {
                        continue;
                    }
                    let sym_metadata = utils::pretty_unwrap(user_segment.add_user_symbol(
                        got_entry,
                        name,
                        size.unwrap_or(Size::new(1)),
                        sym_type,
                    ));
                    sym_metadata.set_got_access_kind(GotAccessKind::Global);

                    // TODO: set visibiility?
                    initials.insert(got_entry);
                }
            } else {
                eprintln!(
                    "Unhandled dynamic symbol: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                    name, vram, rom, size, sym_type, st_visibility, got_entry
                );
            }
        } else {
            eprintln!(
                "Unhandled dynamic symbol: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                name, vram, rom, size, sym_type, st_visibility, got_entry
            );
        }
    }

    // First entry of the GOT is reserved for the lazy resolver
    if let Some(lazy_resolver) = global_offset_table.locals().first() {
        // ido 7.1 ld has its lazy resolver set to 0?
        let vram = Vram::new(lazy_resolver.inner());
        if vram != Vram::new(0) {
            // Something silly, so the user doesn't confuse this as a real symbol
            let name = "$$.LazyResolver";
            let size = Size::new(4);
            let typ = None;

            let sym_metadata =
                utils::pretty_unwrap(user_segment.add_user_symbol(vram, name, size, typ));
            // I'm not sure if this should be considered Local or Global.
            // Maybe make a new kind for this?
            sym_metadata.set_got_access_kind(GotAccessKind::Local);
        }
    }

    utils::pretty_unwrap(
        global_segment.add_global_offset_table(global_config, global_offset_table),
    );

    (global_segment.finish_symbols(), user_segment)
}

fn fill_symtab(
    _elf_file: &ElfFile32,
    global_ranges: RomVramRange,
) -> (GlobalSegmentHeater, UserSegmentBuilder) {
    let global_segment = GlobalSegmentBuilder::new(global_ranges);
    let user_segment = UserSegmentBuilder::new();

    // TODO: handle relocatable elfs

    (global_segment.finish_symbols(), user_segment)
}

fn preheat_sections(
    elf_file: &ElfFile32,
    global_segment: &mut GlobalSegmentHeater,
    global_config: &GlobalConfig,
    executable_settings: &ExecutableSectionSettings,
    data_settings: &DataSectionSettings,
) {
    // Executable sections first
    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        if let Some(ElfSectionType::Progbits(ProgbitsType::Text)) = section_type {
            let name = utils::pretty_unwrap(section.name());
            let raw_bytes = utils::pretty_unwrap(section.data());
            let rom = Rom::new(
                section
                    .elf_section_header()
                    .sh_offset
                    .get(elf_file.endian()),
            );
            let section_address = section.address();
            let vram = if section_address != 0 {
                Vram::new(section.address() as u32)
            } else {
                Vram::new(rom.inner())
            };

            utils::pretty_unwrap(global_segment.preheat_text(
                global_config,
                executable_settings,
                name,
                raw_bytes,
                rom,
                vram,
            ));
        }
    }

    // Data sections later
    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        if let Some(ElfSectionType::Progbits(progbits)) = section_type {
            let name = utils::pretty_unwrap(section.name());
            let raw_bytes = utils::pretty_unwrap(section.data());
            let rom = Rom::new(
                section
                    .elf_section_header()
                    .sh_offset
                    .get(elf_file.endian()),
            );
            let section_address = section.address();
            let vram = if section_address != 0 {
                Vram::new(section.address() as u32)
            } else {
                Vram::new(rom.inner())
            };

            match progbits {
                ProgbitsType::Text => continue,
                ProgbitsType::Data => utils::pretty_unwrap(global_segment.preheat_data(
                    global_config,
                    data_settings,
                    name,
                    raw_bytes,
                    rom,
                    vram,
                )),
                ProgbitsType::Rodata => utils::pretty_unwrap(global_segment.preheat_rodata(
                    global_config,
                    data_settings,
                    name,
                    raw_bytes,
                    rom,
                    vram,
                )),
                ProgbitsType::Got => continue,
                ProgbitsType::Unknown => {
                    eprintln!("Unknown progbits: {}", utils::pretty_unwrap(section.name()))
                }
            }
        }
    }
}

fn create_context(
    elf_file: &ElfFile32,
    global_ranges: RomVramRange,
    executable_settings: &ExecutableSectionSettings,
    data_settings: &DataSectionSettings,
) -> Context {
    let mips_reginfo = MipsReginfo::parse_from_elf(elf_file);
    let dynamic_section = DynamicSection::parse_from_elf(elf_file);
    let gp_value = if let Some(ri_gp_value) = mips_reginfo.and_then(|x| x.ri_gp_value()) {
        Some(ri_gp_value)
    } else {
        dynamic_section.map(|x| x.canonical_gp())
    };
    let gp_config = if let Some(gp) = gp_value {
        println!("{:?}", gp);
        Some(GpConfig::new_pic(gp))
    } else {
        println!("No gp value found.");
        None
    };
    let raw_got_info = gather_raw_got(elf_file);

    let global_config =
        GlobalConfig::new(utils::endian_to_endian(elf_file.endian())).with_gp_config(gp_config);

    print!("    symbols");
    let start = utils::get_time_now();
    // TODO: an object can have both a dynsym and a symtab
    let (mut global_segment, user_segment) =
        if let (Some(dynamic), Some((got_vram, raw_got))) = (dynamic_section, raw_got_info) {
            fill_dyn_symbols(
                elf_file,
                global_ranges,
                &global_config,
                &dynamic,
                got_vram,
                &raw_got,
            )
        } else {
            fill_symtab(elf_file, global_ranges)
        };
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("    preheat sections");
    let start = utils::get_time_now();
    preheat_sections(
        elf_file,
        &mut global_segment,
        &global_config,
        executable_settings,
        data_settings,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    utils::pretty_unwrap(ContextBuilder::new(global_segment, user_segment).build(global_config))
}

fn create_sections(
    elf_file: &ElfFile32,
    context: &mut Context,
    executable_settings: ExecutableSectionSettings,
    data_settings: DataSectionSettings,
    noload_settings: NoloadSectionSettings,
) -> (Vec<ExecutableSection>, Vec<DataSection>, Vec<NoloadSection>) {
    let mut executable_sections = Vec::new();
    let mut data_sections = Vec::new();
    let mut noload_sections = Vec::new();

    let global_ranges = context.global_segment().rom_vram_range();
    let parent_segment_info = ParentSegmentInfo::new(
        global_ranges.rom().start(),
        global_ranges.vram().start(),
        None,
    );

    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        if let Some(ElfSectionType::Progbits(ProgbitsType::Text)) = section_type {
            let name = utils::pretty_unwrap(section.name());
            let raw_bytes = utils::pretty_unwrap(section.data());
            let rom = Rom::new(
                section
                    .elf_section_header()
                    .sh_offset
                    .get(elf_file.endian()),
            );
            let section_address = section.address();
            let vram = if section_address != 0 {
                Vram::new(section.address() as u32)
            } else {
                Vram::new(rom.inner())
            };

            executable_sections.push(utils::pretty_unwrap(context.create_section_text(
                &executable_settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info.clone(),
            )));
        }
    }

    for section in elf_file.sections() {
        let sh_flags = section.elf_section_header().sh_flags.get(elf_file.endian());
        let sh_type = section.elf_section_header().sh_type.get(elf_file.endian());

        let section_type =
            ElfSectionType::new(sh_type, sh_flags, utils::pretty_unwrap(section.name()));
        match section_type {
            Some(ElfSectionType::Progbits(progbits)) => {
                let name = utils::pretty_unwrap(section.name());
                let raw_bytes = utils::pretty_unwrap(section.data());
                let rom = Rom::new(
                    section
                        .elf_section_header()
                        .sh_offset
                        .get(elf_file.endian()),
                );
                let section_address = section.address();
                let vram = if section_address != 0 {
                    Vram::new(section.address() as u32)
                } else {
                    Vram::new(rom.inner())
                };

                match progbits {
                    ProgbitsType::Text => continue,
                    ProgbitsType::Data => {
                        data_sections.push(utils::pretty_unwrap(context.create_section_data(
                            &data_settings,
                            name,
                            raw_bytes,
                            rom,
                            vram,
                            parent_segment_info.clone(),
                        )))
                    }
                    ProgbitsType::Rodata => {
                        data_sections.push(utils::pretty_unwrap(context.create_section_rodata(
                            &data_settings,
                            name,
                            raw_bytes,
                            rom,
                            vram,
                            parent_segment_info.clone(),
                        )))
                    }
                    ProgbitsType::Got => continue,
                    ProgbitsType::Unknown => {
                        eprintln!("Unknown progbits: {}", utils::pretty_unwrap(section.name()))
                    }
                }
            }
            Some(ElfSectionType::Nobits) => {
                let name = utils::pretty_unwrap(section.name());
                let section_address = section.address();
                let vram = if section_address != 0 {
                    Vram::new(section.address() as u32)
                } else {
                    Vram::new(
                        section
                            .elf_section_header()
                            .sh_offset
                            .get(elf_file.endian()),
                    )
                };
                let vram_end = vram + Size::new(section.size() as u32);

                noload_sections.push(utils::pretty_unwrap(context.create_section_bss(
                    &noload_settings,
                    name,
                    AddressRange::new(vram, vram_end),
                    parent_segment_info.clone(),
                )))
            }
            _ => {}
        }
    }

    (executable_sections, data_sections, noload_sections)
}

fn gather_relocs(elf_file: &ElfFile32) -> BTreeMap<Rom, RelocationInfo> {
    let _avoid_warning = elf_file;
    let user_relocs = BTreeMap::new();

    // TODO

    #[expect(clippy::let_and_return)]
    user_relocs
}

fn post_process_sections(
    context: &mut Context,
    user_relocs: BTreeMap<Rom, RelocationInfo>,
    executable_sections: Vec<ExecutableSection>,
    data_sections: Vec<DataSection>,
    noload_sections: Vec<NoloadSection>,
) -> (
    Vec<ExecutableSectionProcessed>,
    Vec<DataSectionProcessed>,
    Vec<NoloadSectionProcessed>,
) {
    let executable_sections = utils::pretty_unwrap(
        executable_sections
            .into_iter()
            .map(|x| x.post_process(context, &user_relocs))
            .collect::<Result<Vec<ExecutableSectionProcessed>, SectionPostProcessError>>(),
    );
    let data_sections = utils::pretty_unwrap(
        data_sections
            .into_iter()
            .map(|x| x.post_process(context, &user_relocs))
            .collect::<Result<Vec<DataSectionProcessed>, SectionPostProcessError>>(),
    );
    let noload_sections = utils::pretty_unwrap(
        noload_sections
            .into_iter()
            .map(|x| x.post_process(context))
            .collect::<Result<Vec<NoloadSectionProcessed>, SectionPostProcessError>>(),
    );

    (executable_sections, data_sections, noload_sections)
}

fn write_sections_to_files(
    output_dir: PathBuf,
    context: Context,
    executable_sections: Vec<ExecutableSectionProcessed>,
    data_sections: Vec<DataSectionProcessed>,
    noload_sections: Vec<NoloadSectionProcessed>,
) {
    utils::pretty_unwrap(fs::create_dir_all(&output_dir));

    let func_display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    for section in executable_sections {
        let name = section.name();
        let sep = if name.starts_with(".") { "" } else { "." };
        let filename = format!("{}{}{}.s", section.vram_range().start(), sep, name);
        let outpath = output_dir.join(filename);

        let mut asm_file = BufWriter::new(utils::pretty_unwrap(fs::File::create(outpath)));
        utils::pretty_unwrap(write!(
            asm_file,
            ".include \"macro.inc\"

/* assembler directives */
.set noat      /* allow manual use of $at */
.set noreorder /* do not insert nops after branches */

.section {}

.align 4
",
            name
        ));
        for symbol in section.functions() {
            utils::pretty_unwrap(write!(
                asm_file,
                "\n{}",
                utils::pretty_unwrap(symbol.display(&context, &func_display_settings))
            ));
        }
    }

    let data_display_settings = SymDataDisplaySettings::new();
    for section in data_sections {
        let name = section.name();
        let sep = if name.starts_with(".") { "" } else { "." };
        let filename = format!("{}{}{}.s", section.vram_range().start(), sep, name);
        let outpath = output_dir.join(filename);

        let mut asm_file = BufWriter::new(utils::pretty_unwrap(fs::File::create(outpath)));
        utils::pretty_unwrap(write!(
            asm_file,
            ".include \"macro.inc\"

.section {}

.align 4
",
            name
        ));
        for symbol in section.data_symbols() {
            utils::pretty_unwrap(write!(
                asm_file,
                "\n{}",
                utils::pretty_unwrap(symbol.display(&context, &data_display_settings))
            ));
        }
    }

    let noload_display_settings = SymNoloadDisplaySettings::new();
    for section in noload_sections {
        let name = section.name();
        let sep = if name.starts_with(".") { "" } else { "." };
        let filename = format!("{}{}{}.s", section.vram_range().start(), sep, name);
        let outpath = output_dir.join(filename);

        let mut asm_file = BufWriter::new(utils::pretty_unwrap(fs::File::create(outpath)));
        utils::pretty_unwrap(write!(
            asm_file,
            ".include \"macro.inc\"

.section {}

.align 4
",
            name
        ));
        for symbol in section.noload_symbols() {
            utils::pretty_unwrap(write!(
                asm_file,
                "\n{}",
                utils::pretty_unwrap(symbol.display(&context, &noload_display_settings))
            ));
        }
    }
}

fn main() {
    let args = Args::parse();

    println!("input path: {:?}", args.input_path);

    print!("Reading elf");
    let start = utils::get_time_now();
    let binary_data = {
        let mut buf = Vec::new();
        let f = File::open(args.input_path).expect("Input file not found");
        BufReader::new(f)
            .read_to_end(&mut buf)
            .expect("Error reading the file");
        buf
    };
    let elf_file = utils::read_elf(&binary_data);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    // print_elf_stuff(&elf_file);

    print!("global ranges");
    let start = utils::get_time_now();
    let global_ranges = create_global_ranges(&elf_file);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    let compiler = args.compiler.map(|x| x.into());
    // TODO: proper InstructionFlags
    let executable_settings = ExecutableSectionSettings::new(
        compiler,
        InstructionFlags::new(IsaVersion::MIPS_III).with_j_as_branch(false),
    );
    // Since we don't have file splits information we allow late rodata strings because late rodata
    // start detection will be borked either way.
    let string_guesser_flags = StringGuesserFlags::default()
        .union(StringGuesserFlags::AllowLateRodata)
        .union(StringGuesserFlags::IgnoreDetectedType);
    let data_settings =
        DataSectionSettings::new(compiler).with_string_guesser_flags(string_guesser_flags);
    let noload_settings = NoloadSectionSettings::new(compiler);

    println!("context:");
    let start = utils::get_time_now();
    let mut context = create_context(
        &elf_file,
        global_ranges,
        &executable_settings,
        &data_settings,
    );
    let end = utils::get_time_now();
    println!("  {:?}", end - start);

    print!("create_sections");
    let start = utils::get_time_now();
    let (executable_sections, data_sections, noload_sections) = create_sections(
        &elf_file,
        &mut context,
        executable_settings,
        data_settings,
        noload_settings,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("user_relocs");
    let start = utils::get_time_now();
    let user_relocs = gather_relocs(&elf_file);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("post_process_sections");
    let start = utils::get_time_now();
    let (executable_sections, data_sections, noload_sections) = post_process_sections(
        &mut context,
        user_relocs,
        executable_sections,
        data_sections,
        noload_sections,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("write_sections_to_files");
    let start = utils::get_time_now();
    write_sections_to_files(
        args.output_dir,
        context,
        executable_sections,
        data_sections,
        noload_sections,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);
}
