# Claude Prompt — Add multilingual Vision (Hindi / Chinese / Spanish / Arabic)

You are **Claude (programmer)** working in the repo at `S3M2P/`.

## Goal
Add **language support** for the Vision doc by shipping translated versions of `VISION.md`:
- Hindi (`hi`)
- Chinese, Simplified (`zh`)
- Spanish (`es`)
- Arabic (`ar`, RTL)

“Language support” here means:
- Separate translated markdown files
- A **language switcher** at the top of each file linking between them

## Non‑negotiables
- **Do not change the meaning** of the English vision.
- **Keep technical tokens unchanged** everywhere:
  - `Autocrate`, `Antimony Labs`, `S3M2P`, `CrateSpec`, `CrateDesign`, `STEP`, `NX`, `BOM`, `CSV`, `Cut List`, `Viewer`, `MBD`, `AP242`, `PMI`, `ASTM D6039`, `ASTM D6199`, `ISPM 15`
- Keep the contract line **exactly**:
  - `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`
- Keep headings/section structure aligned across languages.

## Files to modify / add
- **Modify**: `VISION.md`
- **Add**:
  - `VISION.hi.md`
  - `VISION.zh.md`
  - `VISION.es.md`
  - `VISION.ar.md`

## Task 1 — Add language switcher to the English doc
In `VISION.md`, insert this line **immediately after the title line**:

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

Do not change anything else in `VISION.md`.

## Task 2 — Create translated Vision files
Create the following files with the exact contents below.

### `VISION.hi.md` (Hindi)
```md
# विज़न — Autocrate (Antimony Labs) in S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## हम क्या बना रहे हैं

**Autocrate** भौतिक उत्पादों के लिए एक standards-driven “compiler” है: यह structured intent को **deterministic manufacturing artifacts** में बदल देता है।

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
- STEP एक **output engine** है, “model” नहीं।
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

यह separation AI के लिए “trust layer” है:
- AI changes propose कर सकता है, लेकिन **DNA tests + deterministic outputs** regressions पकड़ लेते हैं।
- Improvements compound होते हैं क्योंकि core clean और reusable रहता है।

## Standards scope (v1)

हम standards को copied text की तरह नहीं, बल्कि **parameterized profiles** (rules + limits) के रूप में model करते हैं।

- **ASTM D6039**: crate profile selection और rule-driven sizing (v1 scope)।
- **ASTM D6199**: wood member class को material/quality inputs के रूप में capture करना।
- **ISPM 15**: export compliance metadata + required marking/decals को parts के रूप में।

## “done” कैसा दिखता है

- NX generated STEP को सही **inch scale** पर deterministic names के साथ एक **assembly** के रूप में import करता है।
- Viewer वही assembly दिखाता है और part inspection support करता है (IDs, category, metadata)।
- BOM/Cut List design graph से match करते हैं और STEP के साथ consistent रहते हैं।

## Roadmap (next wedges)

Autocrate Lite v1 के बाद:

- **Richer PMI / MBD**: property sets, IDs, और downstream-friendly naming conventions।
- **Rule profiles**: shipping severity knobs, और अधिक standards profiles, material availability constraints।
- **Catalog parts**: fasteners/connectors को parametric library items के रूप में (visual + STEP semantics)।
- **More products**: crates के अलावा अन्य “physical compilers” के लिए DNA/CORE reuse।
```

### `VISION.zh.md` (Chinese, Simplified)
```md
# 愿景 — Autocrate（Antimony Labs）在 S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## 我们在构建什么

**Autocrate** 是一个以标准为驱动的“编译器”，面向实体产品：它把结构化意图转化为**确定性的制造产物**。

在 v1 中，意图是一份 **crate 规格**（尺寸、重量、运输方式、合规配置）。产物包括：

- **STEP 装配体（可导入 NX，英寸制）**：一个以 MBD 为先的单一交付物，足以用于制造。
- **BOM（CSV）**：需要采购什么。
- **Cut List（CSV）**：需要切割什么。
- **Viewer 场景**：需要看到什么（平台护城河）。

契约是：

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## 为什么重要（MBD 优先）

CAD 的未来是 **基于模型的定义（MBD）**：一个单一的 STEP 文件（AP242 风格）即可携带装配结构以及下游所需的信息。

Autocrate 采用这种立场：
- STEP 是一个**输出引擎**，而不是“模型”。
- 我们的权威模型是 **CrateDesign** 图。
- STEP/BOM/Cut List 都从同一张图生成，因此保持一致。

## 护城河：可视化

大多数系统把 CAD 交换文件当作真相来源，然后为了可视化再去重解析它们。

我们反其道而行：
- **CrateDesign** 才是事实。
- 我们直接从 **CrateDesign** 渲染，而不解析 STEP。

这让可视化变得：
- **更快**（不需要沉重的 CAD 导入流水线），
- **更可控**（语义化部件 + 元数据），
- **更具防御性**（设计图 + 渲染引擎一体构建）。

## S3M2P 的架构优势（DNA → CORE → TOOLS）

S3M2P 有意分层，让 AI 辅助迭代更安全：

- **DNA**：纯算法、物理/数学/数据结构。确定性、可测试、可复用。
- **CORE**：易用的引擎，对外提供稳定 API 供工具调用。
- **TOOLS**：面向用户的应用（WASM），负责渲染与导出。

这种分层是 AI 的“信任层”：
- AI 可以提出修改，但 **DNA 测试 + 确定性输出** 会捕捉回归。
- 因为核心保持干净且可复用，改进会持续叠加。

## 标准范围（v1）

我们将标准建模为**参数化配置**（规则 + 限制），而不是复制粘贴文本。

- **ASTM D6039**：crate 配置选择与规则驱动的尺寸计算（v1 范围）。
- **ASTM D6199**：木材构件等级作为材料/质量输入捕获。
- **ISPM 15**：出口合规元数据 + 必需标识/贴花作为部件。

## “完成”的样子

- NX 能以正确的**英寸**比例将生成的 STEP 作为**装配体**导入，并且命名确定。
- Viewer 展示同一装配体，并支持部件检查（ID、类别、元数据）。
- BOM/Cut List 与设计图一致，并与 STEP 保持一致。

## 路线图（下一阶段楔子）

在 Autocrate Lite v1 之后：

- **更丰富的 PMI / MBD**：属性集、ID，以及更利于下游的命名约定。
- **规则配置**：运输严苛度调节，更多标准配置，材料可用性约束。
- **目录件**：紧固件/连接件作为参数化库条目（可视化 + STEP 语义）。
- **更多产品**：复用 DNA/CORE，拓展到 crates 之外的其它“physical compilers”。
```

### `VISION.es.md` (Spanish)
```md
# VISIÓN — Autocrate (Antimony Labs) en S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## Qué estamos construyendo

**Autocrate** es un “compilador” guiado por estándares para productos físicos: convierte una intención estructurada en **artefactos de fabricación deterministas**.

En v1, la intención es una **crate specification** (dimensiones, peso, modo de envío, perfil de cumplimiento). Los artefactos son:

- **STEP assembly (NX-importable, inches)**: un único entregable MBD-first suficiente para fabricar.
- **BOM (CSV)**: qué comprar.
- **Cut List (CSV)**: qué cortar.
- **Viewer scene**: qué ver (la fosa defensiva de la plataforma).

El contrato es:

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## Por qué importa (MBD-first)

El futuro del CAD es la **Model-Based Definition (MBD)**: un único archivo STEP (estilo AP242) puede contener la estructura del ensamblaje y la información necesaria aguas abajo.

Autocrate adopta esta postura:
- STEP es un **output engine**, no el “model”.
- El modelo canónico es nuestro grafo **CrateDesign**.
- STEP/BOM/Cut List se generan a partir del mismo grafo, por lo que se mantienen consistentes.

## La fosa defensiva: visualización

La mayoría de los sistemas tratan los archivos de intercambio CAD como fuente de verdad e intentan re-parsearlos para la visualización.

Nosotros hacemos lo contrario:
- **CrateDesign** es la verdad.
- Renderizamos **directamente desde CrateDesign**, sin parsear STEP.

Esto hace que la visualización sea:
- **Más rápida** (sin un pipeline pesado de importación CAD),
- **Más controlable** (piezas semánticas + metadatos),
- **Más defendible** (un grafo de diseño + motor de render construidos juntos).

## La ventaja de arquitectura de S3M2P (DNA → CORE → TOOLS)

S3M2P está intencionalmente estratificado para que la iteración asistida por IA sea más segura:

- **DNA**: algoritmos puros, física/matemática/estructuras de datos. Determinista, testeable, reutilizable.
- **CORE**: motores ergonómicos que exponen APIs estables para tools.
- **TOOLS**: apps orientadas al usuario (WASM) que renderizan y exportan.

Esta separación es la “capa de confianza” para la IA:
- La IA puede proponer cambios, pero las **DNA tests + deterministic outputs** detectan regresiones.
- Las mejoras se acumulan porque el core permanece limpio y reutilizable.

## Alcance de estándares (v1)

Modelamos los estándares como **parameterized profiles** (rules + limits), no como texto copiado.

- **ASTM D6039**: crate profile selection y rule-driven sizing (alcance v1).
- **ASTM D6199**: wood member class capturada como entradas de material/calidad.
- **ISPM 15**: export compliance metadata + required marking/decals como parts.

## Qué significa “done”

- NX importa el STEP generado como un **assembly** a la escala correcta en **inch** con nombres deterministas.
- Viewer muestra el mismo assembly y permite inspección de parts (IDs, category, metadata).
- BOM/Cut List coinciden con el design graph y se mantienen consistentes con STEP.

## Hoja de ruta (próximas cuñas)

Después de Autocrate Lite v1:

- **Richer PMI / MBD**: property sets, IDs y downstream-friendly naming conventions.
- **Rule profiles**: shipping severity knobs, más standards profiles, material availability constraints.
- **Catalog parts**: fasteners/connectors como parametric library items (visual + STEP semantics).
- **More products**: reutilizar DNA/CORE para otros “physical compilers” más allá de crates.
```

### `VISION.ar.md` (Arabic)
```md
# الرؤية — Autocrate (Antimony Labs) ضمن S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## ماذا نبني

**Autocrate** هو “compiler” موجّه بالمعايير للمنتجات المادية: يحوّل النية المهيكلة إلى **مخرجات تصنيع حتمية**.

في v1، تكون النية هي **crate specification** (الأبعاد، الوزن، نمط الشحن، ملف الامتثال). والمخرجات هي:

- **STEP assembly (NX-importable, inches)**: تسليم واحد بنهج MBD-first يكفي للتصنيع.
- **BOM (CSV)**: ماذا نشتري.
- **Cut List (CSV)**: ماذا نقطع.
- **Viewer scene**: ماذا نرى (الميزة الدفاعية للمنصة).

العقد هو:

> `CrateSpec → CrateDesign → { STEP, BOM, Cut List, Viewer }`

## لماذا يهم هذا (MBD-first)

مستقبل CAD هو **Model-Based Definition (MBD)**: يمكن لملف STEP واحد (على نمط AP242) أن يحمل بنية التجميع والمعلومات المطلوبة لاحقًا.

يتبنى Autocrate هذا الموقف:
- STEP هو **output engine** وليس “model”.
- النموذج المرجعي هو مخطط **CrateDesign** لدينا.
- يتم توليد STEP/BOM/Cut List من نفس المخطط، لذا تبقى متسقة.

## الميزة الدفاعية: التصوير/العرض المرئي

تتعامل معظم الأنظمة مع ملفات تبادل CAD كمصدر للحقيقة ثم تحاول إعادة تحليلها من أجل العرض المرئي.

نحن نفعل العكس:
- **CrateDesign** هو الحقيقة.
- نعرض **مباشرة من CrateDesign** دون تحليل STEP.

هذا يجعل العرض المرئي:
- **أسرع** (دون خط استيراد CAD ثقيل)،
- **أكثر قابلية للتحكم** (أجزاء دلالية + بيانات وصفية)،
- **أكثر دفاعًا** (مخطط تصميم + محرك عرض مبنيان معًا).

## ميزة معمارية S3M2P (DNA → CORE → TOOLS)

تم تصميم S3M2P على طبقات عمدًا لجعل التكرار بمساعدة الذكاء الاصطناعي أكثر أمانًا:

- **DNA**: خوارزميات خالصة، فيزياء/رياضيات/هياكل بيانات. حتمية، قابلة للاختبار، قابلة لإعادة الاستخدام.
- **CORE**: محركات مريحة تُعرّض APIs مستقرة للأدوات.
- **TOOLS**: تطبيقات موجهة للمستخدم (WASM) للعرض والتصدير.

هذا الفصل هو “طبقة الثقة” للذكاء الاصطناعي:
- يمكن للذكاء الاصطناعي اقتراح تغييرات، لكن **DNA tests + deterministic outputs** تلتقط الانحدارات.
- تتراكم التحسينات لأن الـ core يبقى نظيفًا وقابلًا لإعادة الاستخدام.

## نطاق المعايير (v1)

نقوم بنمذجة المعايير كـ **parameterized profiles** (rules + limits)، وليس كنص منسوخ.

- **ASTM D6039**: crate profile selection و rule-driven sizing (ضمن نطاق v1).
- **ASTM D6199**: wood member class مُلتقط كمدخلات مادة/جودة.
- **ISPM 15**: export compliance metadata + required marking/decals كأجزاء.

## كيف يبدو “done”

- يستورد NX ملف STEP المُولّد كـ **assembly** على مقياس **inch** الصحيح وبأسماء حتمية.
- يعرض Viewer نفس الـ assembly ويدعم فحص الأجزاء (IDs, category, metadata).
- تتطابق BOM/Cut List مع design graph وتبقى متسقة مع STEP.

## خارطة الطريق (الخطوات التالية)

بعد Autocrate Lite v1:

- **Richer PMI / MBD**: property sets و IDs و downstream-friendly naming conventions.
- **Rule profiles**: shipping severity knobs، المزيد من standards profiles، و material availability constraints.
- **Catalog parts**: fasteners/connectors كعناصر مكتبة parametric (visual + STEP semantics).
- **More products**: إعادة استخدام DNA/CORE لغير crates من “physical compilers”.
```

## Done criteria
- `VISION.md` has the language switcher line added under the title.
- The four new translation files exist and render cleanly on GitHub.
- All five files link to each other correctly.

