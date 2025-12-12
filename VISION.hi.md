# विज़न — Autocrate (Antimony Labs) in S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## हम क्या बना रहे हैं

**Autocrate** भौतिक उत्पादों के लिए एक standards-driven "compiler" है: यह structured intent को **deterministic manufacturing artifacts** में बदल देता है।

v1 में, intent एक **crate specification** (dimensions, weight, shipping mode, compliance profile) है। Artifacts:

- **STEP assembly (NX-importable, inches)**: एक single MBD-first deliverable, जो manufacturing के लिए पर्याप्त है।
- **BOM (CSV)**: क्या खरीदना है।
- **Cut List (CSV)**: क्या काटना है।
- **Viewer scene**: क्या देखना है (platform moat)।

Contract:

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## यह क्यों महत्वपूर्ण है (MBD-first)

CAD का भविष्य **Model-Based Definition (MBD)** है: एक single STEP file (AP242-style) assembly structure और downstream के लिए जरूरी information carry कर सकता है।

Autocrate यह stance अपनाता है:
- STEP एक **output engine** है, "model" नहीं।
- Canonical model हमारा **CrateDesign** graph है।
- STEP/BOM/Cut List उसी graph से generate होते हैं, इसलिए वे consistent रहते हैं।

## moat: visualization

अधिकांश systems CAD exchange files को source of truth मानते हैं और visualization के लिए उन्हें re-parse करने की कोशिश करते हैं।

हम उल्टा करते हैं:
- **CrateDesign** ही truth है।
- हम STEP parse किए बिना **CrateDesign से सीधे render** करते हैं।

इससे visualization:
- **Faster** (heavy CAD import pipeline नहीं),
- **More controllable** (semantic parts + metadata),
- **More defensible** (design graph + rendering engine साथ में)।

## S3M2P architecture advantage (DNA → CORE → TOOLS)

S3M2P को जानबूझकर layers में बनाया गया है ताकि AI-assisted iteration safer हो:

- **DNA**: pure algorithms, physics/math/data structures. Deterministic, testable, reusable.
- **CORE**: ergonomic engines जो tools के लिए stable APIs expose करते हैं।
- **TOOLS**: user-facing apps (WASM) जो render और export करते हैं।

यह separation AI के लिए "trust layer" है:
- AI changes propose कर सकता है, लेकिन **DNA tests + deterministic outputs** regressions पकड़ लेते हैं।
- Improvements compound होते हैं क्योंकि core clean और reusable रहता है।

## Standards scope (v1)

हम standards को copied text की तरह नहीं, बल्कि **parameterized profiles** (rules + limits) के रूप में model करते हैं।

- **ASTM D6039**: crate profile selection और rule-driven sizing (v1 scope)।
- **ASTM D6199**: wood member class को material/quality inputs के रूप में capture करना।
- **ISPM 15**: export compliance metadata + required marking/decals को parts के रूप में।

## "done" कैसा दिखता है

- NX generated STEP को सही **inch scale** पर deterministic names के साथ एक **assembly** के रूप में import करता है।
- Viewer वही assembly दिखाता है और part inspection support करता है (IDs, category, metadata)।
- BOM/Cut List design graph से match करते हैं और STEP के साथ consistent रहते हैं।

## Roadmap (next wedges)

Autocrate Lite v1 के बाद:

- **Richer PMI / MBD**: property sets, IDs, और downstream-friendly naming conventions।
- **Rule profiles**: shipping severity knobs, और अधिक standards profiles, material availability constraints।
- **Catalog parts**: fasteners/connectors को parametric library items के रूप में (visual + STEP semantics)।
- **More products**: crates के अलावा अन्य "physical compilers" के लिए DNA/CORE reuse।
