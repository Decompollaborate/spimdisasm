/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use clap::{error::Result, Parser};
use elf_symbol::{ElfSymSectionIndex, ElfSymType};
use object::{
    self, elf,
    read::elf::{ElfFile32, FileHeader},
    Object, ObjectSection, ObjectSymbol,
};
use parsed_elf::ParsedElf;
use spimdisasm::{
    self,
    addresses::{AddressRange, Rom, RomVramRange, UserSize, Vram},
    analysis::StringGuesserFlags,
    config::{GlobalConfig, GlobalConfigBuilder, GpConfig},
    context::{
        builder::{GlobalSegmentHeater, UserSegmentBuilder},
        Context, ContextBuilder, GlobalSegmentBuilder,
    },
    metadata::{GotAccessKind, LabelType, SymbolType},
    parent_segment_info::ParentSegmentInfo,
    rabbitizer::{InstructionDisplayFlags, InstructionFlags, IsaVersion},
    relocation::RelocationInfo,
    sections::{
        before_proc::{
            DataSection, DataSectionSettings, ExecutableSection, ExecutableSectionSettings,
            NobitsSection, NobitsSectionSettings,
        },
        processed::{DataSectionProcessed, ExecutableSectionProcessed, NobitsSectionProcessed},
        Section, SectionPostProcessError,
    },
    symbols::display::{FunctionDisplaySettings, SymDataDisplaySettings, SymNobitsDisplaySettings},
};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

mod dynamic_section;
mod elf_section_type;
mod elf_symbol;
mod mips_reginfo;
mod parsed_elf;
mod utils;

use elf_section_type::ProgbitsType;

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

fn parse_elf(input_path: &Path) -> ParsedElf {
    let binary_data = {
        let mut buf = Vec::new();
        let f = File::open(input_path).expect("Input file not found");
        BufReader::new(f)
            .read_to_end(&mut buf)
            .expect("Error reading the file");
        buf
    };
    let elf_file = utils::read_elf(&binary_data);

    // print_elf_stuff(&elf_file);

    ParsedElf::parse_elf(elf_file)
}

fn create_global_ranges(elf: &ParsedElf) -> RomVramRange {
    let mut rom_start = None;
    let mut rom_end = None;
    let mut vram_start = None;
    let mut vram_end = None;

    for section in elf.executable_sections() {
        let size = section.size();
        let rom = section.offset();
        let r_end = rom + size;
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(section.offset().inner())
            }
        };
        let v_end = vram + size;

        if let Some(x) = vram_start {
            vram_start = Some(vram.min(x));
        } else {
            vram_start = Some(vram);
        }
        if let Some(x) = vram_end {
            vram_end = Some(v_end.max(x));
        } else {
            vram_end = Some(v_end);
        }

        if let Some(x) = rom_start {
            rom_start = Some(rom.min(x));
        } else {
            rom_start = Some(rom);
        }
        if let Some(x) = rom_end {
            rom_end = Some(r_end.max(x));
        } else {
            rom_end = Some(r_end);
        }
    }

    for section in elf.data_sections() {
        let size = section.size();
        let rom = section.offset();
        let r_end = rom + size;
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(section.offset().inner())
            }
        };
        let v_end = vram + size;

        if let Some(x) = vram_start {
            vram_start = Some(vram.min(x));
        } else {
            vram_start = Some(vram);
        }
        if let Some(x) = vram_end {
            vram_end = Some(v_end.max(x));
        } else {
            vram_end = Some(v_end);
        }

        if let Some(x) = rom_start {
            rom_start = Some(rom.min(x));
        } else {
            rom_start = Some(rom);
        }
        if let Some(x) = rom_end {
            rom_end = Some(r_end.max(x));
        } else {
            rom_end = Some(r_end);
        }
    }

    for section in elf.nobits_sections() {
        let size = section.size();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(section.offset().inner())
            }
        };
        let v_end = vram + size;

        if let Some(x) = vram_start {
            vram_start = Some(vram.min(x));
        } else {
            vram_start = Some(vram);
        }
        if let Some(x) = vram_end {
            vram_end = Some(v_end.max(x));
        } else {
            vram_end = Some(v_end);
        }
    }

    let rom = AddressRange::new(rom_start.unwrap(), rom_end.unwrap());
    let vram = AddressRange::new(vram_start.unwrap(), vram_end.unwrap());

    RomVramRange::new(rom, vram)
}

fn fill_symbols(
    elf: &ParsedElf,
    global_ranges: RomVramRange,
    global_config: &GlobalConfig,
) -> (GlobalSegmentHeater, UserSegmentBuilder) {
    let mut global_segment = GlobalSegmentBuilder::new(global_ranges);
    let mut user_segment = UserSegmentBuilder::new();

    let mut initials = HashSet::new();
    let mut remaining_symbols = Vec::new();

    // Silly hack to allow strings starting with `0x0A` (\n) or `0x09` (\t) to be detected as strings.
    // We need to do this because otherwise spimdisasm will think those values look like addresses,
    // because they happen to be in the middle of the non contiguous address space of the elf.
    utils::pretty_unwrap(global_segment.add_ignored_address_range(
        Vram::new(0x09000000),
        const { UserSize::new_checked(0x00800000).unwrap() },
    ));
    utils::pretty_unwrap(global_segment.add_ignored_address_range(
        Vram::new(0x0A000000),
        const { UserSize::new_checked(0x00800000).unwrap() },
    ));

    for global_entry in elf.got_global_symbols() {
        let got_entry = global_entry.got_entry();
        let initial = got_entry.initial();
        if initial >= 0x10 {
            let initial_vram = Vram::new(initial);
            if !initials.contains(&initial_vram) {
                let elf_sym = global_entry.elf_sym();

                let sym_type = match elf_sym.typ() {
                    ElfSymType::Function => Some(SymbolType::Function),
                    _ => None,
                };

                // This size seems to only be valid for `initial` for `UNDEF` and `COM` symbols.
                let size = elf_sym
                    .size()
                    .filter(|_| {
                        matches!(
                            elf_sym.section_index(),
                            ElfSymSectionIndex::Undef | ElfSymSectionIndex::Common
                        ) && !matches!(elf_sym.typ(), ElfSymType::Function)
                    })
                    .unwrap_or(const { UserSize::new_checked(1).unwrap() });

                let mut sym_metadata = utils::pretty_unwrap(user_segment.add_user_symbol(
                    initial_vram,
                    got_entry.sym_name(),
                    size,
                    sym_type,
                ));
                sym_metadata.set_got_access_kind(GotAccessKind::Global);

                initials.insert(initial_vram);
            }
        }
    }

    let mut added_labels = HashSet::new();
    for (sym_value, syms_per_value) in elf.symbols() {
        if *sym_value < 0x10 {
            continue;
        }

        let mut iter = syms_per_value.syms().iter();

        if let Some(sym) = iter.next() {
            let sym_type = match sym.typ() {
                ElfSymType::Function => Some(SymbolType::Function),
                ElfSymType::NoType => continue,
                _ => None,
            };

            let name = sym.name();
            let vram = Vram::new(sym.value());
            let rom = None;
            let size = {
                sym.size()
                    .filter(|_| !matches!(sym.section_index(), ElfSymSectionIndex::Undef))
            };

            if global_ranges.in_vram_range(vram) {
                let mut sym_metadata = utils::pretty_unwrap(
                    global_segment.add_user_symbol(name, vram, rom, size, sym_type),
                );
                if sym.is_got_global() {
                    sym_metadata.set_got_access_kind(GotAccessKind::Global);
                }
                if let Some(bind) = sym.bind().as_str() {
                    sym_metadata.set_visibility(bind);
                }
            } else {
                remaining_symbols.push(sym);
            }
        }

        if let Some(label_sym) = iter.next() {
            if label_sym.typ() == ElfSymType::NoType {
                continue;
            }

            let name = label_sym.name();
            let vram = Vram::new(label_sym.value());
            let rom = None;

            let mut label = utils::pretty_unwrap(global_segment.add_user_label(
                name,
                vram,
                rom,
                LabelType::AlternativeEntry,
            ));
            if let Some(bind) = label_sym.bind().as_str() {
                label.set_visibility(bind);
            }
            added_labels.insert(sym_value);
        }
    }

    for (label_value, labels_per_value) in elf.labels() {
        if added_labels.contains(label_value) {
            continue;
        }

        if let Some(label_sym) = labels_per_value.syms().iter().next() {
            if label_sym.typ() == ElfSymType::NoType {
                continue;
            }

            let name = label_sym.name();
            let vram = Vram::new(label_sym.value());
            let rom = None;

            let mut label = utils::pretty_unwrap(global_segment.add_user_label(
                name,
                vram,
                rom,
                LabelType::AlternativeEntry,
            ));
            if let Some(bind) = label_sym.bind().as_str() {
                label.set_visibility(bind);
            }
            added_labels.insert(label_value);
        }
    }

    for sym in remaining_symbols {
        eprintln!("Unhandled symbol: {:#08X?}", sym);
    }

    if let Some(global_offset_table) = elf.got() {
        // First entry of the GOT is reserved for the lazy resolver
        if let Some(lazy_resolver) = global_offset_table.locals().first() {
            // ido 7.1 ld has its lazy resolver set to 0?
            let vram = Vram::new(lazy_resolver.inner());
            if vram != Vram::new(0) {
                // Something silly, so the user doesn't confuse this as a real symbol
                let name = "$$.LazyResolver";
                let size = const { UserSize::new_checked(4).unwrap() };
                let typ = None;

                let mut sym_metadata =
                    utils::pretty_unwrap(user_segment.add_user_symbol(vram, name, size, typ));
                // I'm not sure if this should be considered Local or Global.
                // Maybe make a new kind for this?
                sym_metadata.set_got_access_kind(GotAccessKind::Local);
            }
        }

        utils::pretty_unwrap(
            global_segment.add_global_offset_table(global_config, global_offset_table.clone()),
        );
    }

    (global_segment.finish_symbols(), user_segment)
}

fn preheat_sections(
    elf: &ParsedElf,
    global_segment: &mut GlobalSegmentHeater,
    global_config: &GlobalConfig,
    executable_settings: &ExecutableSectionSettings,
    data_settings: &DataSectionSettings,
) {
    // Executable sections first
    for section in elf.executable_sections() {
        let name = section.name();
        let raw_bytes = section.data();
        let rom = section.offset();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(section.offset().inner())
            }
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

    // Data sections later
    for section in elf.data_sections() {
        let progbits = section.progbits_type();
        let name = section.name();
        let raw_bytes = section.data();
        let rom = section.offset();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(section.offset().inner())
            }
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
                eprintln!("Unknown progbits: {}", name);
            }
        }
    }
}

fn create_context(
    elf: &ParsedElf,
    global_ranges: RomVramRange,
    executable_settings: &ExecutableSectionSettings,
    data_settings: &DataSectionSettings,
) -> Context {
    let gp_value = elf.gp_value();
    let gp_config = if let Some(gp) = gp_value {
        println!("{:?}", gp);
        Some(GpConfig::new_pic(gp))
    } else {
        println!("No gp value found.");
        None
    };

    let global_config = GlobalConfigBuilder::new(elf.endian())
        .with_gp_config(gp_config)
        .build();

    print!("    symbols");
    let start = utils::get_time_now();
    let (mut global_segment, user_segment) = fill_symbols(elf, global_ranges, &global_config);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("    preheat sections");
    let start = utils::get_time_now();
    preheat_sections(
        elf,
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
    elf: &ParsedElf,
    context: &mut Context,
    executable_settings: ExecutableSectionSettings,
    data_settings: DataSectionSettings,
    nobits_settings: NobitsSectionSettings,
) -> (Vec<ExecutableSection>, Vec<DataSection>, Vec<NobitsSection>) {
    let mut executable_sections = Vec::new();
    let mut data_sections = Vec::new();
    let mut nobits_sections = Vec::new();

    let global_ranges = context.global_segment().rom_vram_range();
    let parent_segment_info = ParentSegmentInfo::new(
        global_ranges.rom().start(),
        global_ranges.vram().start(),
        None,
    );

    for elf_section in elf.executable_sections() {
        let name = elf_section.name();
        let raw_bytes = elf_section.data();
        let rom = elf_section.offset();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = elf_section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(elf_section.offset().inner())
            }
        };

        let section = utils::pretty_unwrap(context.create_section_text(
            &executable_settings,
            name,
            raw_bytes,
            rom,
            vram,
            parent_segment_info.clone(),
        ));
        executable_sections.push(section);
    }

    for elf_section in elf.data_sections() {
        let progbits = elf_section.progbits_type();
        let name = elf_section.name();
        let raw_bytes = elf_section.data();
        let rom = elf_section.offset();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = elf_section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(elf_section.offset().inner())
            }
        };

        let section = match progbits {
            ProgbitsType::Text => continue,
            ProgbitsType::Data => utils::pretty_unwrap(context.create_section_data(
                &data_settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info.clone(),
            )),
            ProgbitsType::Rodata => utils::pretty_unwrap(context.create_section_rodata(
                &data_settings,
                name,
                raw_bytes,
                rom,
                vram,
                parent_segment_info.clone(),
            )),
            ProgbitsType::Got => continue,
            ProgbitsType::Unknown => {
                eprintln!("Unknown progbits: {}", name);
                continue;
            }
        };
        data_sections.push(section);
    }

    for elf_section in elf.nobits_sections() {
        let name = elf_section.name();
        let vram = {
            // TODO: Hack to handle relocatable elfs
            // They don't have an actual vram address, so we reuse the offset instead.
            let address = elf_section.address();
            if address != Vram::new(0) {
                address
            } else {
                Vram::new(elf_section.offset().inner())
            }
        };
        let vram_end = vram + elf_section.size();

        let section = utils::pretty_unwrap(context.create_section_bss(
            &nobits_settings,
            name,
            AddressRange::new(vram, vram_end),
            parent_segment_info.clone(),
        ));
        nobits_sections.push(section);
    }

    (executable_sections, data_sections, nobits_sections)
}

fn gather_relocs(elf: &ParsedElf) -> BTreeMap<Rom, RelocationInfo> {
    let _avoid_warning = elf;
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
    nobits_sections: Vec<NobitsSection>,
) -> (
    Vec<ExecutableSectionProcessed>,
    Vec<DataSectionProcessed>,
    Vec<NobitsSectionProcessed>,
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
    let nobits_sections = utils::pretty_unwrap(
        nobits_sections
            .into_iter()
            .map(|x| x.post_process(context))
            .collect::<Result<Vec<NobitsSectionProcessed>, SectionPostProcessError>>(),
    );

    (executable_sections, data_sections, nobits_sections)
}

fn write_sections_to_files(
    output_dir: PathBuf,
    context: Context,
    executable_sections: Vec<ExecutableSectionProcessed>,
    data_sections: Vec<DataSectionProcessed>,
    nobits_sections: Vec<NobitsSectionProcessed>,
) {
    utils::pretty_unwrap(fs::create_dir_all(&output_dir));

    let func_display_settings = FunctionDisplaySettings::new(InstructionDisplayFlags::new());
    let data_display_settings = SymDataDisplaySettings::new();
    let nobits_display_settings = SymNobitsDisplaySettings::new();

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
        for symbol in section.symbols() {
            utils::pretty_unwrap(write!(
                asm_file,
                "\n{}",
                utils::pretty_unwrap(symbol.display(
                    &context,
                    &func_display_settings,
                    &data_display_settings
                ))
            ));
        }
    }

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

    for section in nobits_sections {
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
        for symbol in section.nobits_symbols() {
            utils::pretty_unwrap(write!(
                asm_file,
                "\n{}",
                utils::pretty_unwrap(symbol.display(&context, &nobits_display_settings))
            ));
        }
    }
}

fn main() {
    let args = Args::parse();

    println!("input path: {:?}", args.input_path);

    print!("Reading elf");
    let start = utils::get_time_now();
    let elf = parse_elf(&args.input_path);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("global ranges");
    let start = utils::get_time_now();
    let global_ranges = create_global_ranges(&elf);
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
        .union(StringGuesserFlags::AllowUnalignedDereferences)
        .union(StringGuesserFlags::AllowMixedAlignedDereferences)
        .union(StringGuesserFlags::AllowSingleAlignedDereferences);
    let data_settings =
        DataSectionSettings::new(compiler).with_string_guesser_flags(string_guesser_flags);
    let nobits_settings = NobitsSectionSettings::new(compiler);

    println!("context:");
    let start = utils::get_time_now();
    let mut context = create_context(&elf, global_ranges, &executable_settings, &data_settings);
    let end = utils::get_time_now();
    println!("  {:?}", end - start);

    print!("create_sections");
    let start = utils::get_time_now();
    let (executable_sections, data_sections, nobits_sections) = create_sections(
        &elf,
        &mut context,
        executable_settings,
        data_settings,
        nobits_settings,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("user_relocs");
    let start = utils::get_time_now();
    let user_relocs = gather_relocs(&elf);
    let end = utils::get_time_now();
    println!(": {:?}", end - start);

    print!("post_process_sections");
    let start = utils::get_time_now();
    let (executable_sections, data_sections, nobits_sections) = post_process_sections(
        &mut context,
        user_relocs,
        executable_sections,
        data_sections,
        nobits_sections,
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
        nobits_sections,
    );
    let end = utils::get_time_now();
    println!(": {:?}", end - start);
}
