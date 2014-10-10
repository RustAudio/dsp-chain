
use std::slice::Items;
use std::slice::MutItems;
use sound_stream_settings::SoundStreamSettings;

/// A trait to be implemented by any Buffer used for audio processing in rust-dsp.
/// This is primarily implemented for fixed-size arrays where len is a power of 2.
pub trait DspBuffer {
    /// Return the value at the given index.
    fn val(&self, idx: uint) -> f32;
    /// Return an immutable reference to the value at the given index.
    fn get(&self, idx: uint) -> &f32;
    /// Return a mutable reference to the value at the given index.
    fn get_mut(&mut self, idx: uint) -> &mut f32;
    /// Return the DspBuffer as a slice.
    fn as_slice(&self) -> &[f32];
    /// Return the DspBuffer as a mutable slice.
    fn as_mut_slice(&mut self) -> &mut [f32];
    /// Return an immutable Iterator over the values.
    fn iter<'a>(&'a self) -> Items<'a, f32>;
    /// Return a mutable Iterator over the values.
    fn iter_mut<'a>(&'a mut self) -> MutItems<'a, f32>;
    /// Return a DspBuffer full of the given value.
    fn from_elem(val: f32) -> Self;
    /// Return a Zeroed DspBuffer.
    fn zeroed() -> Self;
    /// Return the length of the DspBuffer.
    fn len(&self) -> uint;
    /// Return the mono soundstream settings for this type.
    fn mono_settings(samples_per_sec: u32) -> SoundStreamSettings;
    /// Return the stereo soundstream settings for this type.
    fn stereo_settings(samples_per_sec: u32) -> SoundStreamSettings;
    /// Create a DspBuffer from a Vec.
    #[inline]
    fn from_vec(vec: &Vec<f32>) -> Self {
        let mut buffer: Self = DspBuffer::zeroed();
        for i in range(0u, buffer.len()) {
            *buffer.get_mut(i) = vec[i];
        }
        buffer
    }
    /// Create a Vec from a DspBuffer.
    fn to_vec(&self) -> Vec<f32> {
        Vec::from_fn(self.len(), |idx| self.val(idx))
    }
}

impl_dsp_buffer!([f32, ..2], 2)
impl_dsp_buffer!([f32, ..4], 4)
impl_dsp_buffer!([f32, ..8], 8)
impl_dsp_buffer!([f32, ..16], 16)
impl_dsp_buffer!([f32, ..32], 32)
impl_dsp_buffer!([f32, ..64], 64)
impl_dsp_buffer!([f32, ..128], 128)
impl_dsp_buffer!([f32, ..256], 256)
impl_dsp_buffer!([f32, ..512], 512)
impl_dsp_buffer!([f32, ..1024], 1024)
impl_dsp_buffer!([f32, ..2048], 2048)
impl_dsp_buffer!([f32, ..4096], 4096)
impl_dsp_buffer!([f32, ..8192], 8192)
impl_dsp_buffer!([f32, ..16384], 16384)

