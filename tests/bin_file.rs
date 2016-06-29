extern crate elf;

use std::string::FromUtf8Error;
use std::path::Path;

#[derive(Debug)]
pub enum ParseError {
  ElfError(elf::ParseError),
}

pub struct Symbol {
  pub name: String,
  pub value: u64,
}

pub trait BinFile {
  fn objects(&self) -> Vec<Symbol>;
  fn read_str(&self, addr: u64) -> Result<String, FromUtf8Error>;
  fn get_symbol(&self, name: &str) -> Option<Symbol>;
}

struct ElfFile {
  elf: elf::File,
}

impl ElfFile {
  fn from_file(path: &Path) -> Result<ElfFile, ParseError> {
    match elf::File::open_path(path) {
      Ok(elf) => Ok(ElfFile { elf: elf }),
      Err(e) => Err(ParseError::ElfError(e)),
    }
  }
}

impl BinFile for ElfFile {
  fn objects(&self) -> Vec<Symbol> {
    let symtab = self.elf.get_section(".symtab").unwrap();
    let symbols = self.elf.get_symbols(symtab).unwrap().into_iter();
    symbols.filter(|s| s.symtype == elf::types::STT_OBJECT)
      .map(|s| {
        Symbol {
          name: s.name.clone(),
          value: s.value,
        }
      })
      .collect()
  }

  fn read_str(&self, addr: u64) -> Result<String, FromUtf8Error> {
    let section = self.elf.get_section(".rodata").unwrap();
    let section_va = section.shdr.addr;
    if section_va > addr || addr > section_va + section.shdr.size {
      panic!("{} not in .rodata", addr);
    }
    elf::utils::get_string(section.data.as_slice(),
                           (addr - section_va) as usize)
  }

  fn get_symbol(&self, name: &str) -> Option<Symbol> {
    let symtab = self.elf.get_section(".symtab").unwrap();
    let mut symbols = self.elf.get_symbols(symtab).unwrap().into_iter();
    match symbols.find(|s| s.name == name) {
      Some(s) => {
        Some(Symbol {
          name: s.name.clone(),
          value: s.value,
        })
      }
      None => None,
    }
  }
}

pub fn load(path: &Path) -> Result<Box<BinFile>, ParseError> {
  let file = try!(ElfFile::from_file(path));
  Ok(Box::new(file))
}
