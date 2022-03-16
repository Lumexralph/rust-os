ifeq (, $(shell which qemu-system-x86_64))
$(warning "could not find quemu tool in $(PATH)")
endif

build-os:
	cargo bootimage

run-os:
	qemu-system-x86_64 -drive format=raw,file=target/x86_64-rust_os/debug/bootimage-rust_os.bin