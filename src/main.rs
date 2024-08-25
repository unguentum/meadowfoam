#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use core::fmt::Write;

//mod aml;
mod acpi;
//mod port_io

mod uefi;
mod graphics;
mod triple_fault;

macro_rules! print {
	($writer_name:expr, $($arg:tt)*) => {
		$writer_name.write_fmt(format_args!($($arg)*)).unwrap()
	};
}

#[no_mangle]
pub fn efi_main(handle : uefi::Handle, system_table : *const uefi::SystemTable) -> ! {

	let system_table = unsafe { system_table.as_ref().unwrap() };

	if let Some(interface) = uefi::locate_gop(system_table) {
		let interface = unsafe { &mut *interface };
		if let None = unsafe { interface.current_mode() } {
			interface.set_mode(0);
		}
		interface.set_mode(0);
		let current_mode = unsafe { interface.current_mode().unwrap() };
		let frame_buffer = unsafe { interface.get_framebuffer().unwrap()};
		let pixels_per_line = unsafe { interface.get_pixels_per_line().unwrap() };
		let mut writer = graphics::ScreenWriter::new(frame_buffer, pixels_per_line as usize);
		print!(writer, "Mode set to {}\n", current_mode);
		print!(writer, "TextIO using UEFI GOP\n");
		print!(writer, "Graphics protocol: {:#?}\n", &interface);
		print!(writer, "Graphics mode : {:#?}\n", unsafe {&*interface.mode } );
		print!(writer, "Available graphics modes: {}\n", unsafe {interface.mode.as_ref().unwrap().max_mode});

		if let Some(config_table) = system_table.find_configuration_table(uefi::ACPI_TABLE_GUID) {
			let xsdp = acpi::XSDP::from_raw_pointer(config_table.data as *const acpi::XSDP);
			print!(writer, "XSDP: {:#x?}\n", xsdp);
			let xsdt = unsafe { &*xsdp.xsdt };
			print!(writer, "Tables:\n");
			for entry in xsdt.get_tables() {
				let entry = unsafe{&*entry};
				print!(writer, "{}\n", core::str::from_utf8(&entry.signature).unwrap());
			}
			let fadt = unsafe { xsdt.find_fadt().unwrap().as_ref().unwrap() };
			//print!(writer, "FADT: {:?}\n", fadt);
			let x_dsdt = fadt.x_dsdt;
			print!(writer, "{:?}\n", unsafe { &*x_dsdt });
		} else {
			print!(writer, "No ACPI table found\n");
		}

		print!(writer, "Firmware vendor:");
		system_table.get_firmware_vendor().iter().for_each(|c| print!(writer, "{}", char::from_u32(*c as u32).unwrap_or('?')));
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
	loop {}
}
