pub trait SpiDmaWrite {
    type Error;
    type DmaBuffer: AsRef<[u8]>;
    
    /// Synchronous read/write
    fn write_sync<B: AsRef<[u8]>>(&mut self, buffer: B) -> Result<(), Self::Error>;

    /// Asynchronous (DMA) write
    fn write_async(&mut self, buffer: Self::DmaBuffer) -> Result<(), Self::Error>;

    /// Wait for DMA completion
    fn flush(&mut self) -> Result<(), Self::Error>;
}
