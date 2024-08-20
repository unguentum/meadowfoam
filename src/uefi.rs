use core::ffi::c_void;
use crate::triple_fault::triple_fault;
use core::arch::asm;

type LocateProtocol = extern "efiapi" fn(protocol : *const Guid, registration : *const c_void, interface : *mut *const c_void) -> StatusCode;

type PhysicalAddress = u64;

type GOPQueryMode = extern "efiapi" fn(this : *const GraphicsOutputProtocol, mode_number : u32, size_of_info : *mut UIntN, info:  *mut *const GOPModeInformation) -> StatusCode;
type GOPSetMode = extern "efiapi" fn(this : *mut GraphicsOutputProtocol, mode_number : u32) -> StatusCode;
type GOPBlt = extern "efiapi" fn(this : *mut GraphicsOutputProtocol, blt_buffer : *const GOPBltPixel, blt_operation : GOPBltOperation, source_x : UIntN, source_y : UIntN, destination_x : UIntN, destination_y : UIntN, width : UIntN, height : UIntN, delta : UIntN) -> StatusCode;

// unsigned value of native width
type UIntN = usize;

pub const GRAPHICS_OUTPUT_PROTOCOL_GUID : Guid = Guid { data1 : 0x9042a9de, data2 : 0x23dc, data3 : 0x4a38, data4 : [0x96,0xfb,0x7a,0xde,0xd0,0x80,0x51,0x6a]};

#[repr(C)]
enum GOPBltOperation {
	BltVideoFill,
	BltVideoToBltBuffer,
	BltBufferToVideo,
	BltVideoToVideo,
	GOBltOperationMax,
}

#[derive(Debug)]
#[repr(C)]
enum GraphicsPixelFormat {
	PixelRedGreenBlueReserved8BitPerColor,
	PixelBlueGreenRedReserved8BitPerColor,
	PixelBitMask,
	PixelBltOnly,
	PixelFormatMax,
}

#[derive(Debug)]
#[repr(C)]
struct PixelBitmask {
	red_mask : u32,
	green_mask : u32,
	blue_mask : u32,
	reserved_mask : u32,
}

#[repr(C)]
struct GOPBltPixel {
	blue : u8,
	green : u8,
	red : u8,
	reserved : u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct GOPModeInformation {
	version : u32,
	horizontal_resolution : u32,
	vertical_resolution : u32,
	pixel_format : GraphicsPixelFormat,
	pixel_information : PixelBitmask,
	pixels_per_scanline : u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct GOPMode {
	pub max_mode : u32,
	pub mode : u32,
	info : *const GOPModeInformation,
	size_of_info : UIntN,
	frame_buffer_base : PhysicalAddress,
	frame_buffer_size : UIntN,
}



#[repr(C)]
pub struct Guid {
	data1 : u32,
	data2 : u16,
	data3 : u16,
	data4 : [u8; 8],
}

#[derive(Debug)]
#[repr(C)]
pub struct GraphicsOutputProtocol {
	query_mode : GOPQueryMode,
	set_mode : GOPSetMode,
	blt : GOPBlt,
	pub mode : *const GOPMode,
}

impl GraphicsOutputProtocol {

	pub unsafe fn get_framebuffer(&self) -> Option<&mut [u32]> {
		let frame_buffer_base = self.mode.as_ref()?.frame_buffer_base;
		let frame_buffer_size = self.mode.as_ref()?.frame_buffer_size;
		Some(core::slice::from_raw_parts_mut(frame_buffer_base as *mut u32, frame_buffer_size))
	}

	pub unsafe fn get_pixels_per_line(&self) -> Option<usize> {
		Some(self.mode.as_ref()?.info.as_ref()?.vertical_resolution as usize)
		//Some(self.mode.as_ref()?.info.as_ref()?.pixels_per_scanline as usize)
	}

	pub fn query_mode(&self, mode : u32) -> Option<*const GOPModeInformation>{
		let mut info_pointer : *const GOPModeInformation = core::ptr::null();
		let mut size_of_info : UIntN = 42;
		let status = (self.query_mode)(self, mode, &mut size_of_info, &mut info_pointer);
		match status {
			StatusCode::SUCCESS => Some(info_pointer),
			_ => None,
		}
	}

	pub unsafe fn current_mode(&self) -> Option<u32> {
		Some(self.mode.as_ref()?.mode)
	}
	
	pub fn set_mode(&mut self, mode : u32){

		if self.set_mode as u64 == 0 {
			unsafe { triple_fault(); }
		}

		if mode != unsafe { (*self.mode).mode } {
			unsafe { triple_fault(); }
		}

		if self as *const Self as u64 == 0 {
			unsafe { triple_fault(); }
		}

		unsafe { asm!("sti");};
		let status = (self.set_mode)(self, mode);

		unsafe { triple_fault(); }

		if status != StatusCode::SUCCESS { panic!("Could not set mode"); }		
	}
}

#[repr(C)]
struct TableHeader {
	signature : u64,
	revision : u32,
	header_size : u32,
	crc : u32,
	reserved : u32,
}

#[repr(C)]
struct BootServices {
	header : TableHeader,

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
	locate_protocol : LocateProtocol,
}

#[repr(C)]
pub struct SystemTable {
	header : TableHeader,
	firmware_vendor : *const u16,
	firmware_revision : u32,
	console_in_handle : u64,
	text_in_protocol : *const c_void,
	console_out_handle : *const c_void,
	text_out_protocol : *const c_void,
	console_stderr_handle : *const c_void,
	text_stderr_protocol : *const c_void,
	runtime_services : *const c_void,
	boot_services : *const BootServices,
	number_of_table_entries : UIntN,
		
}

#[repr(C)]
#[derive(PartialEq)]
struct StatusCode(usize);

impl StatusCode {
	pub const SUCCESS : StatusCode = StatusCode(0);
}

pub fn locate_gop(system_table : &SystemTable) -> Option<*mut GraphicsOutputProtocol> {
	let registration = core::ptr::null();
	let mut interface : *const c_void = core::ptr::null();
	let protocol_pointer = &GRAPHICS_OUTPUT_PROTOCOL_GUID as *const Guid;
	let boot_services : &BootServices = unsafe { &*system_table.boot_services };
	let function = boot_services.locate_protocol;
	if let StatusCode::SUCCESS = function(protocol_pointer, registration, &mut interface) {
		return Some(interface as *mut GraphicsOutputProtocol);
	} else {
		return None;
	}
}
