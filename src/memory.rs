use x86_64::{
    structures::paging::{
        PageTable,
        OffsetPageTable,
        Page,
        PhysFrame,
        Mapper,
        Size4KiB,
        FrameAllocator,
        PageTableFlags as Flags,
    },
    VirtAddr,
    PhysAddr,
    registers::control::Cr3
};
use bootloader::bootinfo::{ MemoryMap, MemoryRegionType };

/// A FrameAllocator that always returns `None`.
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator  {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
///  A 'static reference to the memory map passed by the bootloader and a next field
/// that keeps track of number of the next frame that the allocator should return.
pub struct BootFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootFrameAllocator{
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    /// The return type of the function uses the impl Trait feature. This way,
    /// we can specify that we return some type that implements the Iterator
    /// trait with item type PhysFrame, but don’t need to name the concrete return type.
    /// This is important here because we can’t name the concrete type since it
    /// depends on unnamable closure types.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get the usable regions from the memory map.
        let regions = self.memory_map.iter();
        // map each region to its address range,
        // transform to an iterator of frame start addresses
        let frame_addresses = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096));

        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// Creates an example mapping for the given page to frame `0xb8000`.
/// Deprecated, Pls don't use it anymore.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // this is for example purpose, it is not safe.
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
/// The function takes the physical_memory_offset as an argument and
/// returns a new OffsetPageTable instance with a 'static lifetime.
/// This means that the instance stays valid for the complete runtime of our kernel.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable {
    // read the physical frame of the active level 4 table from the CR3 register.
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // deference the pointer to get the value stored in the memory address,
    // return a mutable reference to that value.
    &mut *page_table_ptr
}

// NB: Leaving this for reference purpose, it's not needed anymore.
/// Private function that is called by `translate_addr`.
///
/// This function is safe to limit the scope of `unsafe` because Rust treats
/// the whole body of unsafe functions as an unsafe block. This function must
/// only be reachable through `unsafe fn` from outside of this module.
fn translate_addr_inner(addr: VirtAddr, physical_mem_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active level 4 frame from the CR3 register.
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];

    let mut frame = level_4_table_frame;

    // traverse the multi-level page table.
    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = physical_mem_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the page table entry and update `frame`.
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages are not supported!"),
        };
    }

    // calculate the physical address by adding the page offset.
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
