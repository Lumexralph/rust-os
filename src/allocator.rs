// #[alloc_error_handler] attribute specifies a function that is called when an allocation
// error occurs, similar to how our panic handler is called when a panic occurs.


use alloc::alloc::{GlobalAlloc, Layout};
use core::alloc::Allocator;
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;

/// The #[global_allocator] attribute tells the Rust compiler which allocator instance
/// it should use as the global heap allocator. The attribute is only applicable to
/// a static that implements the GlobalAlloc trait.
/// Since the Dummy allocator is a zero sized type, we don’t need to specify any
/// fields in the initialization expression.
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x4444_4444_000;
// 100KiB, if we need more space in the future, we can increase it.
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        // convert the HEAP_START pointer to a VirtAddr type.
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;

        // convert the addresses into Page types.
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    //  map all pages of the page range to the physical frames.
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?; // apply the question mark operator to return early in the case of an error.

        // set the flags for the page to allow read and write access to the heap memory.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    // initialize the allocator after creating the heap
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
