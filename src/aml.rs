use crate::print;
use crate::SCREEN_WRITER;
use core::fmt::Write;

const GLOBAL_METHODS_MAX : usize = 1024;
static mut GLOBAL_METHODS: [(*const u8, usize); GLOBAL_METHODS_MAX] = [(core::ptr::null(), 0); GLOBAL_METHODS_MAX];
static mut GLOBAL_METHOD_COUNT: usize = 0;

// TODO implement evaluated type checking

// ExpressionOpcode : 0 is missing: MethodInvocation of NullName makes no sense
macro_rules! get_prefix {
	(ExpressionOpcode) => {
		0x70..=0x85 | 0x87..=0x89 | 0x90..=0x99 | 0x11..=0x13 | 0x9C..=0x9E | 0x8E | 0x5C | 0x5E | 0x41..=0x5A | 0x5F | 0x2E | 0x2F
	};
	(NameString) => {
		0x5C | 0x5E | 0 | 0x2E | 0x2F | 0x41..=0x5A | 0x5F
	};
	(TermArg) => {
		0x60..=0x6E | get_prefix!(DataObject) | get_prefix!(ExpressionOpcode)
	};
	(DataObject) => {
		0x13 | 0x12 | get_prefix!(ComputationalData)
	};
	(ComputationalData) => {
		0xA..=0xE | 0 | 1 | 0xFF | 0x11
	};
	(DataRefObject) => {
		get_prefix!(DataObject)
	};
	(PackageElement) => {
		get_prefix!(NameString) | get_prefix!(DataRefObject)
	};
	(SuperName) => {
		get_prefix!(SimpleName) | 0x71 | 0x83 | 0x88
	};
	(SimpleName) => {
		0x60..=0x6E | get_prefix!(NameString)
	};
}

macro_rules! get_extop_prefix {
    (TermArg) => {
        get_extop_prefix!(ExpressionOpcode) | get_extop_prefix!(DataObject)
    };
    (DataObject) => {
        get_extop_prefix!(ComputationalData)
    };
    (ComputationalData) => {
        0x30
    };
    (ExpressionOpcode) => {
        0x23 | 0x12 | 0x28 | 0x1F | 0x33 | 0x29 | 0x25
    };
    (TermArg) => {
        get_extop_prefix!(DataObject) | get_extop_prefix!(ExpressionOpcode)
    };
    (SuperName) => {
        0x31
    };
}

macro_rules! simple_concat_type {
	($new_type:ident, $($prefix:literal),*, $($other_type:ty),*) => {
		struct $new_type;
		impl $new_type {
			fn get_len(data : &[u8]) -> usize {
				print!(concat!("Start parse ", stringify!($new_type), "\n"));
				const PREFIX_COUNT : usize = [$($prefix),*].len();
				const PREFIX : [u8; PREFIX_COUNT] = [$($prefix),*];
				for i in 0..PREFIX_COUNT {
					assert_eq!(data[i], PREFIX[i]);
				}
				let mut combined_length = PREFIX_COUNT;
				$(
					print!(concat!("For ", stringify!($new_type), " parse ", stringify!($other_type), "\n"));
					combined_length += <$other_type>::get_len(&data[combined_length..]);
				)*
				print!("Done parse\n");
				combined_length
			}
		}
	};
}

macro_rules! sized_concat_type {
	($new_type:ident, $($prefix:literal),*, $($other_type:ty),*) => {
		struct $new_type;
		impl $new_type {
			fn get_len(data : &[u8]) -> usize {
				print!(concat!("Start parse ", stringify!($new_type), "\n"));
				const PREFIX_COUNT : usize = [$($prefix),*].len();
				const PREFIX : [u8; PREFIX_COUNT] = [$($prefix),*];
				for i in 0..PREFIX_COUNT {
					assert_eq!(data[i], PREFIX[i]);
				}
				let mut combined_length = PREFIX_COUNT;
				let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[PREFIX_COUNT..]);
				let pkg_end = pkg_size + PREFIX_COUNT;
				combined_length += pkg_len;
				print!(concat!("For ", stringify!($new_type), " parse {} bytes\n"), pkg_size);

				for i in 0..pkg_end {
					print!("{:#x} ", data[i]);
				}
				print!("\n");

				$(
					print!(concat!("For ", stringify!($new_type), " parse ", stringify!($other_type), "\n"));
					combined_length += <$other_type>::get_len(&data[combined_length..pkg_end]);
				)*
				assert_eq!(combined_length, pkg_end);
				combined_length
			}
		}
	};
}

struct ByteData;
impl ByteData {
    fn get_len(_: &[u8]) -> usize {
        1
    }
}

struct WordData;
impl WordData {
    fn get_len(_: &[u8]) -> usize {
        2
    }
}

struct DWordData;
impl DWordData {
    fn get_len(_: &[u8]) -> usize {
        4
    }
}

type NumElements = ByteData;

struct ByteConst;
impl ByteConst {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x0A);
        2
    }
}

struct WordConst;
impl WordConst {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x0B);
        3
    }
}

struct DWordConst;
impl DWordConst {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x0C);
        5
    }
}

struct QWordConst;
impl QWordConst {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x0E);
        9
    }
}

struct String;
impl String {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x0D);
        let mut index = 1;
        while index < data.len() {
            if (0x01..=0x7F).contains(&data[index]) {
                index += 1;
            } else {
                break;
            }
        }
        assert_eq!(data[index], 0);
        index + 1
    }
}

struct ComputationalData;
impl ComputationalData {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x0A => ByteConst::get_len(data),
            0x0B => WordConst::get_len(data),
            0x0C => DWordConst::get_len(data),
            0x0D => String::get_len(data),
            0x0E => QWordConst::get_len(data),
            0x11 => DefBuffer::get_len(data),
            0x5B => match data[1] {
                // RevisionOp
                0x30 => 2,
                _ => panic!("Invalid computational data"),
            },
            // ConstObj
            0x00 | 0x01 | 0xFF => 1,
            _ => panic!("Invalid computational data {:#x}", data[0]),
        }
    }
}

struct DataObject;
impl DataObject {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            get_prefix!(ComputationalData) => ComputationalData::get_len(data),
            0x5B => match data[1] {
                get_extop_prefix!(ComputationalData) => ComputationalData::get_len(data),
                _ => panic!("Invalid data object {:#x}", data[1]),
            },
            0x12 => DefPackage::get_len(data),
            0x13 => DefVarPackage::get_len(data),
            _ => panic!("Invalid data object {:#x}", data[0]),
        }
    }
}

struct MatchOpcode;
impl MatchOpcode {
    fn get_len(data: &[u8]) -> usize {
        assert!((0..=5).contains(&data[0]));
        1
    }
}

struct ArgObj;
impl ArgObj {
    fn get_len(data: &[u8]) -> usize {
        assert!((0x68..=0x6E).contains(&data[0]));
        1
    }
}

struct TermArg;
impl TermArg {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x0A | 0x0B | 0x0C | 0x0E | 0x0D | 0 | 1 | 0xFF => DataObject::get_len(data),
            // prefix 0x11, 0x12, 0x13 are both DataObject and ExpressionOpcode so they are only mentioned once
            get_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
            0x5B => match data[1] {
                get_extop_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
                get_extop_prefix!(DataObject) => DataObject::get_len(data),
                _ => panic!("Invalid term arg"),
            },
            0x60..=0x67 => LocalObj::get_len(data),
            0x68..=0x6E => ArgObj::get_len(data),
            _ => panic!("Invalid term arg"),
        }
    }
}

struct RegionSpace;
impl RegionSpace {
    fn get_len(_: &[u8]) -> usize {
        1
    }
}

struct VarNumElements;
impl VarNumElements {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct RegionOffset;
impl RegionOffset {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct Data;
impl Data {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct RegionLen;
impl RegionLen {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

simple_concat_type!(
    DefOpRegion,
    0x5B,
    0x80,
    NameString,
    RegionSpace,
    RegionOffset,
    RegionLen
);

struct PkgLength;
impl PkgLength {
    fn get_len(data: &[u8]) -> usize {
        let lead_byte = data[0];
        let following_amount = (lead_byte & 0b11000000) >> 6;
        1 + following_amount as usize
    }
    fn get_len_and_size(data: &[u8]) -> (usize, usize) {
        let lead_byte = data[0];
        let following_amount = (lead_byte & 0b11000000) >> 6;
        let size = if following_amount == 0 {
            lead_byte as u32 & 0b111111
        } else {
            let mut dummy: u32 = lead_byte as u32 & 0b1111;
            for i in 0..following_amount as usize {
                dummy |= (data[1 + i] as u32) << (8 * i + 4);
            }
            dummy
        };
        (1 + following_amount as usize, size as usize)
    }
}

struct DataRefObject;
impl DataRefObject {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0xA..=0xE | 0 | 1 | 0xFF | 0x13 | 0x12 | 0x11 => DataObject::get_len(data),
            0x5B => match data[1] {
                0x30 => DataObject::get_len(data),
                _ => todo!("{:#x}", data[1]),
            },
            _ => todo!("{:#x}", data[0]),
        }
    }
}

simple_concat_type!(DefName, 0x08, NameString, DataRefObject);

struct NamedField;
impl NamedField {
    fn get_len(data: &[u8]) -> usize {
        let elem1_len = NameSeg::get_len(data);
        let elem2_len = PkgLength::get_len(&data[elem1_len..]);
        elem1_len + elem2_len
    }
}

type AccessType = ByteData;
type AccessAttrib = ByteData;
type ExtendedAccessAttrib = ByteData;
type AccessLength = ByteData;

simple_concat_type!(ReservedField, 0, PkgLength);
simple_concat_type!(AccessField, 1, AccessType, AccessAttrib);
simple_concat_type!(
    ExtendedAccessField,
    3,
    AccessType,
    ExtendedAccessAttrib,
    AccessLength
);

struct ConnectField;
impl ConnectField {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 2);
        todo!();
    }
}

struct FieldElement;
impl FieldElement {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x0 => ReservedField::get_len(data),
            0x1 => AccessField::get_len(data),
            0x2 => ConnectField::get_len(data),
            0x3 => ExtendedAccessField::get_len(data),
            0x41..=0x5A | 0x5F => NamedField::get_len(data),
            _ => 0,
        }
    }
}

struct FieldList;
impl FieldList {
    fn get_len(data: &[u8]) -> usize {
        let mut index = 0;
        print!("Parsing field list\n");
        while index < data.len() {
            let next_offset = match data[index] {
                0x0 | 0x1 | 0x2 | 0x3 | 0x41..=0x5A | 0x5F => FieldElement::get_len(&data[index..]),
                _ => return index,
            };
            print!(
                "Found element with length {} at index {}\n",
                next_offset, index
            );
            index += next_offset;
        }
        assert_eq!(data.len(), index);
        print!("Done parsing field list\n");
        index
    }
}

struct FieldFlags;
impl FieldFlags {
    fn get_len(_: &[u8]) -> usize {
        1
    }
}

sized_concat_type!(DefField, 0x5B, 0x81, NameString, FieldFlags, FieldList);
sized_concat_type!(DefScope, 0x10, NameString, TermList);

struct MethodFlags;
impl MethodFlags {
    fn get_len(_: &[u8]) -> usize {
        //print!("METHOD FLAGS : {:#b}\n", data[0]);
        1
    }
}

struct SyncFlags;
impl SyncFlags {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0] & 0b11110000, 0);
        1
    }
}

type MutexObject = SuperName;
type Timeout = WordData;

simple_concat_type!(DefMutex, 0x5B, 0x01, NameString, SyncFlags);


struct DefMethod;
impl DefMethod {
	fn get_len(data : &[u8]) -> usize {
		assert_eq!(data[0], 0x14);
		let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[1..]);
		let pkg_end = pkg_size + 1;
		let elem1_len = NameString::get_len(&data[1+pkg_len..pkg_end]);
		let elem2_len = MethodFlags::get_len(&data[1+pkg_len+elem1_len..pkg_end]);
		let elem3_len = TermList::get_len(&data[1+pkg_len+elem1_len+elem2_len..pkg_end]);
		//print!("!\n!\n!\n");
		unsafe {
			if GLOBAL_METHOD_COUNT == GLOBAL_METHODS_MAX {
				panic!("Cannot read more methods");
			}
			GLOBAL_METHODS[GLOBAL_METHOD_COUNT] = (&data[0], data.len());
			GLOBAL_METHOD_COUNT += 1;
			print!("There are {GLOBAL_METHOD_COUNT} methods\n");
		}

		1 + pkg_len + elem1_len + elem2_len + elem3_len
	}
}

struct DefDevice;
impl DefDevice {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x5B);
        assert_eq!(data[1], 0x82);
        let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[2..]);
        let pkg_end = pkg_size + 2;
        let elem1_len = NameString::get_len(&data[2 + pkg_len..pkg_end]);
        print!("DEVICE FOUND: ");
        NameString::print(&data[2 + pkg_len..pkg_end]);
        let elem2_len = TermList::get_len(&data[2+pkg_len+elem1_len..pkg_end]);
        assert_eq!(pkg_size, pkg_len + elem1_len + elem2_len);
        pkg_size + 2
    }
}

struct DefProcessor;
impl DefProcessor {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x5B);
        assert_eq!(data[1], 0x83);
        let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[2..]);
        NameString::get_len(&data[2 + pkg_len..]);
        print!(
            "DEPRECATED! Skipping DefProcessor ({} bytes)\n",
            pkg_size + 2
        );
        pkg_size + 2
    }
}

struct BankValue;
impl BankValue {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct SourceBuff;
impl SourceBuff {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct BitIndex;
impl BitIndex {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct ByteIndex;
impl ByteIndex {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct NumBits;
impl NumBits {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

type ObjectType = ByteData;
struct ArgumentCount;
impl ArgumentCount {
    fn get_len(data: &[u8]) -> usize {
        assert!((0..=7).contains(&data[0]));
        1
    }
}

type SystemLevel = ByteData;
type ResourceOrder = WordData;

sized_concat_type!(
    DefBankField,
    0x5B,
    0x87,
    NameString,
    NameString,
    BankValue,
    FieldFlags,
    FieldList
);
simple_concat_type!(DefCreateBitField, 0x8D, SourceBuff, BitIndex, NameString);
simple_concat_type!(DefCreateByteField, 0x8C, SourceBuff, ByteIndex, NameString);
simple_concat_type!(DefCreateDWordField, 0x8A, SourceBuff, ByteIndex, NameString);
simple_concat_type!(
    DefCreateField,
    0x5B,
    0x13,
    SourceBuff,
    BitIndex,
    NumBits,
    NameString
);
simple_concat_type!(DefCreateQWordField, 0x8F, SourceBuff, ByteIndex, NameString);
simple_concat_type!(DefCreateWordField, 0x8B, SourceBuff, ByteIndex, NameString);
simple_concat_type!(
    DefDataRegion,
    0x5B,
    0x88,
    NameString,
    TermArg,
    TermArg,
    TermArg
);
simple_concat_type!(DefExternal, 0x15, NameString, ObjectType, ArgumentCount);
sized_concat_type!(
    DefPowerRes,
    0x5B,
    0x84,
    NameString,
    SystemLevel,
    ResourceOrder,
    TermList
);
sized_concat_type!(DefThermalZone, 0x5b, 0x85, NameString, TermList);

struct NamedObj;
impl NamedObj {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            // ExtOp
            0x5B => match data[1] {
                0x01 => DefMutex::get_len(data),
                0x13 => DefCreateField::get_len(data),
                0x80 => DefOpRegion::get_len(data),
                0x81 => DefField::get_len(data),
                0x82 => DefDevice::get_len(data),
                0x83 => DefProcessor::get_len(data),
                0x84 => DefPowerRes::get_len(data),
                0x85 => DefThermalZone::get_len(data),
                0x87 => DefBankField::get_len(data),
                0x88 => DefDataRegion::get_len(data),
                _ => panic!("Invalid named object {}", data[1]),
            },
            0x14 => DefMethod::get_len(data),
            0x15 => DefExternal::get_len(data),
            0x8A => DefCreateDWordField::get_len(data),
            0x8B => DefCreateWordField::get_len(data),
            0x8C => DefCreateByteField::get_len(data),
            0x8D => DefCreateBitField::get_len(data),
            0x8F => DefCreateQWordField::get_len(data),
            _ => panic!("Invalid named object {}", data[0]),
        }
    }
}

struct Predicate;
impl Predicate {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct ArgObject;
impl ArgObject {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct StartIndex;
impl StartIndex {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

sized_concat_type!(DefElse, 0xA1, TermList);

struct DefIfElse;
impl DefIfElse {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0xA0);
        let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[1..]);
        let pkg_end = pkg_size + 1;
        let elem1_len = Predicate::get_len(&data[1 + pkg_len..pkg_end]);
        let elem2_len = TermList::get_len(&data[1 + pkg_len + elem1_len..pkg_end]);
        assert_eq!(pkg_end, 1 + pkg_len + elem1_len + elem2_len);
        if pkg_end < data.len() {
            if data[pkg_end] == 0xA1 {
                let elem3_len = DefElse::get_len(&data[pkg_end..]);
                return 1 + pkg_len + elem1_len + elem2_len + elem3_len;
            }
        }
        1 + pkg_len + elem1_len + elem2_len
    }
}

sized_concat_type!(DefPresentElse, 0xA1, TermList);
sized_concat_type!(DefWhile, 0xA2, Predicate, TermList);

simple_concat_type!(DefReturn, 0xA4, ArgObject);
simple_concat_type!(DefRelease, 0x5B, 0x27, MutexObject);

struct DefBreak;
impl DefBreak {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0xA5);
        1
    }
}

struct DefBreakPoint;
impl DefBreakPoint {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0xCC);
        1
    }
}

struct DefContinue;
impl DefContinue {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x9F);
        1
    }
}

struct NotifyValue;
impl NotifyValue {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct FatalArg;
impl FatalArg {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

type NotifyObject = SuperName;
type FatalType = ByteData;
type FatalCode = DWordData;
simple_concat_type!(DefNotify, 0x86, NotifyObject, NotifyValue);
simple_concat_type!(DefFatal, 0x5B, 0x32, FatalType, FatalCode, FatalArg);

struct DefNoop;
impl DefNoop {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0xA3);
        1
    }
}

struct MsecTime;
impl MsecTime {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct UsecTime;
impl UsecTime {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

simple_concat_type!(DefReset, 0x5B, 0x26, EventObject);
simple_concat_type!(DefSignal, 0x5B, 0x24, EventObject);
simple_concat_type!(DefSleep, 0x5B, 0x22, MsecTime);
simple_concat_type!(DefStall, 0x5B, 0x21, UsecTime);

struct StatementOpcode;
impl StatementOpcode {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x5B => match data[1] {
                0x21 => DefStall::get_len(data),
                0x22 => DefSleep::get_len(data),
                0x24 => DefSignal::get_len(data),
                0x26 => DefReset::get_len(data),
                0x27 => DefRelease::get_len(data),
                0x32 => DefFatal::get_len(data),
                _ => panic!("Invalid statement {:#x}", data[1]),
            },
            0x86 => DefNotify::get_len(data),
            0x9F => DefContinue::get_len(data),
            0xA0 => DefIfElse::get_len(data),
            0xA2 => DefWhile::get_len(data),
            0xA3 => DefNoop::get_len(data),
            0xA4 => DefReturn::get_len(data),
            0xA5 => DefBreak::get_len(data),
            0xCC => DefBreakPoint::get_len(data),
            _ => panic!("Invalid statement {:#x}", data[0]),
        }
    }
}

struct Operand;
impl Operand {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct BuffPkgStrObj;
impl BuffPkgStrObj {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct IndexValue;
impl IndexValue {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct BufData;
impl BufData {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct ObjReference;
impl ObjReference {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct BufferSize;
impl BufferSize {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct Dividend;
impl Dividend {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct Divisor;
impl Divisor {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct BCDValue;
impl BCDValue {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct MidObj;
impl MidObj {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct SearchPkg;
impl SearchPkg {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct ShiftCount;
impl ShiftCount {
    fn get_len(data: &[u8]) -> usize {
        TermArg::get_len(data)
    }
}

struct ByteList;
impl ByteList {
    fn get_len(data: &[u8]) -> usize {
        data.len()
    }
}

struct LengthArg;
impl LengthArg {
    fn get_len(data: &[u8]) -> usize {
        data.len()
    }
}

struct LocalObj;
impl LocalObj {
    fn get_len(data: &[u8]) -> usize {
        assert!((0x60..=0x67).contains(&data[0]));
        1
    }
}

struct SimpleName;
impl SimpleName {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x60..=0x67 => LocalObj::get_len(data),
            0x68..=0x6E => ArgObj::get_len(data),
            get_prefix!(NameString) => NameString::get_len(data),
            _ => panic!("Invalid simple name"),
        }
    }
}

struct ReferenceTypeOpcode;
impl ReferenceTypeOpcode {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x88 => DefIndex::get_len(data),
            0x83 => DefDerefOf::get_len(data),
            0x71 => DefRefOf::get_len(data),
            _ => panic!("Invalid reference type opcode"),
        }
    }
}

struct DebugObj;
impl DebugObj {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x5B);
        assert_eq!(data[1], 0x31);
        2
    }
}

struct SuperName;
impl SuperName {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x60..=0x6E | get_prefix!(NameString) => SimpleName::get_len(data),
            0x71 | 0x83 | 0x88 => ReferenceTypeOpcode::get_len(data),
            0x5B => match data[1] {
                0x31 => DebugObj::get_len(data),
                _ => panic!("Invalid super name {:#x} {:#x}", data[0], data[1]),
            },
            _ => panic!("Invalid super name"),
        }
    }
}

struct Target;
impl Target {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            get_prefix!(SuperName) => SuperName::get_len(data),
            0x5B => match data[1] {
                get_extop_prefix!(SuperName) => SuperName::get_len(data),
                _ => panic!("Invalid target {:#x}", data[1]),
            },
            _ => panic!("Invalid target {:#x}", data[0]),
        }
    }
}

struct PackageElement;
impl PackageElement {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            get_prefix!(NameString) => NameString::get_len(data),
            get_prefix!(DataRefObject) => DataRefObject::get_len(data),
            _ => panic!("Invalid package element {:#x}", data[0]),
        }
    }
}

struct PackageElementList;
impl PackageElementList {
    fn get_len(data: &[u8]) -> usize {
        if data.len() == 0 {
            return 0;
        }
        let mut index = 0;
        print!("Parsing package element list\n");
        while index < data.len() {
            print!("Parsing package element at index {index}\n");
            index += match data[index] {
                get_prefix!(PackageElement) => PackageElement::get_len(&data[index..]),
                _ => panic!("Invalid package element {:#x}", data[index]),
            };
        }
        print!("Done parsing package element list\n");
        assert_eq!(index, data.len());
        index
    }
}

type Remainder = Target;
type Quotient = Target;
type EventObject = SuperName;

simple_concat_type!(DefAcquire, 0x5B, 0x23, MutexObject, Timeout);
simple_concat_type!(DefAdd, 0x72, Operand, Operand, Target);
simple_concat_type!(DefAnd, 0x7B, Operand, Operand, Target);
sized_concat_type!(DefBuffer, 0x11, BufferSize, ByteList);
simple_concat_type!(DefConcat, 0x73, Data, Data, Target);
simple_concat_type!(DefConcatRes, 0x84, BufData, BufData, Target);
simple_concat_type!(DefCondRefOf, 0x5B, 0x12, SuperName, Target);
simple_concat_type!(DefCopyObject, 0x9D, TermArg, SimpleName);
simple_concat_type!(DefDecrement, 0x76, SuperName);
simple_concat_type!(DefDerefOf, 0x83, ObjReference);
simple_concat_type!(DefDivide, 0x78, Dividend, Divisor, Remainder, Quotient);
simple_concat_type!(DefFindSetLeftBit, 0x81, Operand, Target);
simple_concat_type!(DefFindSetRightBit, 0x82, Operand, Target);
simple_concat_type!(DefFromBCD, 0x5B, 0x28, BCDValue, Target);
simple_concat_type!(DefIncrement, 0x75, SuperName);
simple_concat_type!(DefIndex, 0x88, BuffPkgStrObj, IndexValue, Target);
simple_concat_type!(DefLAnd, 0x90, Operand, Operand);
simple_concat_type!(DefLEqual, 0x93, Operand, Operand);
simple_concat_type!(DefLGreater, 0x94, Operand, Operand);
simple_concat_type!(DefLGreaterEqual, 0x92, 0x95, Operand, Operand);
simple_concat_type!(DefLLess, 0x95, Operand, Operand);
simple_concat_type!(DefLLessEqual, 0x92, 0x94, Operand, Operand);
simple_concat_type!(DefMid, 0x9E, MidObj, TermArg, TermArg, Target);
simple_concat_type!(DefLNot, 0x92, Operand);
simple_concat_type!(DefLNotEqual, 0x92, 0x93, Operand, Operand);
simple_concat_type!(
    DefLoadTable,
    0x5B,
    0x1F,
    TermArg,
    TermArg,
    TermArg,
    TermArg,
    TermArg,
    TermArg
);
simple_concat_type!(DefLOr, 0x91, Operand, Operand);
simple_concat_type!(
    DefMatch,
    0x89,
    SearchPkg,
    MatchOpcode,
    Operand,
    MatchOpcode,
    Operand,
    StartIndex
);
simple_concat_type!(DefMod, 0x85, Dividend, Divisor, Target);
simple_concat_type!(DefMultiply, 0x77, Operand, Operand, Target);
simple_concat_type!(DefNAnd, 0x7C, Operand, Operand, Target);
simple_concat_type!(DefNOr, 0x7E, Operand, Operand, Target);
simple_concat_type!(DefNot, 0x80, Operand, Target);
simple_concat_type!(DefOr, 0x7D, Operand, Operand, Target);
sized_concat_type!(DefPackage, 0x12, NumElements, PackageElementList);
sized_concat_type!(DefVarPackage, 013, VarNumElements, PackageElementList);
simple_concat_type!(DefRefOf, 0x71, SuperName);
simple_concat_type!(DefShiftLeft, 0x79, Operand, ShiftCount, Target);
simple_concat_type!(DefShiftRight, 0x7A, Operand, ShiftCount, Target);
simple_concat_type!(DefSizeOf, 0x87, SuperName);
simple_concat_type!(DefStore, 0x70, TermArg, SuperName);
simple_concat_type!(DefSubtract, 0x74, Operand, Operand, Target);
simple_concat_type!(DefToBCD, 0x5B, 0x29, Operand, Target);
simple_concat_type!(DefToBuffer, 0x96, Operand, Target);
simple_concat_type!(DefToDecimalString, 0x97, Operand, Target);
simple_concat_type!(DefToHexString, 0x98, Operand, Target);
simple_concat_type!(DefToInteger, 0x99, Operand, Target);
simple_concat_type!(DefToString, 0x9C, TermArg, LengthArg, Target);
simple_concat_type!(DefWait, 0x5B, 0x25, EventObject, Operand);
simple_concat_type!(DefXOr, 0x7F, Operand, Operand, Target);

struct DefTimer;
impl DefTimer {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x5B);
        assert_eq!(data[1], 0x33);
        2
    }
}

struct MethodInvocation;
impl MethodInvocation {
    fn get_len(data: &[u8]) -> usize {
        //print!("Parsing method invocation\n");
        let elem1_len = NameString::get_len(data);
        //print!("Searching for method\n");
        let arg_count = get_method_arg_count(&data[..elem1_len]);
        let elem2_len = TermArgList::get_len(&data[elem1_len..], arg_count);
        elem1_len + elem2_len
    }
}

struct DefObjectType;
impl DefObjectType {
    fn get_len(_: &[u8]) -> usize {
        todo!();
    }
}

struct ExpressionOpcode;
impl ExpressionOpcode {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x11 => DefBuffer::get_len(data),
            0x12 => DefPackage::get_len(data),
            0x70 => DefStore::get_len(data),
            0x71 => DefRefOf::get_len(data),
            0x72 => DefAdd::get_len(data),
            0x73 => DefConcat::get_len(data),
            0x74 => DefSubtract::get_len(data),
            0x75 => DefIncrement::get_len(data),
            0x76 => DefDecrement::get_len(data),
            0x77 => DefMultiply::get_len(data),
            0x78 => DefDivide::get_len(data),
            0x79 => DefShiftLeft::get_len(data),
            0x7A => DefShiftRight::get_len(data),
            0x7B => DefAnd::get_len(data),
            0x7C => DefNAnd::get_len(data),
            0x7D => DefOr::get_len(data),
            0x7E => DefNOr::get_len(data),
            0x7F => DefXOr::get_len(data),
            0x80 => DefNot::get_len(data),
            0x81 => DefFindSetLeftBit::get_len(data),
            0x82 => DefFindSetRightBit::get_len(data),
            0x83 => DefDerefOf::get_len(data),
            0x84 => DefConcatRes::get_len(data),
            0x85 => DefMod::get_len(data),
            0x87 => DefSizeOf::get_len(data),
            0x88 => DefIndex::get_len(data),
            0x89 => DefMatch::get_len(data),
            0x8E => DefObjectType::get_len(data),
            0x90 => DefLAnd::get_len(data),
            0x91 => DefLOr::get_len(data),
            0x92 => match data[1] {
                0x93 => DefLNotEqual::get_len(data),
                0x94 => DefLLessEqual::get_len(data),
                0x95 => DefLGreaterEqual::get_len(data),
                _ => DefLNot::get_len(data),
            },
            0x93 => DefLEqual::get_len(data),
            0x94 => DefLGreater::get_len(data),
            0x95 => DefLLess::get_len(data),
            0x96 => DefToBuffer::get_len(data),
            0x97 => DefToDecimalString::get_len(data),
            0x98 => DefToHexString::get_len(data),
            0x99 => DefToInteger::get_len(data),
            0x9C => DefToString::get_len(data),
            0x9D => DefCopyObject::get_len(data),
            0x9E => DefMid::get_len(data),
            0x5B => match data[1] {
                0x12 => DefCondRefOf::get_len(data),
                0x1F => DefLoadTable::get_len(data),
                0x23 => DefAcquire::get_len(data),
                0x25 => DefWait::get_len(data),
                0x28 => DefFromBCD::get_len(data),
                0x29 => DefToBCD::get_len(data),
                0x33 => DefTimer::get_len(data),
                _ => panic!("Invalid expression {:#x}", data[1]),
            },
            get_prefix!(NameString) => MethodInvocation::get_len(data),
            _ => panic!("Invalid expression {:#x}", data[0]),
        }
    }
}

struct NameSeg;
impl NameSeg {
    fn get_len(data: &[u8]) -> usize {
        assert!((0x41..=0x5A).contains(&data[0]) || data[0] == 0x5F);
        for i in 1..=3 {
            assert!(
                (0x41..=0x5A).contains(&data[i])
                    || data[i] == 0x5F
                    || (0x30..=0x39).contains(&data[i])
            );
        }
        4
    }

    fn is_valid(data: &[u8]) -> bool {
        if !(0x41..=0x5A).contains(&data[0]) && data[0] != 0x5F {
            return false;
        }
        for i in 1..=3 {
            if !(0x41..=0x5A).contains(&data[i])
                && data[i] != 0x5F
                && !(0x30..=0x39).contains(&data[i])
            {
                return false;
            }
        }
        true
    }
}

struct DualNamePath;
impl DualNamePath {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x2E);
        let elem1_len = NameSeg::get_len(&data[1..]);
        let elem2_len = NameSeg::get_len(&data[1 + elem1_len..]);
        1 + elem1_len + elem2_len
    }
    fn is_valid(data: &[u8]) -> bool {
        if data[0] != 0x2E {
            return false;
        }
        if !NameSeg::is_valid(&data[1..]) {
            return false;
        }
        let elem1_len = NameSeg::get_len(&data[1..]);
        if !NameSeg::is_valid(&data[1 + elem1_len..]) {
            return false;
        }
        true
    }
}

struct MultiNamePath;
impl MultiNamePath {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x2F);
        let seg_count = data[1];
        assert!(seg_count > 0);
        let mut offset = 0;
        for _ in 0..seg_count {
            offset += NameSeg::get_len(&data[2 + offset..]);
        }
        2 + offset
    }
    fn is_valid(data: &[u8]) -> bool {
        if data[0] != 0x2F {
            return false;
        }
        let seg_count = data[1];
        if seg_count == 0 {
            return false;
        }
        let mut offset = 0;
        for _ in 0..seg_count {
            if !NameSeg::is_valid(&data[2 + offset..]) {
                return false;
            }
        }
        true
    }
}

struct NamePath;
impl NamePath {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x41..=0x5A | 0x5F => NameSeg::get_len(data),
            0x2E => DualNamePath::get_len(data),
            0x2F => MultiNamePath::get_len(data),
            0 => 1,
            _ => panic!("Invalid name path"),
        }
    }
    fn is_valid(data: &[u8]) -> bool {
        match data[0] {
            0x41..=0x5A | 0x5F => NameSeg::is_valid(data),
            0 => true,
            0x2E => DualNamePath::is_valid(data),
            0x2F => MultiNamePath::is_valid(data),
            _ => false,
        }
    }
}

struct NameString;
impl NameString {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x5C => 1 + NamePath::get_len(&data[1..]),
            0x5E => {
                let mut prefix_count = 1;
                while data[prefix_count] == 0x5E {
                    prefix_count += 1;
                }
                prefix_count + NamePath::get_len(&data[prefix_count..])
            }
            0x41..=0x5A | 0x5F | 0x2E | 0x2F | 0 => NamePath::get_len(data),
            _ => panic!("Invalid name string"),
        }
    }
    fn is_valid(data: &[u8]) -> bool {
        match data[0] {
            0x5C => NamePath::is_valid(&data[1..]),
            0x5E => {
                let mut prefix_count = 1;
                while data[prefix_count] == 0x5E {
                    prefix_count += 1;
                }
                NamePath::is_valid(&data[prefix_count..])
            }
            0x41..=0x5A | 0x5F | 0x2E | 0x2F | 0 => NamePath::is_valid(data),
            _ => false,
        }
    }
    fn print(data: &[u8]) {
        for i in 0..Self::get_len(data) {
            print!("{}", char::from_u32(data[i] as u32).unwrap());
        }
    }
}

struct DefAlias;
impl DefAlias {
    fn get_len(data: &[u8]) -> usize {
        assert_eq!(data[0], 0x06);
        let elem1_len = NameString::get_len(&data[1..]);
        let elem2_len = NameString::get_len(&data[1 + elem1_len..]);
        1 + elem1_len + elem2_len
    }
}

struct NameSpaceModifierObj;
impl NameSpaceModifierObj {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x06 => DefAlias::get_len(data),
            0x08 => DefName::get_len(data),
            0x10 => DefScope::get_len(data),
            _ => panic!("Invalid namespace modifier object"),
        }
    }
}

struct Object;
impl Object {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x06 | 0x08 | 0x10 => NameSpaceModifierObj::get_len(data),
            0x5B => match data[1] {
                0x87 | 0x13 | 0x88 | 0x80 | 0x84 | 0x85 => NamedObj::get_len(data),
                _ => panic!("Invalid object"),
            },
            0x8A..=0x8D | 0x8F | 0x14 | 0x15 => NamedObj::get_len(data),
            _ => panic!("Invalid object"),
        }
    }
}

struct TermObject;
impl TermObject {
    fn get_len(data: &[u8]) -> usize {
        match data[0] {
            0x06 | 0x08 | 0x10 | 0x8A..=0x8D | 0x8F | 0x14 | 0x15 => Object::get_len(data),
            0xCC | 0x9F | 0xA0 | 0x86 | 0xA2..=0xA5 => StatementOpcode::get_len(data),
            get_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
            // ExtOp
            0x5B => match data[1] {
                // 0x83 DEPRECATED PROCESSOROP
                0x87 | 0x13 | 0x88 | 0x80 | 0x84 | 0x85 | 0x81 | 0x82 | 0x83 | 0x01 => {
                    NamedObj::get_len(data)
                }
                0x32 | 0x27 | 0x26 | 0x24 | 0x22 | 0x21 => StatementOpcode::get_len(data),
                get_extop_prefix!(ExpressionOpcode) => ExpressionOpcode::get_len(data),
                _ => panic!("Invalid term object {:#x} {:#x}", data[0], data[1]),
            },
            _ => panic!("Invalid term object {:#x}", data[0]),
        }
    }
}

struct TermArgList;
impl TermArgList {
    fn get_len(data: &[u8], arg_count: usize) -> usize {
        let mut index = 0;
		print!("Parsing term arg list with {arg_count} arguments\n");
        for arg in 0..arg_count {
            index += TermArg::get_len(&data[index..]);
        }
        index
    }
}

struct TermList;
impl TermList {
    fn get_len(data: &[u8]) -> usize {
        let mut index = 0;
        print!("Parsing term list\n");
        while index < data.len() {
            print!("Index: {index}\n");
            index += TermObject::get_len(&data[index..]);
        }
        print!("Done parsing term list\n");
        assert_eq!(index, data.len());
        index
    }
}

fn get_method_arg_count(name_string: &[u8]) -> usize {

	//for a in name_string.iter() {
	//	print!("{:#x}", a);
	//}
	//print!("\n");

	for method in 0..unsafe {GLOBAL_METHOD_COUNT} {
		let data : &[u8] = unsafe{core::slice::from_raw_parts(GLOBAL_METHODS[method].0, GLOBAL_METHODS[method].1)};

		//for a in data.iter() {
		//	print!("{:#x}", a);
		//}
		//print!("\n");

		assert_eq!(data[0], 0x14);
		let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[1..]);
		let pkg_end = pkg_size + 1;
		let name_len = NameString::get_len(&data[1+pkg_len..pkg_end]);
		if name_len == name_string.len() {
			if data[1+pkg_len..1+pkg_len+name_len] == *name_string {
				print!("Found method\n");
				let method_flags = data[1+pkg_len+name_len] as usize;
				return method_flags & 0b111;
			}
		}
	}
	// method not found
	print!("Method not found\n");
	//todo!();
	0
}

pub fn parse_aml(data: &[u8]) {
    print!("Parsing {} bytes of aml term list\n", data.len());

	print!("Skipping parse, looking for devices\n");

	for x in 0..data.len() - 1 {
		if data[x] == 0x5B {
			if data[x+1] == 0x82 {
				let (pkg_len, pkg_size) = PkgLength::get_len_and_size(&data[x+2..]);
				NameString::print(&data[x+2+pkg_len..]);
				print!(" ");
			}
		}
	}

	
/*
    let len = TermList::get_len(data);
    assert_eq!(len, data.len());
    print!("Successfully parsed {} bytes of AML\n", data.len());
*/
}

/*
Errors found in the ACPI ml spec

missing NamedObject fields:
    DefMethod
    DefField
    DefDevice
    DefMutex

A lot of hyperlink errors
Backslash in root character messes up documentation

Package length definition missing

size of ObjectReference missing

DataRefObject parsing impossible ?

The MethodInvocation Expression is messed up and makes parsing 100 times harder (no arg length)

you can use DefStore to read from a NamedField???
does this mean a NamedField name is a TermArg???

-> NameString is missing in TermArg

does a NamedField name count as a methodinvocation?

Target : SuperName | NullName -> NullName redundant

AccessLength is not defined

*/
