use core::ffi::c_void;

type LocateProtocol = extern "efiapi" fn(protocol : *const GUID, registration : *const c_void, interface : *mut *const c_void) -> StatusCode;

type PhysicalAddress = u64;
pub type Handle = *mut c_void;

type GOPQueryMode = extern "efiapi" fn(this : *const GraphicsOutputProtocol, mode_number : u32, size_of_info : *mut UIntN, info:  *mut *const GOPModeInformation) -> StatusCode;
type GOPSetMode = extern "efiapi" fn(this : *mut GraphicsOutputProtocol, mode_number : u32) -> StatusCode;
type GOPBlt = extern "efiapi" fn(this : *mut GraphicsOutputProtocol, blt_buffer : *const GOPBltPixel, blt_operation : GOPBltOperation, source_x : UIntN, source_y : UIntN, destination_x : UIntN, destination_y : UIntN, width : UIntN, height : UIntN, delta : UIntN) -> StatusCode;

// unsigned value of native width
type UIntN = usize;

pub const GRAPHICS_OUTPUT_PROTOCOL_GUID : GUID = GUID { data1 : 0x9042a9de, data2 : 0x23dc, data3 : 0x4a38, data4 : [0x96,0xfb,0x7a,0xde,0xd0,0x80,0x51,0x6a]};
pub const ACPI_TABLE_GUID : GUID = GUID { data1 : 0x8868e871, data2: 0xe4f1, data3 : 0x11d3, data4 : [0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81]};

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
#[derive(PartialEq)]
#[derive(Debug)]
pub struct GUID {
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

	pub unsafe fn get_framebuffer(&self) -> Option<*mut u32> {
		Some(self.mode.as_ref()?.frame_buffer_base as *mut u32)
	}

	pub unsafe fn get_pixel_width(&self) -> Option<usize> {
		Some(self.mode.as_ref()?.info.as_ref()?.pixels_per_scanline as usize)
	}

	pub unsafe fn get_pixel_height(&self) -> Option<usize> {
		Some(self.mode.as_ref()?.info.as_ref()?.vertical_resolution as usize)
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
		let status = (self.set_mode)(self, mode);
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
	configuration_table : *const ConfigurationTable,
}

#[repr(C)]
#[derive(Debug)]
pub struct ConfigurationTable {
	guid : GUID,
	pub data : *const c_void,
}

#[repr(C)]
#[derive(PartialEq)]
struct StatusCode(usize);

impl StatusCode {
	pub const SUCCESS : StatusCode = StatusCode(0);
}

impl SystemTable {
	pub fn get_configuration_tables(&self) -> &[ConfigurationTable] {
		unsafe { core::slice::from_raw_parts(self.configuration_table, self.number_of_table_entries) }
	}
	pub fn find_configuration_table(&self, guid : GUID) -> Option<&ConfigurationTable> {
		for table in self.get_configuration_tables() {
			if table.guid == guid {
				return Some(table);
			}
		}
		None
	}
	pub fn get_firmware_vendor(&self) -> &[u16] {
		let mut strlen = 0;
		while unsafe { *self.firmware_vendor.add(strlen) } != 0 {
			strlen += 1;
		}
		unsafe { core::slice::from_raw_parts(self.firmware_vendor, strlen) }
	}
}

pub fn locate_gop(system_table : &SystemTable) -> Option<*mut GraphicsOutputProtocol> {
	let registration = core::ptr::null();
	let mut interface : *const c_void = core::ptr::null();
	let protocol_pointer = &GRAPHICS_OUTPUT_PROTOCOL_GUID as *const GUID;
	let boot_services : &BootServices = unsafe { &*system_table.boot_services };
	let function = boot_services.locate_protocol;
	if let StatusCode::SUCCESS = function(protocol_pointer, registration, &mut interface) {
		return Some(interface as *mut GraphicsOutputProtocol);
	} else {
		return None;
	}
}
