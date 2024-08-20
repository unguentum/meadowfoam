#![no_std]
#![no_main]

use core::panic::PanicInfo;

use core::arch::global_asm;
use core::arch::asm;

use core::mem::size_of;

use core::slice;
use core::ptr;

use core::fmt::Write;

mod aml;
mod uefi;
mod graphics;
mod triple_fault;

macro_rules! print {
	($writer_name:expr, $($arg:tt)*) => {
		$writer_name.write_fmt(format_args!($($arg)*)).unwrap();
	};
}

global_asm!(r#"
.global _entry
_entry:
 mov rdi, rax
 mov rsi, rbx
 jmp kernel_entry
_stop:
 hlt
 jmp _stop
"#);

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

	assert!(magic == 0x36d76289);

	com0_init();
	com0_write(b"Hello world");

	let total_size : u32 = unsafe { core::ptr::read_volatile(multiboot_info as *const u32) };
	let mut offset : isize = 8;
	let base_pointer = multiboot_info as *const u8;
	while (offset as u32) < total_size {
		let next_type = unsafe { *(base_pointer.offset(offset) as *const u32) };
		let next_size = unsafe { *(base_pointer.offset(offset+4) as *const u32) };

		if next_type == 12 { // efi system table
			let system_table_pointer = unsafe { *(base_pointer.offset(offset + 8) as *const u64) };
			let system_table = unsafe { &*(system_table_pointer as *const uefi::SystemTable) };
			if let Some(interface) = uefi::locate_gop(system_table) {
				let interface = unsafe { &mut *interface };
				if let None = unsafe { interface.current_mode() } {
					interface.set_mode(0);
				}
				let current_mode = unsafe { interface.current_mode().unwrap() };
				interface.set_mode(current_mode);
				let frame_buffer = unsafe { interface.get_framebuffer().unwrap()};
				let pixels_per_line = unsafe { interface.get_pixels_per_line().unwrap() };
				let mut writer = graphics::ScreenWriter::new(frame_buffer, pixels_per_line as usize);
				print!(writer, "TextIO using UEFI GOP\n");
				print!(writer, "Graphics protocol: {:#?}\n", &interface);
				print!(writer, "Graphics mode : {:#?}\n", unsafe {&*interface.mode } );

				for mode in 0.. unsafe{interface.mode.as_ref().unwrap().max_mode}{
					let info = interface.query_mode(mode);
					print!(writer, "{:#?}", unsafe { info.unwrap().as_ref() });
				}
			}
		}

		if next_type == 15 { // ACPI new RSDP -> copy of RSDPv2 (XSDP)
			let xsdp_pointer = unsafe { base_pointer.offset(offset + 8) as *const XSDP };
			let xsdp = unsafe { &*xsdp_pointer };
			let xsdt = unsafe { &*xsdp.xsdt };
			let entry_amount = xsdt.entry_amount();
			for entry_index in 0..entry_amount {
				let entry = xsdt.get_entry(entry_index);
				if let Some(entry) = entry {
					let signature = unsafe { (*entry).header.signature };
					if signature == *b"FACP" {
						let fadt = entry as *const FADT;
						let dsdt : &Sdt = unsafe {&*(*fadt).x_dsdt};
						let dsdt_body = dsdt.get_body();
						aml::aml_parse_bytes(dsdt_body);
					}
				}
			}
		}

		offset += next_size as isize;
		if offset % 8 != 0 {
			offset += 8 - (offset % 8);
		}
	}
	loop {
		unsafe {
			asm!("hlt");
		}
	}
}


#[panic_handler]
fn panic_handler(panic_info : &PanicInfo) -> ! {
	loop {}
}
