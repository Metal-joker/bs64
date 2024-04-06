use thiserror::Error;

pub mod avx2;
pub mod simple;

/// The error type for encoding and decoding.
#[derive(Error, Debug)]
pub enum CodecError {
    #[error("codec error")]
    CodecError(#[from] std::io::Error),
    #[error("output length {0} is < expected length {1}")]
    OutputLengthTooShort(usize, usize),
    #[error("input length {0} != 0 % 4")]
    InputModError(usize),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unknown codec error")]
    Unknown,
}

#[derive(Default)]
pub struct EncodeOptions {}

pub fn encode_len(input_len: usize) -> usize {
    match input_len % 3 {
        0 => input_len / 3 * 4,
        _ => input_len / 3 * 4 + 4,
    }
}

pub fn decode_len(input_len: usize) -> usize {
    (input_len / 4) * 3
}

impl EncodeOptions {
    fn encode_with_fallback(&self, output: &mut [u8], input: &[u8]) -> usize {
        if std::env::consts::ARCH == "x86_64" {
            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("avx2") {
                return avx2::encode(output, input);
            }
        }
        simple::encode(input, output)
    }

    pub fn encode(self, input: &[u8]) -> String {
        let mut output = vec![0u8; encode_len(input.len())];
        self.encode_with_fallback(&mut output, input);
        unsafe { String::from_utf8_unchecked(output) }
    }

    pub fn encode_mut(self, input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
        if output.len() < encode_len(input.len()) {
            Err(CodecError::OutputLengthTooShort(
                output.len(),
                encode_len(input.len()),
            ))
        } else {
            Ok(self.encode_with_fallback(output, input))
        }
    }
}

#[derive(Default)]
pub struct DecodeOptions {}

impl DecodeOptions {
    fn decode_with_fallback(&self, output: &mut [u8], input: &[u8]) -> Result<usize, CodecError> {
        if std::env::consts::ARCH == "x86_64" {
            #[cfg(target_arch = "x86_64")]
            if is_x86_feature_detected!("avx2") {
                return avx2::decode(output, input);
            }
        }
        simple::decode(input, output)
    }

    pub fn decode(self, input: &[u8]) -> Result<Vec<u8>, CodecError> {
        let mut output = vec![0u8; decode_len(input.len())];
        let decode_len = self.decode_with_fallback(&mut output, input)?;
        output.truncate(decode_len);
        Ok(output)
    }

    pub fn decode_mut(self, input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
        if output.len() < decode_len(input.len()) {
            Err(CodecError::OutputLengthTooShort(
                output.len(),
                decode_len(input.len()),
            ))
        } else {
            self.decode_with_fallback(output, input)
        }
    }
}

pub fn encode(input: &[u8]) -> String {
    EncodeOptions::default().encode(input)
}

pub fn encode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
    EncodeOptions::default().encode_mut(input, output)
}

pub fn decode(input: &[u8]) -> Result<Vec<u8>, CodecError> {
    DecodeOptions::default().decode(input)
}

pub fn decode_mut(input: &[u8], output: &mut [u8]) -> Result<usize, CodecError> {
    DecodeOptions::default().decode_mut(input, output)
}
