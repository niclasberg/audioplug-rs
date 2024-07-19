use std::marker::PhantomData;

pub struct AudioBuffer {
    num_channels: usize,
    num_samples: usize,
    channel_samples: *const *mut f32
}

impl AudioBuffer {
    pub fn from_slice<const NCHANNELS: usize, const NSAMPLES: usize>(channel_samples: &[&mut [f32; NSAMPLES]; NCHANNELS]) -> Self {
        let channel_samples = channel_samples as *const _;
        let channel_samples = unsafe { std::mem::transmute(channel_samples) };
        Self { 
            num_channels: NCHANNELS,
            num_samples: NSAMPLES,
            channel_samples
        }
    }

    pub unsafe fn from_ptr(channel_samples: *const *mut f32, num_channels: usize, num_samples: usize) -> Self {
        Self { num_channels, num_samples, channel_samples }
    }

    pub fn samples(&self) -> usize {
        self.num_samples
    }

    pub fn channels(&self) -> usize {
        self.num_channels
    }

    pub fn channel<'a>(&'a self, index: usize) -> ChannelSamples<'a> {
        assert!(index < self.num_channels);
        ChannelSamples {
            samples: unsafe { *self.channel_samples.offset(index as isize) },
            num_samples: self.num_samples,
            _phantom: PhantomData,
        }
    }

    pub fn channel_mut<'a>(&'a self, index: usize) -> ChannelSamplesMut<'a> {
        assert!(index < self.num_channels);
        ChannelSamplesMut {
            samples: unsafe { *self.channel_samples.offset(index as isize) },
            num_samples: self.num_samples,
            _phantom: PhantomData,
        }
    }

    pub fn channels_iter<'a>(&'a self) -> ChannelsIter<'a> {
        ChannelsIter { 
            current_channel_samples: self.channel_samples as *const _, 
            end_channel_samples: unsafe { self.channel_samples.offset(self.num_channels as isize) as *const _ }, 
            num_samples: self.num_samples, 
            _phantom: PhantomData
        }
    }

    pub fn channels_iter_mut<'a>(&'a mut self) -> ChannelsIterMut<'a> {
        ChannelsIterMut { 
            current_channel_samples: self.channel_samples, 
            end_channel_samples: unsafe { self.channel_samples.offset(self.num_channels as isize) as *const _ }, 
            num_samples: self.num_samples, 
            _phantom: PhantomData 
        }
    }
}

pub struct FrameIterator<'a> {
    _phantom: PhantomData<&'a f32>
}


pub struct FrameSamples<'a> {
    _phantom: PhantomData<&'a f32>
}



pub struct ChannelsIter<'a> {
    current_channel_samples: *const *const f32,
    end_channel_samples: *const *const f32,
    num_samples: usize,
    _phantom: PhantomData<&'a &'a f32>
}

impl<'a> Iterator for ChannelsIter<'a> {
    type Item = ChannelSamples<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel_samples == self.end_channel_samples {
            None
        } else {
            let channel_samples = unsafe { 
				// Weird, tried to derefence but ableton gave us an unaligned pointer which caused 
				// panic. Use read_unaligned instead
                let samples = self.current_channel_samples.read_unaligned();
                ChannelSamples { samples, num_samples: self.num_samples, _phantom: PhantomData }
            };
            self.current_channel_samples = unsafe { self.current_channel_samples.offset(1) };
            Some(channel_samples)
        }
    }
}

pub struct ChannelsIterMut<'a> {
    current_channel_samples: *const *mut f32,
    end_channel_samples: *const *mut f32,
    num_samples: usize,
    _phantom: PhantomData<&'a f32>
}

impl<'a> Iterator for ChannelsIterMut<'a> {
    type Item = ChannelSamplesMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel_samples == self.end_channel_samples {
            None
        } else {
            let channel_samples = unsafe { 
				// Weird, tried to derefence but ableton gave us an unaligned pointer which caused 
				// panic. Use read_unaligned instead
                let samples = self.current_channel_samples.read_unaligned();
                ChannelSamplesMut { samples, num_samples: self.num_samples, _phantom: PhantomData }
            };
            self.current_channel_samples = unsafe { self.current_channel_samples.offset(1) };
            Some(channel_samples)
        }
    }
}

pub struct ChannelSamples<'a> {
    samples: *const f32,
    num_samples: usize,
    _phantom: PhantomData<&'a f32>
}

impl<'a> ChannelSamples<'a> {
    pub fn iter(&self) -> ChannelSamplesIter<'a> {
        ChannelSamplesIter::new(self.samples, self.num_samples)
    }

    pub fn as_slice(&self) -> &'a [f32] {
        unsafe { std::slice::from_raw_parts(self.samples, self.num_samples) }
    }

    pub fn len(&self) -> usize {
        self.num_samples
    }
}  

impl<'a> IntoIterator for ChannelSamples<'a> {
    type Item = f32;
    type IntoIter = ChannelSamplesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ChannelSamplesIter<'a> {
    current_sample: *const f32,
    last_sample: *const f32,
    _phantom: PhantomData<&'a f32>
}

impl<'a> ChannelSamplesIter<'a> {
    fn new(current_sample: *const f32, num_samples: usize) -> Self {
        Self {
            current_sample,
            last_sample: unsafe { current_sample.offset(num_samples as isize) },
            _phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for ChannelSamplesIter<'a> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample == self.last_sample {
            None
        } else {
            let sample = unsafe { *self.current_sample };
            self.current_sample = unsafe { self.current_sample.offset(1) };
            Some(sample)
        }
    }
}

pub struct ChannelSamplesMut<'a> {
    samples: *mut f32,
    num_samples: usize,
    _phantom: PhantomData<&'a mut f32>
}

impl<'a> ChannelSamplesMut<'a> {
    pub fn as_slice(&self) -> &'a [f32] {
        unsafe { std::slice::from_raw_parts(self.samples, self.num_samples) }
    }

    pub fn as_mut_slice(&mut self) -> &'a mut [f32] {
        unsafe { std::slice::from_raw_parts_mut(self.samples, self.num_samples) }
    }

    pub fn len(&self) -> usize {
        self.num_samples
    }
}

impl<'a> ChannelSamplesMut<'a> {
    pub fn iter(&self) -> ChannelSamplesIter<'a> {
        ChannelSamplesIter::new(self.samples, self.num_samples)
    }

    pub fn iter_mut(&mut self) -> ChannelSamplesIterMut<'a> {
        ChannelSamplesIterMut {
            current_sample: self.samples,
            last_sample: unsafe { self.samples.offset(self.num_samples as _) },
            _phantom: PhantomData,
        }
    }
}

pub struct ChannelSamplesIterMut<'a> {
    current_sample: *mut f32,
    last_sample: *mut f32,
    _phantom: PhantomData<&'a mut f32>
}

impl<'a> Iterator for ChannelSamplesIterMut<'a> {
    type Item = &'a mut f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample == self.last_sample {
            None
        } else {
            let sample = unsafe { &mut *self.current_sample };
            self.current_sample = unsafe { self.current_sample.offset(1) };
            Some(sample)
        }
    }
}

#[cfg(test)]
mod test {
    use super::AudioBuffer;

    #[test]
    pub fn empty_buffer() {
        let mut buffer = AudioBuffer::from_slice::<0, 0>(&[]);
        let mut channel_iter = buffer.channels_iter();
        assert!(channel_iter.next().is_none());
        let mut channel_iter = buffer.channels_iter_mut();
        assert!(channel_iter.next().is_none());
    }

    #[test]
    pub fn channel_iter() {
        let mut channel1_data = [1.0f32, 2.0f32, 3.0f32];
        let mut channel2_data = [4.0f32, 5.0, 6.0];
        let data = [&mut channel1_data, &mut channel2_data];
        let buffer = AudioBuffer::from_slice(&data);

        let mut channel_iter = buffer.channels_iter();
        let samples = channel_iter.next();
        assert!(samples.is_some());
        let mut samples_iter = samples.unwrap().iter();
        assert_eq!(samples_iter.next(), Some(1.0f32));
        assert_eq!(samples_iter.next(), Some(2.0f32));
        assert_eq!(samples_iter.next(), Some(3.0f32));
        assert_eq!(samples_iter.next(), None);
        
        let mut samples_iter = channel_iter.next().unwrap().iter();
        assert_eq!(samples_iter.next(), Some(4.0f32));
        assert_eq!(samples_iter.next(), Some(5.0f32));
        assert_eq!(samples_iter.next(), Some(6.0f32));
        assert_eq!(samples_iter.next(), None);
    }

    #[test] 
    pub fn channel_iter_mut() {
        let mut channel1_data = [0.0f32; 3];
        let mut channel2_data = [0.0f32; 3];
        let data = [&mut channel1_data, &mut channel2_data];
        let mut buffer = AudioBuffer::from_slice(&data);

        let data = [[0.0f32, 1.0, 2.0], [3.0, 4.0, 5.0]];
        for (i, mut channel) in buffer.channels_iter_mut().enumerate() {
            for (j, sample) in channel.iter_mut().enumerate() {
                *sample = data[i][j];
            }
        }

        assert_eq!(buffer.channel(0).as_slice(), data[0]);
        assert_eq!(buffer.channel(1).as_slice(), data[1]);
    }
}