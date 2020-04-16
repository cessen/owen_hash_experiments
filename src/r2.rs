const GOLDEN_RATIO_32: u32 = (std::u32::MAX as f64 * (1.0 / 1.6180339887)) as u32;
const PLASTIC_32: u32 = (std::u32::MAX as f64 * (1.0 / 1.3247179572)) as u32;

const SQRT_2: u32 = (std::u32::MAX as f64 * 0.4142135623730) as u32;
const SQRT_3: u32 = (std::u32::MAX as f64 * 0.7320508075688) as u32;
const SQRT_5: u32 = (std::u32::MAX as f64 * 0.2360679774) as u32;

#[inline]
pub fn sample_0(index: u32) -> f32 {
    u32_to_0_1_f32(GOLDEN_RATIO_32.wrapping_mul(index))
}

#[inline]
pub fn sample_1(index: u32) -> f32 {
    u32_to_0_1_f32(PLASTIC_32.wrapping_mul(index))
}

#[inline]
pub fn sample_2(index: u32) -> f32 {
    u32_to_0_1_f32(SQRT_2.wrapping_mul(index))
}

#[inline]
pub fn sample_3(index: u32) -> f32 {
    u32_to_0_1_f32(SQRT_3.wrapping_mul(index))
}

#[inline]
pub fn sample_4(index: u32) -> f32 {
    u32_to_0_1_f32(SQRT_5.wrapping_mul(index))
}

#[inline(always)]
fn u32_to_0_1_f32(n: u32) -> f32 {
    n as f32 * (1.0 / (1u64 << 32) as f32)
}
