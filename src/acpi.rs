#[repr(C, packed)]
struct AcpiSdtHeader {
	signature : [u8; 4],
	length : u32,
	revision : u8,
	checksum : u8,
	oemid : [u8; 6],
	oem_table_id : [u8; 8],
	oem_revision : u32,
	creator_id : u32,
	creator_revision : u32,
}

#[repr(C, packed)]
struct Sdt {
	header : AcpiSdtHeader,
}

impl Sdt {
	fn get_body(&self) -> &[u8] {
		let byte_pointer = self as *const Self as *const u8;
		let byte_pointer = unsafe { byte_pointer.add(size_of::<AcpiSdtHeader>()) };
		unsafe { slice::from_raw_parts(byte_pointer, self.header.length as usize - size_of::<AcpiSdtHeader>()) }
	}
}

#[repr(C, packed)]
struct XSDT {
	header : AcpiSdtHeader,
}

impl XSDT {
	fn entry_amount(&self) -> usize {
		let entry_length = self.header.length as usize - size_of::<AcpiSdtHeader>();
		entry_length / size_of::<*const AcpiSdtHeader>()
	}
	fn get_entry(&self, index : usize) -> Option<*const Sdt> {
		if index >= self.entry_amount() {
			return None;
		}
		let byte_pointer = self as *const Self as *const u8;
		let byte_pointer = unsafe { byte_pointer.add(size_of::<AcpiSdtHeader>() + index * size_of::<*const Sdt>()) };
		let data = unsafe { ptr::read_unaligned(byte_pointer as *const *const Sdt) };
		Some(data)
	}
}

#[repr(C, packed)]
struct XSDP {
	signature : [u8; 8],
	checksum : u8,
	oemid : [u8; 6],
	revision : u8,
	rsdt_address : u32,
	length : u32,
	xsdt : *const XSDT,
	extended_checksum : u8,
	reserved : [u8; 3],
}

#[repr(C, packed)]
struct FADT {
	header : AcpiSdtHeader,
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
	x_dsdt : *const Sdt,

}

#[repr(C, packed)]
struct GenericAddressStructure {
	address_space : u8,
	bit_width : u8,
	bit_offset : u8,
	access_size : u8,
	address : u64,
}
