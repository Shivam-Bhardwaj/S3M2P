# PLL Design Engine - Technical Documentation

Comprehensive Phase-Locked Loop (PLL) design algorithms for automatic circuit generation.

## Theory of Operation

### PLL Block Diagram

```
┌────────┐   ┌─────┐   ┌────────────┐   ┌─────┐   ┌─────────┐
│  f_ref │──→│ ÷R  │──→│    PFD     │──→│ CP  │──→│ Filter  │──┐
└────────┘   └─────┘   │            │   └─────┘   │  Z(s)   │  │
                       │            │              └─────────┘  │
                       └─────▲──────┘                           │
                             │                                  │
                   ┌─────┐   │                        ┌─────┐  │
            f_out ◀│ VCO │◀──┼────────────────────────│     │◀─┘
                   └─────┘   │                        │ Vt  │
                             │        ┌─────┐         └─────┘
                             └────────│ ÷N  │
                                      └─────┘
```

### Transfer Functions

**Open-Loop Gain:**
```
           K_phi * K_vco      Z(s)
G(s)H(s) = ─────────────  *  ─────
                 N              s
```

Where:
- `K_phi` = Phase detector gain (A/rad) = Charge pump current / 2π
- `K_vco` = VCO gain (rad/s/V)
- `N` = Feedback divider
- `Z(s)` = Loop filter impedance
- `s` = Complex frequency (jω)

**Closed-Loop Transfer Function:**
```
       G(s)
H(s) = ─────────
       1 + G(s)
```

**Phase Transfer Function (input to output phase):**
```
       φ_out(s)      N · G(s)
H(s) = ────────  =  ──────────
       φ_ref(s)     1 + G(s)
```

## Loop Filter Design

### Passive 2nd Order (Type II)

```
    ┌───────R1──────┬─────┐
    │               │     │
    │              C2     │  Vtune
Icp │───────┐       │     │    to
    │       │       │     │   VCO
    │      C1       │     │
    │       │       │     │
    └───────┴───────┴─────┘
            │
           GND
```

#### Transfer Function

```
Z(s) = (1 + sR1C1) / (sC1(1 + sR1C2))
```

This creates:
- **Zero** at ω_z = 1/(R1C1) - provides phase boost
- **Pole** at ω_p = 1/(R1C2) - attenuates high-frequency noise
- **Pole at origin** from C1 - ensures Type II behavior (zero steady-state phase error)

#### Design Equations

Given:
- ω_c = 2πf_c (crossover frequency, rad/s)
- PM = Phase margin (degrees)
- K = K_phi * K_vco / N (loop gain)

**Time Constants:**
```
T1 = R1·C1 = (1/cos(PM) - tan(PM)) / ω_c

T2 = R1·C2 = 1 / (ω_c² · T1)
```

**Component Values:**
```
C1 = K / ω_c²

R1 = T1 / C1

C2 = T2 / R1
```

#### Derivation

The open-loop gain magnitude at crossover:
```
|G(jω_c)| = 1  (0 dB)
```

The phase at crossover determines stability:
```
∠G(jω_c) = -180° + PM
```

For the 2nd order filter:
```
∠Z(jω_c) = arctan(ω_c·R1·C1) - 90° - arctan(ω_c·R1·C2)
```

The zero provides +90° phase boost (ideal), the pole at C2 provides negative phase.
The design equations balance these to achieve the target phase margin.

### Passive 3rd Order

Adds an additional RC stage for reference spur attenuation:

```
    ┌───R1───┬─────┬───R2───┐
    │        │     │        │
   C1       C2     │       C3
    │        │     │        │
    └────────┴─────┴────────┘
```

The 3rd pole is placed at ω_p3 = 5-10 × ω_c to:
- Attenuate reference spurs at f_pfd
- Minimize impact on phase margin (< 5° phase loss)

**Design:**
```
C3 = C2 / 5

R2 = 1 / (ω_p3 · C3)     where ω_p3 = 7·ω_c
```

## Stability Analysis

### Bode Plot Method

The stability is determined from the open-loop Bode plot:

1. **Phase Margin (PM):**
   ```
   PM = 180° + ∠G(jω_c)
   ```
   Where ω_c is the crossover frequency (|G(jω_c)| = 0 dB)

   **Guidelines:**
   - PM < 30°: Unstable or oscillatory
   - PM = 45°: Slight overshoot (10-20%)
   - PM = 60°: Critically damped
   - PM > 70°: Overdamped, slow response

2. **Gain Margin (GM):**
   ```
   GM = -|G(jω_180)|
   ```
   Where ω_180 is the frequency at ∠G(jω) = -180°

   **Guidelines:**
   - GM > 6 dB: Stable
   - GM < 3 dB: Marginally stable

### Complex Number Evaluation

For AC analysis at s = jω:

**Loop Filter Impedance:**
```
              1 + jωR1C1
Z(jω) = ──────────────────────
         jωC1(1 + jωR1C2)
```

Numerator: `Z_num = 1 + jωR1C1`
Denominator: `Z_den = jωC1 - ω²R1C1C2`

**Open-Loop Gain:**
```
G(jω)H(jω) = (K/jω) · Z(jω)
```

The 1/jω term represents the integrator from the VCO.

### Lock Time Estimation

The lock time (settling time to within ±Δf) is approximately:
```
t_lock ≈ (1 / (2·ζ·ω_n)) · ln(Δf_initial / Δf_final)
```

For Type II PLL with ζ ≈ PM/100 (in radians):
```
t_lock ≈ 5 / ω_c
```

This is a conservative estimate (5 time constants).

## Integer-N PLL Design

### Divider Ratio Calculation

Given:
- f_ref: Reference frequency
- f_out: Desired output frequency

**PFD Frequency:**
```
f_pfd = f_ref / R
```

**Output Frequency:**
```
f_out = f_pfd · N = f_ref · (N/R)
```

**Design Strategy:**
1. Maximize f_pfd for best phase noise and lock time
2. Start with R = 1
3. Increase R if N > N_max (typically 65535 for most PLL ICs)
4. Ensure f_pfd ≥ 10 × f_bw for stability
5. Ensure f_pfd ≥ 1 MHz to avoid flicker noise

### Prescaler

For very high N values (> 1024), use a dual-modulus prescaler (P/P+1):

```
N_effective = P·A + B

where 0 ≤ B < P
```

Common prescaler ratios: P = 8, 16, 32, 64

## Phase Noise

### Noise Contributors

1. **Reference Oscillator:**
   ```
   L_ref(f) at f_out = L_ref(f) + 20·log10(N)
   ```
   The reference noise is multiplied by N²

2. **PFD + Charge Pump:**
   ```
   L_pfd(f) = 10·log10(F·k·T / P_signal)
   ```
   Where F is the noise figure, typically 3-6 dB

3. **VCO:**
   ```
   L_vco(f) = L_vco,free(f) · |1 - H(f)|²
   ```
   VCO noise is shaped by the high-pass response (1 - H(s))

4. **Frequency Divider:**
   ```
   L_div(f) = 20·log10(N) + noise_floor
   ```

### Total Phase Noise

Inside the loop bandwidth (f < f_bw):
- Dominated by reference, PFD, and divider noise
- Multiplied by N²

Outside the loop bandwidth (f > f_bw):
- Dominated by VCO free-running noise
- Loop provides no correction

## E-Series Component Values

Standard resistor and capacitor values follow geometric progressions:

**E24 Series** (5% tolerance):
Multiplier k = 24√10 ≈ 1.10
Values: 1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0...

**E96 Series** (1% tolerance):
Multiplier k = 96√10 ≈ 1.024
Values: 1.00, 1.02, 1.05, 1.07, 1.10, 1.13, 1.15, 1.18, 1.21...

The algorithm finds the nearest value by:
1. Normalizing target to 1-10 range: `v_norm = target / 10^floor(log10(target))`
2. Finding minimum |v_norm - E_series[i]|
3. Scaling back: `result = E_series[i] × 10^floor(log10(target))`

## Example Design

**Requirements:**
- f_ref = 10 MHz
- f_out = 2.4 GHz
- f_bw = 100 kHz
- PM = 45°

**Step 1: Dividers**
```
R = 1 (maximize PFD frequency)
f_pfd = 10 MHz
N = 2400 / 10 = 240
```

**Step 2: Loop Parameters**
```
K_phi = 1 mA / 2π = 159 µA/rad
K_vco = 10 MHz/V × 2π = 62.8 Mrad/s/V
ω_c = 2π × 100 kHz = 628 krad/s
```

**Step 3: Loop Filter**
```
T1 = (1/cos(45°) - tan(45°)) / 628k = 0.226 µs
T2 = 1 / (628k² × 0.226µ) = 11.2 ns

C1 = (159µ × 62.8M) / (240 × 628k²) = 1.05 nF  →  1.0 nF (E24)
R1 = 0.226µ / 1.0n = 226 Ω  →  220 Ω (E24)
C2 = 11.2n / 220 = 51 pF  →  51 pF (E24)
```

**Step 4: Verify Stability**
Generate Bode plot and verify:
- PM ≈ 45° ± 5°
- GM > 6 dB
- f_crossover ≈ 100 kHz

## References

1. Gardner, F.M. "Phaselock Techniques", 3rd Ed., Wiley, 2005
2. Best, R.E. "Phase-Locked Loops", 6th Ed., McGraw-Hill, 2007
3. Banerjee, D. "PLL Performance, Simulation and Design", 4th Ed., 2006
4. Razavi, B. "RF Microelectronics", 2nd Ed., Prentice Hall, 2012
5. Egan, W.F. "Frequency Synthesis by Phase Lock", 2nd Ed., Wiley, 2000
