# الرؤية — Autocrate (Antimony Labs) ضمن S3M2P

**Languages:** [English](VISION.md) | [हिन्दी](VISION.hi.md) | [中文](VISION.zh.md) | [Español](VISION.es.md) | [العربية](VISION.ar.md)

## ماذا نبني

**Autocrate** هو "compiler" موجّه بالمعايير للمنتجات المادية: يحوّل النية المهيكلة إلى **مخرجات تصنيع حتمية**.

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
- STEP هو **output engine** وليس "model".
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

هذا الفصل هو "طبقة الثقة" للذكاء الاصطناعي:
- يمكن للذكاء الاصطناعي اقتراح تغييرات، لكن **DNA tests + deterministic outputs** تلتقط الانحدارات.
- تتراكم التحسينات لأن الـ core يبقى نظيفًا وقابلًا لإعادة الاستخدام.

## نطاق المعايير (v1)

نقوم بنمذجة المعايير كـ **parameterized profiles** (rules + limits)، وليس كنص منسوخ.

- **ASTM D6039**: crate profile selection و rule-driven sizing (ضمن نطاق v1).
- **ASTM D6199**: wood member class مُلتقط كمدخلات مادة/جودة.
- **ISPM 15**: export compliance metadata + required marking/decals كأجزاء.

## كيف يبدو "done"

- يستورد NX ملف STEP المُولّد كـ **assembly** على مقياس **inch** الصحيح وبأسماء حتمية.
- يعرض Viewer نفس الـ assembly ويدعم فحص الأجزاء (IDs, category, metadata).
- تتطابق BOM/Cut List مع design graph وتبقى متسقة مع STEP.

## خارطة الطريق (الخطوات التالية)

بعد Autocrate Lite v1:

- **Richer PMI / MBD**: property sets و IDs و downstream-friendly naming conventions.
- **Rule profiles**: shipping severity knobs، المزيد من standards profiles، و material availability constraints.
- **Catalog parts**: fasteners/connectors كعناصر مكتبة parametric (visual + STEP semantics).
- **More products**: إعادة استخدام DNA/CORE لغير crates من "physical compilers".
