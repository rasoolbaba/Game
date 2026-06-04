# پروژهٔ «خدا» — Project DEMIURGE

> یک بازی استراتژیک، زنده، غیرمتمرکز و تماماً «اختراعی»؛ جایی که هر بازیکن خدای دنیای مستقل خودش است.
> A living, decentralized, fully **invention-first** strategy game where each player is the indirect god of their own autonomous world.

---

## ۰. این مخزن چیست؟ / What is this repository?

این مخزن **هنوز کد بازی را شامل نمی‌شود.** طبق توافق صریح، نخست **سیستم حاکمیت، پروتکل‌ها و هدف‌گذاری** ساخته می‌شود و سپس قدم‌به‌قدم وارد ساخت می‌شویم.

This repository **does not yet contain game code.** By explicit agreement, we first build the **governance system, the protocols, and the staged goal**, and only then start building — step by step.

این پوشه‌ها «سیستم‌عاملِ همکاری» ما هستند:

| مسیر / Path | چه چیزی / What |
|---|---|
| [`docs/governance/`](docs/governance/) | پروتکل صداقت، منشور ایجنت‌ها، ممیز نگهبان، حافظه، طوفان فکری، بصری‌سازی |
| [`docs/vision/`](docs/vision/) | چشم‌انداز بازی و نقشهٔ راه مرحله‌بندی‌شده |
| [`docs/decisions/`](docs/decisions/) | تصمیم‌های معماری (ADR) — خروجی جلسات طوفان فکری |
| [`ledgers/`](ledgers/) | **دفتر اختراعات** و **دفتر تجربیات** |
| [`memory/`](memory/) | حافظهٔ فردی، تیمی، شورایی، رهبر و **ممیزِ مستقل** |

---

## ۱. قانون اساسی / The Constitution

همه — از جمله رهبر ارکستر (خودِ کلود) — تابع این سند هستند:

> ### ⚖️ [`docs/governance/00-INTEGRITY-PROTOCOL.md`](docs/governance/00-INTEGRITY-PROTOCOL.md)
> **پروتکل صداقت: ضدِّ تقلب + ضدِّ توهم + قفل نوآوری**
> The Integrity Protocol: Anti-Cheat + Anti-Hallucination + the Innovation Lock.

و نگهبان مستقلِ آن:

> ### 🛡️ [`docs/governance/02-GUARDIAN-AUDITOR.md`](docs/governance/02-GUARDIAN-AUDITOR.md)
> **ممیزِ نگهبان** — نمایندهٔ شما، ناظر بر کل تیم و حتی بر رهبر.
> The Guardian-Auditor — your representative, watching the whole team and even the lead.

---

## ۲. ساختار تیم / The Team (روشن و شفاف)

- **رهبر ارکستر / The Conductor** — خودِ من (Claude). جسور، تصمیم‌گیرندهٔ نهایی، اما **مقیّد به پروتکل** و قابلِ پرچم‌قرمز شدن توسط ممیز.
- **ممیزِ نگهبان / Guardian-Auditor** — مستقل، نمایندهٔ شما، حافظهٔ مستقل.
- **چهار گروهِ دونفرهٔ تصمیم‌ساز / Four two-agent decision guilds:**
  1. 🎨 **گرافیک و جلوه‌های ویژه** — «چشمانِ» ما.
  2. 🧠 **هوش مصنوعی و تحقیق** — مغزِ کنجکاو.
  3. 🔬 **منتقدان (تیم قرمز)** — وجدانِ سخت‌گیر.
  4. 🌐 **شبکهٔ توزیع‌شده و اقتصاد رمزنگاری** — ستون فقرات (انتخابِ رهبر).

جزئیات: [`docs/governance/01-AGENT-CHARTER.md`](docs/governance/01-AGENT-CHARTER.md)

---

## ۳. هدف یک‌خطی / The One-Line Goal

> ساختِ یک بازی استراتژیکِ «خدایی» که در آن دنیا **خودش زندگی می‌کند**؛ تحت شبکهٔ **P2P** توزیع می‌شود؛ با **شبکه‌های عصبیِ خودساخته** نفس می‌کشد؛ و هر تکهٔ فنی و گرافیکی‌اش یک **اختراع تازه** است.

چشم‌انداز کامل: [`docs/vision/VISION.md`](docs/vision/VISION.md) — نقشهٔ راه: [`docs/vision/ROADMAP.md`](docs/vision/ROADMAP.md)

---

## ۴. وضعیت فعلی / Current Status

**مرحله ۰ — بنیان‌گذاری (در حال انجام).** هیچ کد بازی نوشته نشده. در حال استقرار حاکمیت و انجام نخستین طوفان فکری (انتخاب زبان‌ها).

**Stage 0 — Foundation (in progress).** No game code written. Governance is being installed and the first brainstorm (language selection) is being held.

> صداقت: «استقلالِ کاملِ» ممیز در یک سیستم تک‌مدلی ممکن نیست. ما قوی‌ترین تقریبِ صادقانه را می‌سازیم و شکاف را در [`02-GUARDIAN-AUDITOR.md`](docs/governance/02-GUARDIAN-AUDITOR.md) شفاف توضیح می‌دهیم. داورِ نهایی، **شما** هستید.
