# 🧠 حافظهٔ پروژه / Project Memory

ساختار و قواعد در [`../docs/governance/03-MEMORY-ARCHITECTURE.md`](../docs/governance/03-MEMORY-ARCHITECTURE.md).

پنج لایه: **فردی** (هر ایجنت) · **تیمی** (هر گروه) · **شورا** (تصمیم‌های میان‌گروهی) · **رهبر** · **ممیز** (مستقل).

قاعدهٔ طلایی: **فقط-افزودنی.** هرگز ورودیِ قبلی را پاک نکن؛ تصحیح = ورودیِ جدید با ارجاع.

```
memory/
├── orchestrator/LEAD-LOG.md       رهبر
├── auditor/AUDIT-LOG.md           ممیز (مستقل، کسی بازنویسی‌اش نمی‌کند)
├── council/COUNCIL-LOG.md         شورا (مبنای تصمیمِ رهبر)
└── teams/<team>/
    ├── TEAM-LOG.md                حافظهٔ تیمی
    └── members/<agent>.md         حافظهٔ فردی
```
تیم‌ها: `graphics-vfx` · `ai-research` · `red-team-critics` · `netecon`
