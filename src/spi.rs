pub trait SpiDmaWrite {
    type Error;

    /// Synchronous read/write
    fn write_sync<B: AsRef<[u8]>>(&mut self, buffer: B) -> Result<(), Self::Error>;

    /// Asynchronous (DMA) write
    fn write_async<B: AsRef<[u8]>>(&mut self, buffer: B) -> Result<(), Self::Error>;

    /// Wait for DMA completion
    fn flush(&mut self) -> Result<(), Self::Error>;
}
