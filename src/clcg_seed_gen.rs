// clcg with 2 lcg z and w 
const MOD1: i64 = 2_147_483_563;
const MOD2: i64 = 2_147_483_399;
const MULT1: i64 = 40_014;
const MULT2: i64 = 40_692;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clcg {
    z: i64,
    w: i64,
}
impl Clcg {
    pub fn new(seed: u64) -> Self {
        let z = (seed % (MOD1 as u64 - 1)).max(1) as i64;
        let w = ((seed >> 32) % (MOD2 as u64 - 1)).max(1) as i64;
        Self { z, w }
    }
    pub fn next_u32(&mut self) -> u32 {
        self.z = (self.z * MULT1) % MOD1;
        self.w = (self.w * MULT2) % MOD2;
        let mut u = self.z - self.w;
        if u < 0 {
            u += MOD1;
        }
        u as u32
    }
    /// Uniform float in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / (u32::MAX as f64 + 1.0)
    }
    pub fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        (hi << 32) | lo as u64
    }
}