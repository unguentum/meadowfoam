#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use core::fmt::Write;

//mod aml;
//mod acpi
//mod port_io

mod uefi;
mod graphics;
mod triple_fault;

macro_rules! print {
	($writer_name:expr, $($arg:tt)*) => {
		$writer_name.write_fmt(format_args!($($arg)*)).unwrap();
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
		let current_mode = unsafe { interface.current_mode().unwrap() };
		interface.set_mode(current_mode);
		let frame_buffer = unsafe { interface.get_framebuffer().unwrap()};
		let pixels_per_line = unsafe { interface.get_pixels_per_line().unwrap() };
		let mut writer = graphics::ScreenWriter::new(frame_buffer, pixels_per_line as usize);
		print!(writer, "Mode set to {}\n", current_mode);
		print!(writer, "TextIO using UEFI GOP\n");
		print!(writer, "Graphics protocol: {:#?}\n", &interface);
		print!(writer, "Graphics mode : {:#?}\n", unsafe {&*interface.mode } );
		print!(writer, "Available graphics modes:");
		for mode in 0.. unsafe{interface.mode.as_ref().unwrap().max_mode}{
			let info = interface.query_mode(mode);
			print!(writer, "{:#?}", unsafe { info.unwrap().as_ref() });
		}
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
