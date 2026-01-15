// Tue Jan 13 2026 - Alex

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write, BufReader, BufWriter};

pub struct BinaryUtils;

impl BinaryUtils {
    pub fn read_u8(data: &[u8], offset: usize) -> Option<u8> {
        data.get(offset).copied()
    }

    pub fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
        if offset + 2 > data.len() {
            return None;
        }
        Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    pub fn read_u16_be(data: &[u8], offset: usize) -> Option<u16> {
        if offset + 2 > data.len() {
            return None;
        }
        Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
    }

    pub fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
        if offset + 4 > data.len() {
            return None;
        }
        Some(u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]))
    }

    pub fn read_u32_be(data: &[u8], offset: usize) -> Option<u32> {
        if offset + 4 > data.len() {
            return None;
        }
        Some(u32::from_be_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ]))
    }

    pub fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
        if offset + 8 > data.len() {
            return None;
        }
        Some(u64::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]))
    }

    pub fn read_u64_be(data: &[u8], offset: usize) -> Option<u64> {
        if offset + 8 > data.len() {
            return None;
        }
        Some(u64::from_be_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ]))
    }

    pub fn read_i8(data: &[u8], offset: usize) -> Option<i8> {
        data.get(offset).map(|&b| b as i8)
    }

    pub fn read_i16_le(data: &[u8], offset: usize) -> Option<i16> {
        Self::read_u16_le(data, offset).map(|v| v as i16)
    }

    pub fn read_i32_le(data: &[u8], offset: usize) -> Option<i32> {
        Self::read_u32_le(data, offset).map(|v| v as i32)
    }

    pub fn read_i64_le(data: &[u8], offset: usize) -> Option<i64> {
        Self::read_u64_le(data, offset).map(|v| v as i64)
    }

    pub fn read_f32_le(data: &[u8], offset: usize) -> Option<f32> {
        Self::read_u32_le(data, offset).map(f32::from_bits)
    }

    pub fn read_f64_le(data: &[u8], offset: usize) -> Option<f64> {
        Self::read_u64_le(data, offset).map(f64::from_bits)
    }

    pub fn write_u8(data: &mut [u8], offset: usize, value: u8) -> bool {
        if offset < data.len() {
            data[offset] = value;
            true
        } else {
            false
        }
    }

    pub fn write_u16_le(data: &mut [u8], offset: usize, value: u16) -> bool {
        if offset + 2 <= data.len() {
            let bytes = value.to_le_bytes();
            data[offset] = bytes[0];
            data[offset + 1] = bytes[1];
            true
        } else {
            false
        }
    }

    pub fn write_u32_le(data: &mut [u8], offset: usize, value: u32) -> bool {
        if offset + 4 <= data.len() {
            let bytes = value.to_le_bytes();
            data[offset..offset + 4].copy_from_slice(&bytes);
            true
        } else {
            false
        }
    }

    pub fn write_u64_le(data: &mut [u8], offset: usize, value: u64) -> bool {
        if offset + 8 <= data.len() {
            let bytes = value.to_le_bytes();
            data[offset..offset + 8].copy_from_slice(&bytes);
            true
        } else {
            false
        }
    }

    pub fn read_cstring(data: &[u8], offset: usize, max_len: usize) -> Option<String> {
        if offset >= data.len() {
            return None;
        }

        let end = data.len().min(offset + max_len);
        let slice = &data[offset..end];

        let null_pos = slice.iter().position(|&b| b == 0)?;
        std::str::from_utf8(&slice[..null_pos]).ok().map(|s| s.to_string())
    }

    pub fn read_fixed_string(data: &[u8], offset: usize, len: usize) -> Option<String> {
        if offset + len > data.len() {
            return None;
        }

        let slice = &data[offset..offset + len];
        let trimmed = slice.iter()
            .position(|&b| b == 0)
            .map(|pos| &slice[..pos])
            .unwrap_or(slice);

        std::str::from_utf8(trimmed).ok().map(|s| s.to_string())
    }

    pub fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
        data.windows(pattern.len())
            .position(|window| window == pattern)
    }

    pub fn find_pattern_with_mask(data: &[u8], pattern: &[u8], mask: &[u8]) -> Option<usize> {
        if pattern.len() != mask.len() {
            return None;
        }

        data.windows(pattern.len()).position(|window| {
            window.iter()
                .zip(pattern.iter())
                .zip(mask.iter())
                .all(|((d, p), m)| *m == 0 || d == p)
        })
    }

    pub fn find_all_patterns(data: &[u8], pattern: &[u8]) -> Vec<usize> {
        let mut results = Vec::new();
        let mut offset = 0;

        while let Some(pos) = Self::find_pattern(&data[offset..], pattern) {
            results.push(offset + pos);
            offset += pos + 1;
        }

        results
    }

    pub fn hex_dump(data: &[u8], offset: usize, len: usize) -> String {
        let mut result = String::new();
        let start = offset;
        let end = (offset + len).min(data.len());

        for (i, chunk) in data[start..end].chunks(16).enumerate() {
            let addr = start + i * 16;
            result.push_str(&format!("{:08x}  ", addr));

            for (j, &byte) in chunk.iter().enumerate() {
                if j == 8 {
                    result.push(' ');
                }
                result.push_str(&format!("{:02x} ", byte));
            }

            for _ in chunk.len()..16 {
                result.push_str("   ");
            }
            if chunk.len() <= 8 {
                result.push(' ');
            }

            result.push_str(" |");
            for &byte in chunk {
                let c = if byte >= 0x20 && byte < 0x7f { byte as char } else { '.' };
                result.push(c);
            }
            result.push_str("|\n");
        }

        result
    }

    pub fn compare_regions(data1: &[u8], data2: &[u8]) -> Vec<(usize, u8, u8)> {
        let min_len = data1.len().min(data2.len());
        let mut differences = Vec::new();

        for i in 0..min_len {
            if data1[i] != data2[i] {
                differences.push((i, data1[i], data2[i]));
            }
        }

        differences
    }

    pub fn xor_bytes(data: &mut [u8], key: &[u8]) {
        if key.is_empty() {
            return;
        }

        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
    }

    pub fn compute_checksum(data: &[u8]) -> u32 {
        let mut sum: u32 = 0;
        for &byte in data {
            sum = sum.wrapping_add(byte as u32);
        }
        sum
    }

    pub fn compute_crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;

        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }

        !crc
    }

    pub fn load_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn save_file<P: AsRef<Path>>(path: P, data: &[u8]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(data)?;
        Ok(())
    }
}

pub fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    BinaryUtils::read_u32_le(data, offset)
}

pub fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
    BinaryUtils::read_u64_le(data, offset)
}

pub fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    BinaryUtils::find_pattern(data, pattern)
}

pub fn hex_dump(data: &[u8]) -> String {
    BinaryUtils::hex_dump(data, 0, data.len())
}
