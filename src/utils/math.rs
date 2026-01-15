// Tue Jan 13 2026 - Alex

pub struct MathUtils;

impl MathUtils {
    pub fn align_up(value: u64, alignment: u64) -> u64 {
        if alignment == 0 || alignment == 1 {
            return value;
        }
        (value + alignment - 1) & !(alignment - 1)
    }

    pub fn align_down(value: u64, alignment: u64) -> u64 {
        if alignment == 0 || alignment == 1 {
            return value;
        }
        value & !(alignment - 1)
    }

    pub fn is_aligned(value: u64, alignment: u64) -> bool {
        if alignment == 0 || alignment == 1 {
            return true;
        }
        (value & (alignment - 1)) == 0
    }

    pub fn is_power_of_two(n: u64) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }

    pub fn next_power_of_two(n: u64) -> u64 {
        if n == 0 {
            return 1;
        }
        if Self::is_power_of_two(n) {
            return n;
        }
        1u64 << (64 - n.leading_zeros())
    }

    pub fn prev_power_of_two(n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        1u64 << (63 - n.leading_zeros())
    }

    pub fn log2(n: u64) -> u32 {
        if n == 0 {
            return 0;
        }
        63 - n.leading_zeros()
    }

    pub fn count_bits(n: u64) -> u32 {
        n.count_ones()
    }

    pub fn rotate_left(value: u64, shift: u32, width: u32) -> u64 {
        let shift = shift % width;
        let mask = (1u64 << width) - 1;
        ((value << shift) | (value >> (width - shift))) & mask
    }

    pub fn rotate_right(value: u64, shift: u32, width: u32) -> u64 {
        let shift = shift % width;
        let mask = (1u64 << width) - 1;
        ((value >> shift) | (value << (width - shift))) & mask
    }

    pub fn sign_extend(value: u64, from_bits: u32) -> i64 {
        let sign_bit = 1u64 << (from_bits - 1);
        if value & sign_bit != 0 {
            (value | !((1u64 << from_bits) - 1)) as i64
        } else {
            value as i64
        }
    }

    pub fn zero_extend(value: u64, from_bits: u32) -> u64 {
        value & ((1u64 << from_bits) - 1)
    }

    pub fn extract_bits(value: u64, start: u32, count: u32) -> u64 {
        (value >> start) & ((1u64 << count) - 1)
    }

    pub fn insert_bits(value: u64, bits: u64, start: u32, count: u32) -> u64 {
        let mask = ((1u64 << count) - 1) << start;
        (value & !mask) | ((bits << start) & mask)
    }

    pub fn set_bit(value: u64, bit: u32) -> u64 {
        value | (1u64 << bit)
    }

    pub fn clear_bit(value: u64, bit: u32) -> u64 {
        value & !(1u64 << bit)
    }

    pub fn toggle_bit(value: u64, bit: u32) -> u64 {
        value ^ (1u64 << bit)
    }

    pub fn test_bit(value: u64, bit: u32) -> bool {
        (value & (1u64 << bit)) != 0
    }

    pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    pub fn percentage(current: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (current as f64 / total as f64) * 100.0
        }
    }

    pub fn ratio(current: u64, total: u64) -> f64 {
        if total == 0 {
            0.0
        } else {
            current as f64 / total as f64
        }
    }

    pub fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    pub fn lcm(a: u64, b: u64) -> u64 {
        if a == 0 || b == 0 {
            return 0;
        }
        (a / Self::gcd(a, b)) * b
    }

    pub fn abs_diff(a: u64, b: u64) -> u64 {
        if a > b { a - b } else { b - a }
    }

    pub fn saturating_add(a: u64, b: u64, max: u64) -> u64 {
        a.saturating_add(b).min(max)
    }

    pub fn wrapping_add(a: u64, b: u64, modulus: u64) -> u64 {
        if modulus == 0 {
            return a.wrapping_add(b);
        }
        (a + b) % modulus
    }

    pub fn range_overlap(start1: u64, end1: u64, start2: u64, end2: u64) -> Option<(u64, u64)> {
        let start = start1.max(start2);
        let end = end1.min(end2);

        if start < end {
            Some((start, end))
        } else {
            None
        }
    }

    pub fn range_contains(outer_start: u64, outer_end: u64, inner_start: u64, inner_end: u64) -> bool {
        outer_start <= inner_start && inner_end <= outer_end
    }

    pub fn address_in_range(addr: u64, start: u64, size: u64) -> bool {
        addr >= start && addr < start.saturating_add(size)
    }
}

pub fn align_up(value: u64, alignment: u64) -> u64 {
    MathUtils::align_up(value, alignment)
}

pub fn align_down(value: u64, alignment: u64) -> u64 {
    MathUtils::align_down(value, alignment)
}

pub fn is_power_of_two(n: u64) -> bool {
    MathUtils::is_power_of_two(n)
}

pub fn sign_extend(value: u64, from_bits: u32) -> i64 {
    MathUtils::sign_extend(value, from_bits)
}

pub fn extract_bits(value: u64, start: u32, count: u32) -> u64 {
    MathUtils::extract_bits(value, start, count)
}

pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    MathUtils::clamp(value, min, max)
}

pub fn percentage(current: u64, total: u64) -> f64 {
    MathUtils::percentage(current, total)
}
