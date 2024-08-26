use core::ptr;

#[derive(Debug)]
#[repr(C)]
pub struct SDTHeader {
	pub signature : [u8; 4],
	length : u32,
	revision : u8,
	checksum : u8,
	oemid : [u8; 6],
	oem_table_id : [u8; 8],
	oem_revision : u32,
	creator_id : u32,
	creator_revision : u32,
}

#[repr(C)]
pub struct XSDT {
	header : SDTHeader,
}

#[derive(Debug)]
#[repr(C)]
pub struct DSDT {
	header : SDTHeader,
}
impl SDT for DSDT {
	fn get_header(&self) -> &SDTHeader {
		&self.header
	}
}

pub trait SDT {
	fn get_header(&self) -> &SDTHeader;
	fn get_body_pointer(&self) -> *const u8 {
		unsafe { (self as *const Self as *const u8).add(size_of::<SDTHeader>()) }
	}
	fn get_body(&self) -> &[u8] {
		unsafe { core::slice::from_raw_parts(self.get_body_pointer(), self.get_header().length as usize - size_of::<SDTHeader>()) }
	}
}

impl XSDT {
	const FADT_SIGNATURE : &[u8; 4] = b"FACP";
	fn find_table(&self, signature : &[u8; 4]) -> Option<*const SDTHeader> {
		unsafe {
			self.get_tables().find(|table| table.as_ref().unwrap().signature == *signature)
		}
	}
	pub fn find_fadt(&self) -> Option<*const FADT> {
		self.find_table(Self::FADT_SIGNATURE).map(|ptr| ptr as *const FADT)
	}
	pub fn get_tables(&self) -> impl Iterator<Item = *const SDTHeader> + '_{
		(0..self.table_amount()).map(|index| self.get_table(index).expect("Elements in range are present"))
	}
	fn table_amount(&self) -> usize {
		let entry_length = self.header.length as usize - size_of::<SDTHeader>();
		entry_length / size_of::<*const SDTHeader>()
	}
	fn get_table(&self, index : usize) -> Option<*const SDTHeader> {
		if index >= self.table_amount() {
			return None;
		}
		let byte_pointer = self as *const Self as *const u8;
		let byte_pointer = unsafe { byte_pointer.add(size_of::<SDTHeader>() + index * size_of::<u64>()) };
		let data = unsafe {ptr::read_unaligned(byte_pointer as *const *const SDTHeader)};
		Some(data)
	}
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct XSDP {
	signature : [u8; 8],
	checksum : u8,
	oemid : [u8; 6],
	revision : u8,
	rsdt_address : u32,
	length : u32,
	pub xsdt : *const XSDT,
	extended_checksum : u8,
	reserved : [u8; 3],
}

impl XSDP {

	pub fn from_raw_pointer(ptr : *const XSDP) -> &'static XSDP {
		let reference = unsafe { ptr.as_ref().unwrap() };
		reference.verify().unwrap();
		reference
	}

	fn verify(&self) -> Result<(), &'static str> {
		// TODO checksum
		if self.signature != *b"RSD PTR " {
			return Err("Wrong signature");
		}
		return Ok(());
	}
}

#[repr(C, packed)]
pub struct FADT {
	header : SDTHeader,
	firmware_ctrl : u32,
	dsdt : u32,
	reserved : u8,

	preferred_power_management_profile : u8,
	sci_interrupt : u16,
	smi_command_port : u32,

	acpi_enable : u8,
	acpi_disable : u8,
	s4bios_req : u8,
	pstate_control : u8,

	e1 : u32,
	e2 : u32,
	e3 : u32,
	e4 : u32,
	e5 : u32,
	e6 : u32,
	e7 : u32,
	e8 : u32,

	b1 : u8,
	b2 : u8,
	b3 : u8,
	b4 : u8,
	b5 : u8,
	b6 : u8,
	b7 : u8,
	b8 : u8,

	c1 : u16,
	c2 : u16,
	c3 : u16,
	c4 : u16,

	d1 : u8,
	d2 : u8,
	d3 : u8,
	d4 : u8,
	d5 : u8,

	boot_architecture_flags : u16,
	reserved2 : u8,
	flags : u32,

	reset_reg : GenericAddressStructure,
	reset_value : u8,
	reserved3 : [u8; 3],
	x_firmware_control : u64,
	pub x_dsdt : *const DSDT,

}

#[derive(Debug)]
#[repr(C, packed)]
struct GenericAddressStructure {
	address_space : u8,
	bit_width : u8,
	bit_offset : u8,
	access_size : u8,
	address : u64,
}
