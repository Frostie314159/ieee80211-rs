pub trait Encapsulatable {
    /// Generate the additional authenticated data (AAD).
    fn generate_aad(&self, buffer: &mut [u8; 30]) -> usize;
}
