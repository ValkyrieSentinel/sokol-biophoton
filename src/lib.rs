#![cfg_attr(not(feature = "std"), no_std)]

use core::mem::size_of;

fn no_std_sin(mut x: f32) -> f32 {
    let pi = core::f32::consts::PI;
    let tau = 2.0 * pi;
    x = x % tau;
    if x < 0.0 { x += tau; }
    if x > pi { return -no_std_sin(x - pi); }
    let x2 = x * x;
    x * (1.0 - x2 / 6.0 + (x2 * x2) / 120.0)
}

fn no_std_cos(x: f32) -> f32 {
    no_std_sin(x + core::f32::consts::FRAC_PI_2)
}

#[derive(Debug, Clone, Copy)]
pub struct BioMetabolism {
    pub atp: f32,
    pub stress_level: f32,
    pub atp_recovery_rate: f32,
}

impl Default for BioMetabolism {
    fn default() -> Self {
        Self {
            atp: 1.0,
            stress_level: 0.0,
            atp_recovery_rate: 0.02,
        }
    }
}

impl BioMetabolism {
    pub fn regenerate(&mut self) {
        self.atp = f32::min(1.0, self.atp + self.atp_recovery_rate);
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BiophotonPulse {
    pub magic: u16,           
    pub content_hash: u16,    
    pub threat_type: u8,      
    pub stress_intensity: u8, 
    pub phase: u8,            
    pub emitter_id: u16,    
}

impl BiophotonPulse {
    pub const MAGIC: u16 = 0xB10F;
    pub const SIZE: usize = size_of::<Self>(); 

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts((self as *const Self) as *const u8, Self::SIZE) }
    }

    #[inline(always)]
    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let pulse = unsafe { &*(bytes.as_ptr() as *const Self) };
        if pulse.magic == Self::MAGIC {
            Some(pulse)
        } else {
            None
        }
    }
}

pub struct BiophotonEmitter {
    pub node_id: u16,
    pub threshold: f32,
    pub current_phase: u8,
}

impl BiophotonEmitter {
    pub const fn new(node_id: u16, threshold: f32) -> Self {
        Self {
            node_id,
            threshold,
            current_phase: 0,
        }
    }

    pub fn emit_if_stressed(
        &mut self,
        metabolism: &mut BioMetabolism,
        network_rtt_ms: u32,
        content_hash: u16,
        buffer: &mut [u8],
    ) -> Option<usize> {
        if metabolism.stress_level >= self.threshold && metabolism.atp >= 0.1 {
            if buffer.len() < BiophotonPulse::SIZE {
                return None;
            }

            let cost = metabolism.stress_level * 0.15;
            metabolism.atp = f32::max(0.0, metabolism.atp - cost);

            let doppler_shift = (network_rtt_ms % 255) as u8;
            self.current_phase = self.current_phase.wrapping_add(32).wrapping_add(doppler_shift);

            let pulse = BiophotonPulse {
                magic: BiophotonPulse::MAGIC,
                content_hash,
                threat_type: if metabolism.atp < 0.2 { 1 } else { 3 },
                stress_intensity: (metabolism.stress_level * 255.0) as u8,
                phase: self.current_phase,
                emitter_id: self.node_id,
            };

            buffer[..BiophotonPulse::SIZE].copy_from_slice(pulse.as_bytes());
            Some(BiophotonPulse::SIZE)
        } else {
            None
        }
    }
}

pub struct BiophotonReceptor {
    pub base_sensitivity: f32,
    pub decay_rate: f32,
    pub wave_field_real: f32,
    pub wave_field_imag: f32,
    pub active_emitters: [u16; 8],
    pub emitter_count: usize,
    pub blacklisted_emitters: [u16; 4], 
    pub blacklist_count: usize,
}

impl BiophotonReceptor {
    pub const fn new(sensitivity: f32, decay_rate: f32) -> Self {
        Self {
            base_sensitivity: sensitivity,
            decay_rate,
            wave_field_real: 0.0,
            wave_field_imag: 0.0,
            active_emitters: [0; 8],
            emitter_count: 0,
            blacklisted_emitters: [0; 4],
            blacklist_count: 0,
        }
    }

    pub fn is_blacklisted(&self, id: u16) -> bool {
        for i in 0..self.blacklist_count {
            if self.blacklisted_emitters[i] == id {
                return true;
            }
        }
        false
    }

    pub fn neutralize(&mut self, id: u16) {
        if !self.is_blacklisted(id) && self.blacklist_count < 4 {
            self.blacklisted_emitters[self.blacklist_count] = id;
            self.blacklist_count += 1;
        }
    }

    fn register_emitter(&mut self, id: u16) {
        for i in 0..self.emitter_count {
            if self.active_emitters[i] == id {
                return;
            }
        }
        if self.emitter_count < 8 {
            self.active_emitters[self.emitter_count] = id;
            self.emitter_count += 1;
        }
    }

    pub fn quorum_multiplier(&self) -> f32 {
        1.0 + (self.emitter_count as f32 * 0.25)
    }

    pub fn absorb(&mut self, bytes: &[u8], distance_metric: f32) -> bool {
        if let Some(pulse) = BiophotonPulse::from_bytes(bytes) {
            
            if self.is_blacklisted(pulse.emitter_id) {
                return false;
            }

        
            if pulse.threat_type == 2 {
                self.neutralize(pulse.emitter_id);
                return false;
            }

            
            if pulse.stress_intensity < 15 {
                return false;
            }

            self.register_emitter(pulse.emitter_id);

            let amplitude = (pulse.stress_intensity as f32) / 255.0;
            let attenuated_amplitude = amplitude / (distance_metric * distance_metric);
            let phase_rad = (pulse.phase as f32 / 255.0) * 2.0 * core::f32::consts::PI;

            let effective_sensitivity = self.base_sensitivity * self.quorum_multiplier();

            self.wave_field_real += attenuated_amplitude * effective_sensitivity * no_std_cos(phase_rad);
            self.wave_field_imag += attenuated_amplitude * effective_sensitivity * no_std_sin(phase_rad);

            true
        } else {
            false
        }
    }

    pub fn current_energy(&self) -> f32 {
        (self.wave_field_real * self.wave_field_real) + (self.wave_field_imag * self.wave_field_imag)
    }

    pub fn tick(&mut self, metabolism: &mut BioMetabolism) {
        metabolism.regenerate();

        self.wave_field_real *= 1.0 - self.decay_rate;
        self.wave_field_imag *= 1.0 - self.decay_rate;

        if self.emitter_count > 0 && self.current_energy() < 0.01 {
            self.emitter_count = 0;
        }

        let energy = self.current_energy();
        if energy > 0.05 {
            let induced_stress = energy * 0.2;
            metabolism.stress_level = f32::min(1.0, metabolism.stress_level + induced_stress);
        }
    }
}