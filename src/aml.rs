use crate::SCREEN_WRITER;
use crate::print;
use core::fmt::Write;


// ExpressionOpcode : 0 is missing: MethodInvocation of NullName makes no sense
macro_rules! get_prefix {
	(ExpressionOpcode) => {
		0x70..=0x85 | 0x87..=0x89 | 0x90..=0x99 | 0x11..=0x13 | 0x9C..=0x9E | 0x8E | 0x5C | 0x5E | 0x41..=0x5A | 0x5F | 0x2E | 0x2F
	};
}

macro_rules! get_extop_prefix {
	(ExpressionOpcode) => {
		0x23 | 0x12 | 0x28 | 0x1F | 0x33 | 0x29 | 0x25
	};
}

macro_rules! simple_concat_type {
	($new_type:ident, $($prefix:literal),*, $($other_type:ty),*) => {
		struct $new_type;
		impl $new_type {
			fn get_len(data : &[u8]) -> usize {
				const PREFIX_COUNT : usize = [$($prefix),*].len();	
				const PREFIX : [u8; PREFIX_COUNT] = [$($prefix),*];
				for i in 0..PREFIX_COUNT {
					assert_eq!(data[i], PREFIX[i]);
				}
				let mut combined_length = PREFIX_COUNT;
				$(
					combined_length += <$other_type>::get_len(&data[combined_length..]);
				)*
				combined_length
			}
		}
	};
}

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
			// ConstObj
			0x00 | 0x01 | 0xFF => 1,
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

struct ArgObj;
impl ArgObj {
	fn get_len(data : &[u8]) -> usize {
		assert!((0x68..=0x6E).contains(&data[0]));
		1
	}
}

struct TermArg;
impl TermArg {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x0A | 0x0B | 0x0C | 0x0E | 0x0D | 0 | 1 | 0xFF => DataObject::get_len(data),
			// prefix 0x11, 0x12, 0x13 are both DataObject and ExpressionOpcode so they are only mentioned once
			get_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
			0x5B => match data[1] {
				0x30 => DataObject::get_len(data),
				_ => todo!("{}", data[1])
			}
			0x60..=0x67 => LocalObj::get_len(data),
			0x68..=0x6E => ArgObj::get_len(data),
			_ => todo!("{}", data[0])
		}
	}
}

struct RegionSpace;
impl RegionSpace {
	fn get_len(_ : &[u8]) -> usize {
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

simple_concat_type!(DefOpRegion, 0x5B, 0x80, NameString, RegionSpace, RegionOffset, RegionLen);

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
	fn get_len(_ : &[u8]) -> usize {
		todo!();
	}
}

struct NamedField;
impl NamedField {
	fn get_len(data : &[u8]) -> usize {
		let elem1_len = NameSeg::get_len(data);
		let elem2_len = PkgLength::get_len(&data[elem1_len..]);
		elem1_len + elem2_len		
	}
}

struct FieldElement;
impl FieldElement {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x0 => todo!(),
			0x1 => todo!(),
			0x2 => todo!(),
			0x3 => todo!(),
			0x41..=0x5A | 0x5F => NamedField::get_len(data),
			_ => 0,
		}
	}
}

struct FieldList;
impl FieldList {
	fn get_len(data : &[u8]) -> usize {
		let mut index = 0;
		print!("Parsing field list\n");
		while index < data.len() {
			let next_offset = match data[index] {
				0x0 | 0x1 | 0x2 | 0x3 | 0x41..=0x5A | 0x5F => FieldElement::get_len(&data[index..]),
				_ => return index,
			};
			print!("Found element with length {} at index {} ", next_offset, index);
			index += next_offset;
		}
		print!("\n");
		index
	}
}

struct FieldFlags;
impl FieldFlags {
	fn get_len(_ : &[u8]) -> usize {
		1
	}
}

simple_concat_type!(DefField, 0x5B, 0x81, PkgLength, NameString, FieldFlags, FieldList);
simple_concat_type!(DefScope, 0x10, PkgLength, NameString, TermList);

struct MethodFlags;
impl MethodFlags {
	fn get_len(_ : &[u8]) -> usize {
		1
	}
}

simple_concat_type!(DefMethod, 0x14, PkgLength, NameString, MethodFlags, TermList);

struct NamedObj;
impl NamedObj {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			// ExtOp
			0x5B => match data[1] {
				0x80 => DefOpRegion::get_len(data),
				0x81 => DefField::get_len(data),
				_ => todo!("{}", data[1]),
			}
			0x14 => DefMethod::get_len(data),
			_ => todo!("{}", data[0])
		}
	}
}

struct Predicate;
impl Predicate {
	fn get_len(data : &[u8]) -> usize {
		TermArg::get_len(data)
	}
}

simple_concat_type!(DefWhile, 0xA2, PkgLength, Predicate, TermList);

struct StatementOpcode;
impl StatementOpcode {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0xA2 => DefWhile::get_len(data),
			_ => todo!("{:#x}", data[0]),
		}
	}
}

struct Operand;
impl Operand {
	fn get_len(data : &[u8]) -> usize {
		TermArg::get_len(data)
	}
}

struct LocalObj;
impl LocalObj {
	fn get_len(data : &[u8]) -> usize {
		assert!((0x60..=0x67).contains(&data[0]));
		1
	}
}

struct SimpleName;
impl SimpleName {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x60..=0x67 => LocalObj::get_len(data),
			_ => todo!()
		}
	}
}

struct SuperName;
impl SuperName {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x60..=0x67 => SimpleName::get_len(data),
			_ => todo!(),
		}
	}
}

struct Target;
impl Target {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0 => 1,
			0x60..=0x67 => SuperName::get_len(data),
			0x5B => match data[1] {
				0x31 => SuperName::get_len(data),
				_ => todo!("{}", data[1]),
			}
			_ => todo!("{:x}", data[0]),
		}
	}
}

struct DefToBuffer;
impl DefToBuffer {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x96);
		let elem1_len = Operand::get_len(&data[1..]);
		let elem2_len = Target::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len		
	}
}

struct DefSubtract;
impl DefSubtract {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x74);
		let elem1_len = Operand::get_len(&data[1..]);
		let elem2_len = Operand::get_len(&data[1+elem1_len..]);
		let elem3_len = Target::get_len(&data[1+elem1_len+elem2_len..]);
		1 + elem1_len + elem2_len + elem3_len
	}
}

struct DefSizeOf;
impl DefSizeOf {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x87);
		let elem1_len = SuperName::get_len(&data[1..]);
		1 + elem1_len
	}
}

struct DefStore;
impl DefStore {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x70);
		let elem1_len = TermArg::get_len(&data[1..]);
		let elem2_len = SuperName::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len
	}
}

struct DefToHexString;
impl DefToHexString {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x98);
		let elem1_len = Operand::get_len(&data[1..]);
		let elem2_len = Target::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len
	}
}

struct DefLLess;
impl DefLLess {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x95);
		let elem1_len = Operand::get_len(&data[1..]);
		let elem2_len = Operand::get_len(&data[1+elem1_len..]);
		1 + elem1_len + elem2_len
	}
}

struct ExpressionOpcode;
impl ExpressionOpcode {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x70 => DefStore::get_len(data),
			0x74 => DefSubtract::get_len(data),
			0x87 => DefSizeOf::get_len(data),
			0x95 => DefLLess::get_len(data),
			0x96 => DefToBuffer::get_len(data),
			0x98 => DefToHexString::get_len(data),
			_ => todo!("{:#x}", data[0])
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
		for _ in 0..seg_count {
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
			0x8A..=0x8D | 0x8F | 0x14 | 0x15 => NamedObj::get_len(data),
			_ => panic!("Invalid object")
		}
	}
}

struct TermObject;
impl TermObject {
	fn get_len(data : &[u8]) -> usize {
		match data[0] {
			0x06 | 0x08 | 0x10 | 0x8A..=0x8D | 0x8F | 0x14 | 0x15 => Object::get_len(data),
			0xCC | 0x9F | 0xA0 | 0x86 | 0xA2..=0xA5 => StatementOpcode::get_len(data),
			get_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
			// ExtOp
			0x5B => match data[1] {
				0x87 | 0x13 | 0x88 | 0x80 | 0x84 | 0x85 | 0x81 => NamedObj::get_len(data),
				0x32 | 0x27 | 0x26 | 0x24 | 0x22 | 0x21 => StatementOpcode::get_len(data),
				get_extop_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
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
	assert_eq!(len, data.len());
}


/*
Errors found in the ACPI spec
NamedObject does not include DefMethod nor DefField
A lot of link errors
Backslash in root character messes up documentation
*/
