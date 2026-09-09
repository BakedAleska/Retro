use serde::Deserialize;

/// Behavioral differences between CHIP-8 interpreters that several opcodes
/// are ambiguous about. See <https://chip8.gulrak.net/> for a full survey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quirks {
    /// 8XY6/8XYE shift VX in place instead of shifting VY into VX.
    pub shift_uses_vx: bool,
    /// BNNN jumps to NNN + VX (indexed by the opcode's own X) instead of NNN + V0.
    pub jump_offset_vx: bool,
    /// FX55/FX65 leave I unchanged instead of incrementing it past the last register touched.
    pub memory_leaves_i: bool,
    /// 8XY1/8XY2/8XY3 (OR/AND/XOR) reset VF to 0 after the operation.
    pub vf_reset_on_logic: bool,
    /// DXYN clips sprites at the screen edge instead of wrapping them around.
    pub clip_sprites: bool,
}

/// The two interpreter behaviors `config.toml` can select between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Platform {
    #[serde(rename = "CHIP-8")]
    Chip8,
    #[serde(rename = "CHIP-48")]
    Chip48,
}

impl Platform {
    pub fn quirks(self) -> Quirks {
        match self {
            Platform::Chip8 => Quirks {
                shift_uses_vx: false,
                jump_offset_vx: false,
                memory_leaves_i: false,
                vf_reset_on_logic: true,
                clip_sprites: true,
            },
            Platform::Chip48 => Quirks {
                shift_uses_vx: true,
                jump_offset_vx: true,
                memory_leaves_i: true,
                vf_reset_on_logic: false,
                clip_sprites: true,
            },
        }
    }
}
