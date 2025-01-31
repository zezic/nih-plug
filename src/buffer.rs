// nih-plug: plugins, but rewritten in Rust
// Copyright (C) 2022 Robbert van der Helm
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// The audio buffers used during processing. This contains the output audio output buffers with the
/// inputs already copied to the outputs. You can either use the iterator adapters to conveniently
/// and efficiently iterate over the samples, or you can do your own thing using the raw audio
/// buffers.
#[derive(Default)]
pub struct Buffer<'a> {
    /// Contains slices for the plugin's outputs. You can't directly create a nested slice form
    /// apointer to pointers, so this needs to be preallocated in the setup call and kept around
    /// between process calls. And because storing a reference to this means a) that you need a lot
    /// of lifetime annotations everywhere and b) that at some point you need unsound lifetime casts
    /// because this `Buffers` either cannot have the same lifetime as the separately stored output
    /// buffers, and it also cannot be stored in a field next to it because that would mean
    /// containing mutable references to data stored in a mutex.
    output_slices: Vec<&'a mut [f32]>,
}

impl<'a> Buffer<'a> {
    /// Returns true if this buffer does not contain any samples.
    pub fn is_empty(&self) -> bool {
        self.output_slices.is_empty() || self.output_slices[0].is_empty()
    }

    /// Obtain the raw audio buffers.
    pub fn as_raw(&mut self) -> &mut [&'a mut [f32]] {
        &mut self.output_slices
    }

    /// Iterate over the samples, returning a channel iterator for each sample.
    pub fn iter_mut(&mut self) -> Samples<'_, 'a> {
        Samples {
            buffers: &mut self.output_slices,
            current_sample: 0,
        }
    }

    /// Access the raw output slice vector. This neds to be resized to match the number of output
    /// channels during the plugin's initialization. Then during audio processing, these slices
    /// should be updated to point to the plugin's audio buffers.
    ///
    /// # Safety
    ///
    /// The stored slices must point to live data when this object is passed to the plugins' process
    /// function. The rest of this object also assumes all channel lengths are equal. Panics will
    /// likely occur if this is not the case.
    pub unsafe fn as_raw_vec(&mut self) -> &mut Vec<&'a mut [f32]> {
        &mut self.output_slices
    }
}

/// An iterator over all samples in the buffer, yielding iterators over each channel for every
/// sample. This iteration order offers good cache locality for per-sample access.
pub struct Samples<'outer, 'inner> {
    /// The raw output buffers.
    pub(self) buffers: &'outer mut [&'inner mut [f32]],
    pub(self) current_sample: usize,
}

impl<'outer, 'inner> Iterator for Samples<'outer, 'inner> {
    type Item = Channels<'outer, 'inner>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample < self.buffers[0].len() {
            // SAFETY: We guarantee that each sample is only mutably borrowed once in the channels
            // iterator
            let buffers: &'outer mut _ = unsafe { &mut *(self.buffers as *mut _) };
            let channels = Channels {
                buffers,
                current_sample: self.current_sample,
                current_channel: 0,
            };

            self.current_sample += 1;

            Some(channels)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffers[0].len() - self.current_sample;
        (remaining, Some(remaining))
    }
}

impl<'outer, 'inner> ExactSizeIterator for Samples<'outer, 'inner> {}

/// An iterator over the channel data for a sample, yielded by [Samples].
pub struct Channels<'outer, 'inner> {
    /// The raw output buffers.
    pub(self) buffers: &'outer mut [&'inner mut [f32]],
    pub(self) current_sample: usize,
    pub(self) current_channel: usize,
}

impl<'outer, 'inner> Iterator for Channels<'outer, 'inner> {
    type Item = &'inner mut f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel < self.buffers.len() {
            // SAFETY: These bounds have already been checked
            let sample = unsafe {
                self.buffers
                    .get_unchecked_mut(self.current_channel)
                    .get_unchecked_mut(self.current_sample)
            };
            // SAFETY: It is not possible to have multiple mutable references to the same sample at
            // the same time
            let sample: &'inner mut f32 = unsafe { &mut *(sample as *mut f32) };

            self.current_channel += 1;

            Some(sample)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffers.len() - self.current_channel;
        (remaining, Some(remaining))
    }
}

impl<'outer, 'inner> ExactSizeIterator for Channels<'outer, 'inner> {}
