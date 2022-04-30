use ash::vk;

pub struct BufferPool {
    current_buffer: vk::Buffer,
    current_offset: usize,
    current_size: usize,
    owned_buffers: Vec<vk::Buffer>,
}

impl BufferPool {
    pub fn reserve_memory(&mut self, data: &[u8], alignment: u32) -> BufferAllocation {
        let mut base_offset = Self::next_aligned(self.current_offset, alignment);
        let mut end_offset = base_offset + data.len();

        if end_offset >= self.current_size {
            self.allocate_buffer(data.len());
            assert!(self.current_offset == 0 && self.current_size >= data.len());
            base_offset = 0;
            end_offset = data.len();
        }

        self.current_offset = end_offset;

        // TODO move data

        BufferAllocation {
            buffer: self.current_buffer,
            offset: base_offset,
        }
    }

    fn allocate_buffer(&mut self, min_size: usize) {
        todo!()
    }

    fn next_aligned(offset: usize, alignment: u32) -> usize {
        let alignment = alignment as usize;
        let diff = offset % alignment;
        if diff == 0 {
            offset
        } else {
            offset + (alignment - diff)
        }
    }
}

pub struct BufferAllocation {
    pub buffer: vk::Buffer,
    pub offset: usize,
}