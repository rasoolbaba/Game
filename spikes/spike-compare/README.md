# SPIKE-COMPARE — Rust vs AssemblyScript vs pure-JS (determinism + performance)

> 🧪 **کدِ آزمایشیِ دورریختنی** (Stage-0 spike) — **کدِ بازی نیست.** هدف: تصمیمِ پشته (`ADR-0001`) با شاهدِ واقعی، نه حدس.

## چرا؟ / Why
حاکم در دروازهٔ تصویب، گزینهٔ **«اول SPIKE-COMPARE»** را انتخاب کرد: پیش از تثبیتِ Rust، آن را با گزینه‌های ساده‌تر با شاهد بسنجیم. این، همان پرچم‌قرمزِ «کامل‌نبودنِ» تیمِ قرمز بود.

## چه چیزی آزموده شد / What
یک هستهٔ **یکسانِ** عصبیِ fixed-pointِ صحیح، در چهار پیاده‌سازی، با مقایسهٔ:
1. **قطعیت** — آیا اثرانگشتِ خروجی بیت‌به‌بیت یکسان است؟
2. **کارایی** — تیک بر ثانیه.
3. **اندازهٔ wasm** و ملاحظاتِ کیفی.

## نتیجه / Result
👉 همه‌چیز در [`RESULTS.md`](RESULTS.md). خلاصه: **قطعیت در هر چهار برقرار است؛ Rust برای کارایی برنده است** (نه چون «تنها Rust قطعی است»).

## ساختار / Layout
```
gen.mjs            تولیدِ LUT مشترک (js/rs/ts)
js/sim.mjs         پیاده‌سازیِ JS (BigInt)
rust/sim.rs        Rust نیتیو   ·  rust/sim_wasm.rs + run_wasm.mjs  Rust→wasm
as/sim.ts          AssemblyScript→wasm  ·  as/run.mjs لودر
run-all.sh         اجرای هر چهار + assert یکسانیِ اثرانگشت
RESULTS.md         شواهد + یافته‌ها + هشدارها + بازتولید
```
بازتولید: `node gen.mjs && ./run-all.sh 1000000` (پس از build؛ راهنما در RESULTS.md §۷).
