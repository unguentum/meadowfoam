OUTPUT_FORMAT("elf64-x86-64")
ENTRY(_entry)

SECTIONS {
	. = 1M;

	.multiboot_header ALIGN(0x10) : {
		KEEP(*(.multiboot_header))
	}

	.text : {
		*(.text*)
	}

	.rodata : {
		*(.rodata*)
	}

	.data : {
		*(.data*)
	}

	.bss : {
		*(COMMON)
		*(.bss*)
	}

	.shstrtab : {
		*(.shstrtab)
	}

	#.rest : {
	#	*(*)
	#}

}
