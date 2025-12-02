use serde::{Deserialize, Serialize};

/// Phase noise components at a specific offset
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoiseComponents {
    pub total_dbc_hz: f64,
    pub ref_dbc_hz: f64,
    pub pfd_dbc_hz: f64,
    pub vco_dbc_hz: f64,
    pub filter_dbc_hz: f64,
    pub divider_dbc_hz: f64,
}

/// Complete phase noise profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseNoiseProfile {
    pub offsets_hz: Vec<f64>,
    pub total_dbc_hz: Vec<f64>,
    pub integrated_jitter_fs: f64,
    pub components: Vec<NoiseComponents>,
}

/// Results of a transient simulation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransientResult {
    pub time_s: Vec<f64>,
    pub freq_hz: Vec<f64>,
    pub phase_error_deg: Vec<f64>,
    pub lock_time_us: f64,
    pub overshoot_percent: f64,
}

/// PLL architecture type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PLLArchitecture {
    IntegerN,
    FractionalN,
}

/// User requirements for PLL design
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PLLRequirements {
    /// Reference oscillator frequency (Hz)
    pub ref_freq_hz: f64,
    /// Minimum output frequency (Hz)
    pub output_freq_min_hz: f64,
    /// Maximum output frequency (Hz)
    pub output_freq_max_hz: f64,
    /// Target loop bandwidth (Hz)
    pub loop_bandwidth_hz: f64,
    /// Target phase margin (degrees)
    pub phase_margin_deg: f64,
    /// PLL architecture
    pub architecture: PLLArchitecture,
    /// Supply voltage (V)
    pub supply_voltage: f64,
}

/// Divider configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DividerConfig {
    IntegerN {
        n: u32,
        prescaler: Option<u32>,
    },
    FractionalN {
        n_int: u32,
        n_frac: u32,
        modulus: u32,
        modulator_order: u32,
    },
}

/// Loop filter component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterComponent {
    pub designator: String,
    pub value: f64,
    pub actual_value: f64, // E-series value
    pub unit: String,
    pub tolerance_pct: f64,
}

/// Loop filter design
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopFilterDesign {
    pub topology: LoopFilterTopology,
    pub components: Vec<FilterComponent>,
    pub c1_pf: f64,
    pub c2_pf: f64,
    pub r1_ohms: f64,
    pub c3_pf: Option<f64>,
    pub r2_ohms: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopFilterTopology {
    PassiveSecondOrder,
    PassiveThirdOrder,
    PassiveFourthOrder,
}

/// Transfer function representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferFunction {
    pub zeros: Vec<f64>,
    pub poles: Vec<f64>,
    pub gain: f64,
}

/// Bode plot data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodePlot {
    pub frequencies_hz: Vec<f64>,
    pub magnitude_db: Vec<f64>,
    pub phase_deg: Vec<f64>,
}

/// PLL performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PLLPerformance {
    pub phase_margin_deg: f64,
    pub gain_margin_db: f64,
    pub crossover_freq_hz: f64,
    pub loop_bandwidth_hz: f64,
    pub lock_time_us: f64,
}

/// Complete PLL design result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PLLDesign {
    pub requirements: PLLRequirements,
    pub divider_r: u32,
    pub divider_n: DividerConfig,
    pub pfd_freq_hz: f64,
    pub loop_filter: LoopFilterDesign,
    pub charge_pump_current_ua: f64,
    pub vco_gain_mhz_per_v: f64,
    pub performance: PLLPerformance,
    pub bode_plot: BodePlot,
    pub phase_noise: PhaseNoiseProfile,
    pub transient: TransientResult,
}

/// Validation errors
#[derive(Clone, Debug)]
pub enum ValidationError {
    FrequencyRangeInvalid { min: f64, max: f64, reason: String },
    LoopBandwidthTooHigh { bandwidth: f64, max_allowed: f64 },
    PhaseMarginOutOfRange { margin: f64 },
    ReferenceFrequencyTooLow { freq: f64, min_required: f64 },
}

/// Validation result
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl PLLRequirements {
    /// Validate the requirements
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check frequency range
        if self.output_freq_min_hz >= self.output_freq_max_hz {
            errors.push(ValidationError::FrequencyRangeInvalid {
                min: self.output_freq_min_hz,
                max: self.output_freq_max_hz,
                reason: "Minimum frequency must be less than maximum".to_string(),
            });
        }

        // Check loop bandwidth vs reference frequency
        let max_bandwidth = self.ref_freq_hz / 10.0;
        if self.loop_bandwidth_hz > max_bandwidth {
            errors.push(ValidationError::LoopBandwidthTooHigh {
                bandwidth: self.loop_bandwidth_hz,
                max_allowed: max_bandwidth,
            });
        }

        // Check phase margin range
        if self.phase_margin_deg < 30.0 || self.phase_margin_deg > 70.0 {
            warnings.push(format!(
                "Phase margin of {:.1}° is outside typical range (30-70°)",
                self.phase_margin_deg
            ));
        }

        // Check reference frequency minimum
        if self.ref_freq_hz < 1e6 {
            warnings.push(format!(
                "Reference frequency of {:.1} MHz is very low",
                self.ref_freq_hz / 1e6
            ));
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}
