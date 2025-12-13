//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lessons.rs | ESP32/src/lessons.rs
//! PURPOSE: ESP32 lesson definitions and curriculum structure
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → ESP32
//! ═══════════════════════════════════════════════════════════════════════════════

/// A single ESP32 lesson
pub struct Lesson {
    pub id: usize,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub intuition: &'static str,
    pub key_concepts: &'static [&'static str],
}

/// All ESP32 lessons
pub static LESSONS: &[Lesson] = &[
    Lesson {
        id: 0,
        title: "GPIO Debounce",
        subtitle: "Button Input Filtering",
        icon: "🔘",
        description: "Learn how mechanical buttons produce noisy signals and how to filter them using software debouncing techniques.",
        intuition: "When you press a physical button, the metal contacts bounce rapidly before settling. This creates multiple false triggers. Debouncing waits for the signal to stabilize before registering a press.",
        key_concepts: &["Contact Bounce", "Sample Rate", "Debounce Window", "Rising/Falling Edge", "State Machine"],
    },
    Lesson {
        id: 1,
        title: "PWM Control",
        subtitle: "Pulse Width Modulation",
        icon: "📶",
        description: "Control LED brightness and motor speed using PWM. Learn how duty cycle affects average power output.",
        intuition: "Instead of varying voltage directly, PWM rapidly switches between on and off. The ratio of on-time to total time (duty cycle) determines the perceived brightness or speed.",
        key_concepts: &["Duty Cycle", "Frequency", "Resolution", "LED Dimming", "Motor Speed"],
    },
    Lesson {
        id: 2,
        title: "ADC Reading",
        subtitle: "Analog to Digital Conversion",
        icon: "📊",
        description: "Read analog sensors like potentiometers and temperature sensors. Convert continuous voltage to discrete digital values.",
        intuition: "The real world is analog - temperatures, light levels, and voltages vary smoothly. ADC samples these continuous signals at discrete intervals and quantizes them into digital numbers.",
        key_concepts: &["Resolution (bits)", "Sampling Rate", "Voltage Reference", "Quantization", "Averaging"],
    },
    Lesson {
        id: 3,
        title: "I2C Communication",
        subtitle: "Two-Wire Serial Protocol",
        icon: "🔗",
        description: "Connect multiple sensors and displays using just two wires. Learn addressing, clock synchronization, and data framing.",
        intuition: "I2C is like a shared telephone line where each device has a unique phone number (address). The master calls out addresses and only the matching device responds.",
        key_concepts: &["SDA/SCL", "Address", "Start/Stop", "ACK/NAK", "Clock Stretching"],
    },
];
