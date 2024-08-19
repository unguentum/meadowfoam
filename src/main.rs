#![no_std]
#![no_main]

use core::panic::PanicInfo;

use core::arch::global_asm;
use core::arch::asm;

use core::mem::size_of;
use core::ffi::c_void;

use core::slice;
use core::ptr;

mod aml;

global_asm!(r#"
.global _entry
_entry:
 cli
 mov rdi, rax
 mov rsi, rbx
 jmp kernel_entry
_stop:
 jmp _stop
"#);

#[repr(C, packed)]
struct EfiTableHeader {
	signature : u64,
	revision : u32,
	header_size : u32,
	crc : u32,
	reserved : u32,
}

#[repr(C, packed)]
struct EfiBootServices {
	header : EfiTableHeader,

	raise_tpl : *const c_void,
	restore_tpl : *const c_void,

	allocate_pages : *const c_void,
	free_pages : *const c_void,
	get_memory_map : *const c_void,
	allocate_pool : *const c_void,
	free_pool : *const c_void,

	create_event : *const c_void,
	set_timer : *const c_void,
	wait_for_event : *const c_void,
	signal_event : *const c_void,
	close_event : *const c_void,
	check_event : *const c_void,

	install_protocol_interface : *const c_void,
	reinstall_protocol_interface : *const c_void,
	uninstall_protocol_interface : *const c_void,
	handle_protocol : *const c_void,
	reserved_0 : *const c_void,
	register_protocol_notify : *const c_void,
	locate_handle : *const c_void,
	locate_device_path : *const c_void,
	install_configuration_table : *const c_void,

	load_image : *const c_void,
	start_image : *const c_void,
	exit : *const c_void,
	unload_image : *const c_void,
	exit_boot_services : *const c_void,

	get_next_monotonic_count : *const c_void,
	stall : *const c_void,
	set_watchdog_timer : *const c_void,

	connect_controller : *const c_void,
	disconnect_controller : *const c_void,

	open_protocol : *const c_void,
	close_protocol : *const c_void,
	open_protocol_information : *const c_void,

	protocols_per_handle : *const c_void,
	locate_handle_buffer : *const c_void,
	locate_protocol : *const c_void,
}

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

#[repr(C, packed)]
struct EfiSystemTable {
	header : EfiTableHeader,
	vendor : *const u16,
	revision : u32,
	console_in_handle : *const c_void,
	text_in_protocol : *const c_void,
	console_out_handle : *const c_void,
	text_out_protocol : *const c_void,
	console_stderr_handle : *const c_void,
	text_stderr_protocol : *const c_void,
	runtime_services : *const c_void,
	boot_services : *const EfiBootServices,	
}

#[repr(C, packed)]
struct Multiboot2Header {
	magic : u32,
	architecture : u32,
	header_length : u32,
	checksum : u32,
	efi_tag_type : u16,
	efi_tag_flags : u16,
	efi_tag_size : u32,
	stop_tag_type : u16,
	stop_tag_flags : u16,
	stop_tag_size : u32,
}

const HEADER_SIZE : u32 = size_of::<Multiboot2Header>() as u32;
const MAGIC : u32 = 0xE85250D6;

#[used]
#[link_section = ".multiboot_header"]
static MULTIBOOT2_HEADER : Multiboot2Header = Multiboot2Header {
	magic : MAGIC,
	architecture : 0,
	header_length : HEADER_SIZE,
	checksum : -(MAGIC as i32 + 0 + HEADER_SIZE as i32) as u32,
	efi_tag_type : 7,
	efi_tag_flags : 0,
	efi_tag_size : 8,
	stop_tag_type : 0,
	stop_tag_flags : 0,
	stop_tag_size : 8,
};


const COM1 : u16 = 0x3F8;

fn outb(port : u16, value : u8){
	unsafe { asm!("out dx, al", in("dx") port, in("al") value); }
}

fn inb(port : u16) -> u8 {
	let mut ret : u8;
	unsafe { asm!("in al, dx", out("al") ret, in("dx") port); }
	ret
}

fn com0_write_byte(c : u8){
	loop {
		let status = inb(COM1 + 5);
		if status & (1 << 5) != 0 {
			break;
		}
	}
	outb(COM1, c);
}

fn com0_write(s : &[u8]){
	for c in s {
		com0_write_byte(*c);
	}
}

fn com0_init(){
	let mut register_value = inb(COM1 + 3);
	register_value |= 1 << 7;
	outb(COM1 + 3, register_value);

	outb(COM1 + 0, 12);
	outb(COM1 + 1, 0);

	outb(COM1 + 2, 0b111);

	let mut register_value = inb(COM1 + 3);
	register_value &= !(1 << 7);
	outb(COM1 + 3, register_value);	
}

#[no_mangle]
pub fn kernel_entry(magic : u32, multiboot_info : u64) -> ! {

	com0_init();
	com0_write(b"Hello world");

	let total_size : u32 = unsafe { core::ptr::read_volatile(multiboot_info as *const u32) };
	let mut offset : isize = 8;
	let base_pointer = multiboot_info as *const u8;
	while (offset as u32) < total_size {
		let next_type = unsafe { *(base_pointer.offset(offset) as *const u32) };
		let next_size = unsafe { *(base_pointer.offset(offset+4) as *const u32) };

		//if next_type == 12 { // efi system table
			//let system_table_pointer = *(base_pointer.offset(offset + 8) as *const u64);
			//let system_table = &*(system_table_pointer as *const EfiSystemTable);
		//}

		if next_type == 15 { // ACPI new RSDP -> copy of RSDPv2 (XSDP)
			let xsdp_pointer = unsafe { base_pointer.offset(offset + 8) as *const XSDP };
			let xsdp = unsafe { &*xsdp_pointer };
			let xsdt = unsafe { &*xsdp.xsdt };
			let entry_amount = xsdt.entry_amount();
			for entry_index in 0..entry_amount {
				let entry = xsdt.get_entry(entry_index);
				if let Some(entry) = entry {
					if unsafe { (*entry).header.signature } == *b"FACP" {
						let fadt = entry as *const FADT;
						let dsdt : &Sdt = unsafe {&*(*fadt).x_dsdt};
						let dsdt_body = dsdt.get_body();

						aml::aml_parse_bytes(dsdt_body);

						loop {}
					}
				}
			}
		}

		offset += next_size as isize;
		if offset % 8 != 0 {
			offset += 8 - (offset % 8);
		}
	}
	loop {}
}


#[panic_handler]
fn panic_handler(panic_info : &PanicInfo) -> ! {
	loop {}
}
