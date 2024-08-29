#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use core::fmt::Write;

mod aml;
mod acpi;
//mod port_io

mod uefi;
mod graphics;
mod triple_fault;

use crate::acpi::SDT;
use crate::graphics::SCREEN_WRITER;

#[no_mangle]
pub fn efi_main(_handle : uefi::Handle, system_table : *const uefi::SystemTable) -> ! {

	let system_table = unsafe { system_table.as_ref().unwrap() };

	if let Some(interface) = uefi::locate_gop(system_table) {
		let interface = unsafe { &mut *interface };
		if let None = unsafe { interface.current_mode() } {
			interface.set_mode(0);
		}
		interface.set_mode(0);
		let current_mode = unsafe { interface.current_mode().unwrap() };
		let frame_buffer = unsafe { interface.get_framebuffer().unwrap()};
		let pixel_width = unsafe { interface.get_pixel_width().unwrap() };
		let pixel_height = unsafe { interface.get_pixel_height().unwrap() };
		unsafe { SCREEN_WRITER.init(frame_buffer, pixel_width, pixel_height); }
		print!("Mode set to {}\n", current_mode);
		print!("TextIO using UEFI GOP\n");
		print!("Graphics protocol: {:#?}\n", &interface);
		print!("Graphics mode : {:#?}\n", &*interface.mode);
		print!("Available graphics modes: {}\n", interface.mode.as_ref().unwrap().max_mode);

		if let Some(config_table) = system_table.find_configuration_table(uefi::ACPI_TABLE_GUID) {
			let xsdp = acpi::XSDP::from_raw_pointer(config_table.data as *const acpi::XSDP);
			print!("XSDP: {:#x?}\n", xsdp);
			let xsdt = unsafe { &*xsdp.xsdt };
			print!("Tables:\n");
			for entry in xsdt.get_tables() {
				let entry = unsafe{&*entry};
				print!("{}\n", core::str::from_utf8(&entry.signature).unwrap());
			}
			let fadt = unsafe { xsdt.find_fadt().unwrap().as_ref().unwrap() };
			let x_dsdt = unsafe { &*fadt.x_dsdt };
			print!("{:?}\n", x_dsdt);
			print!("AML bytes: {}\n", x_dsdt.get_body().len());
			aml::parse_aml(x_dsdt.get_body());
		} else {
			print!("No ACPI table found\n");
		}

		print!("Firmware vendor:");
		system_table.get_firmware_vendor().iter().for_each(|c| print!("{}", char::from_u32(*c as u32).unwrap_or('?')));
	}

	loop {
		unsafe {
			asm!("cli");
			asm!("hlt");
		}
	}
}


#[panic_handler]
fn panic_handler(panic_info : &PanicInfo) -> ! {
	print!("!PANIC: {:?} at {:?}", panic_info.message(), panic_info.location());
	loop {}
}
