#[macro_use]
use crate::*;

struct WordConst;
impl WordConst {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x0B);
		3
	}
}

struct ComputationalData;
impl ComputationalData {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x0B => WordConst::get_len(data),
			_ => todo!("{}", data[0])
		}
	}
}

struct DataObject;
impl DataObject {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x0A..=0x0E | 0 | 1 | 0xFF | 0x11 => ComputationalData::get_len(data),
			0x5B => match data[1] {
				0x30 => ComputationalData::get_len(data),
				_ => todo!("{}", data[1])
			}
			_ => todo!("{}", data[0])
		}
	}
}

struct TermArg;
impl TermArg {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x0A | 0x0B | 0x0C | 0x0E | 0x0D | 0 | 1 | 0xFF | 0x11 | 0x12 | 0x13 => DataObject::get_len(data),
			0x5B => match data[1] {
				0x30 => DataObject::get_len(data),
				_ => todo!("{}", data[1])
			}
			_ => todo!("{}", data[0])
		}
	}
}

struct RegionSpace;
impl RegionSpace {
	fn get_len(data : &[u8]) -> usize {
		1
	}
}

struct RegionOffset;
impl RegionOffset {
	fn get_len(data : &[u8]) -> usize {
		TermArg::get_len(data)
	}
}

struct RegionLen;
impl RegionLen {
	fn get_len(data : &[u8]) -> usize {
		TermArg::get_len(data)
	}
}

struct DefOpRegion;
impl DefOpRegion {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x5B);
		assert_eq!(data[1], 0x80);
		let elem1_len = NameString::get_len(&data[2..]);
		let elem2_len = RegionSpace::get_len(&data[2+elem1_len..]);
		let elem3_len = RegionOffset::get_len(&data[2+elem1_len+elem2_len..]);
		let elem4_len = RegionLen::get_len(&data[2+elem1_len+elem2_len+elem3_len..]);
		2 + elem1_len + elem2_len + elem3_len + elem4_len
	}
}

struct PkgLength;
impl PkgLength {
	fn get_len(data : &[u8]) -> usize {
		let lead_byte = data[0];
		let following_amount = (lead_byte & 0b11000000) >> 6;
		1 + following_amount as usize
	}
}

struct DefName;
impl DefName {
	fn get_len(data : &[u8]) -> usize {
		todo!();
	}
}

struct DefScope;
impl DefScope {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x10);
		let elem1_len = PkgLength::get_len(&data[1..]);
		let elem2_len = NameString::get_len(&data[1+elem1_len..]);
		let elem3_len = TermList::get_len(&data[1+elem1_len+elem2_len..]);
		1 + elem1_len + elem2_len + elem3_len
	}
}
struct NamedObj;
impl NamedObj {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			// ExtOp
			0x5B => match data[1] {
				0x80 => DefOpRegion::get_len(data),
				_ => todo!("{}", data[1]),
			}
			_ => todo!("{}", data[0])
		}
	}
}
struct Statement;
impl Statement {
	fn get_len(data : &[u8]) -> usize {
		todo!();
	}
}

struct Expression;
impl Expression {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			_ => todo!("{}", data[0])
		}
	}
}

struct NameSeg;
impl NameSeg {
	fn get_len(data : &[u8]) -> usize {
		assert!((0x41..=0x5A).contains(&data[0]) || data[0] == 0x5F);
		for i in 1..=3 {
			assert!((0x41..=0x5A).contains(&data[i]) || data[i] == 0x5F || (0x30..=0x39).contains(&data[i]));
		}
		4
	}
}

struct DualNamePath;
impl DualNamePath {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x2E);
		let elem1_len = NameSeg::get_len(&data[1..]);
		let elem2_len = NameSeg::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len
	}
}

struct MultiNamePath;
impl MultiNamePath {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x2F);
		let seg_count = data[1];
		assert!(seg_count > 0);
		let mut offset = 0;
		for name_seg in 0..seg_count {
			offset += NameSeg::get_len(&data[2+offset..]);
		}
		2 + offset
	}
}

struct NamePath;
impl NamePath {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x41..=0x5A | 0x5F => NameSeg::get_len(data),
			0x2E => DualNamePath::get_len(data),
			0x2F => MultiNamePath::get_len(data),
			0 => 1,
			_ => panic!("Invalid name path")
		}
	}
}

struct NameString;
impl NameString {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x5C => 1 + NamePath::get_len(&data[1..]),
			0x5E => {
				let mut prefix_count = 1;
				while data[prefix_count] == 0x5E {
					prefix_count += 1;
				}
				prefix_count + NamePath::get_len(&data[prefix_count..])
			},
			0x41..=0x5A | 0x5F | 0x2E | 0x2F | 0 => NamePath::get_len(data),
			_ => panic!("Invalid name string")
		}
	}
}

struct DefAlias;
impl DefAlias {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x06);
		let elem1_len = NameString::get_len(&data[1..]);
		let elem2_len = NameString::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len
	}
}

struct NameSpaceModifierObj;
impl NameSpaceModifierObj {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x06 => DefAlias::get_len(data),
			0x08 => DefName::get_len(data),
			0x10 => DefScope::get_len(data),
			_ => panic!("Invalid namespace modifier object")
		}
	}
}

struct Object;
impl Object {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x06 | 0x08 | 0x10 => NameSpaceModifierObj::get_len(data),
			0x5B => match data[1] {
				0x87 | 0x13 | 0x88 | 0x80 | 0x84 | 0x85 => NamedObj::get_len(data),
				_ => panic!("Invalid object")
			}
			0x8A..=0x8D | 0x8F | 0x15 => NamedObj::get_len(data),
			_ => panic!("Invalid object")
		}
	}
}

struct TermObject;
impl TermObject {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x06 | 0x08 | 0x10 | 0x8A..=0x8D | 0x8F | 0x15 => Object::get_len(data),
			0xCC | 0x9F | 0xA0 | 0x86 | 0xA2..=0xA5 => Statement::get_len(data),
			0x70..=0x85 | 0x87..=0x89 | 0x90..=0x99 | 0x11..=0x13 | 0x9C..=0x9E | 0x8E | 0x5C | 0x5E | 0x41..=0x5A | 0x5F | 0x2E | 0x2F | 0 => Expression::get_len(data),
			// ExtOp
			0x5B => match data[1] {
				0x87 | 0x13 | 0x88 | 0x80 | 0x84 | 0x85 => NamedObj::get_len(data),
				0x32 | 0x27 | 0x26 | 0x24 | 0x22 | 0x21 => Statement::get_len(data),
				0x23 | 0x12 | 0x28 | 0x1F | 0x33 | 0x29 | 0x25 => Expression::get_len(data),
				_ => panic!("Invalid term object")
			},
			_ => panic!("Invalid term object")
		}
	}
}

struct TermList;
impl TermList {
	fn get_len(data : &[u8]) -> usize {
		let mut index = 0;
		print!("Parsing term list\n");
		while index < data.len() {
			print!("Index: {index}\n");
			index += TermObject::get_len(&data[index..]);
		}
		assert_eq!(index, data.len());
		index
	}
}

pub fn parse_aml(data : &[u8]) {
	for b in 0..100 {
		print!("{:x} ", data[b]);
	}
	print!("\n");
	let len = TermList::get_len(data);
}
