/*!
 * Comprehensive benchmark comparing skey-engine vs vime-engine.
 *
 * Tests both engines with identical inputs across multiple scenarios:
 * - Single character transforms
 * - Short word composition
 * - Long text/sentence processing
 * - Incremental typing simulation (real-world IME usage)
 * - Rapid keystroke accumulation
 * - VNI input method
 * - Charset conversion (TCVN3, VNIWin)
 * - TeipVni (combined Telex+VNI)
 */

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// Import both engines
use skey_engine as skey;
use vime_engine::composition::Composition;
use vime_engine::phonology::TonePlacement;
use vime_engine::DefaultKeymap;

// ============================================================================
// Test Data
// ============================================================================

const SINGLE_CHARS: &[&str] = &["a", "o", "u", "e", "i", "y"];

const SHORT_WORDS: &[&str] = &[
    "tooi",       // simple tone + shape
    "tieengs",    // multiple tones/shapes
    "nguwowif",   // complex uo sequence
    "dduwowcj",   // d-stroke + uo + coda
    "chao",       // onset cluster
    "nguyen",     // complex onset + coda
    "toan",       // diphthong + coda
    "kien",       // simple diphthong
];

const SENTENCES: &[&str] = &[
    "tooi ddeens tuwf vuofn cuar mej tooi, maf toi laf con trai duy nhaats",
    "nguyen van a di hoc o truong dai hoc quoc gia ha noi",
    "tooi laf ngwowoi viet nam, toi thich an pho va bun cha",
];

const INCREMENTAL_SEQUENCES: &[&[&str]] = &[
    // "xin chào"
    &["x", "xi", "xin", "xin ", "xin c", "xin ch", "xin cha", "xin chaf", "xin chafo"],
    // "tiếng Việt"
    &["t", "ti", "tie", "tiee", "tieen", "tieeng", "tieengs",
      "tieengs ", "tieengs V", "tieengs Vi", "tieengs Vie", "tieengs Viee", "tieengs Viej", "tieengs Vieejt"],
    // "được"
    &["d", "dd", "ddu", "dduw", "dduwo", "dduwow", "dduwowc", "dduwowcj"],
    // "người"
    &["n", "ng", "ngu", "nguw", "nguwo", "nguwow", "nguwowi", "nguwowif"],
    // "toàn" - tone reassignment
    &["t", "to", "tof", "tofa", "tofan"],
];

const VNI_WORDS: &[&str] = &[
    "tie61ng",
    "vie65t",
    "tie61ng2",
];

const CHARS_TEXT: &str = "tiếng Việt là ngôn ngữ của người Việt Nam, được viết bằng chữ Latinh với các dấu thanh điệu";

const RAPID_CHARS: &[&str] = &[
    "t", "ti", "tie", "tiee", "tieen", "tieeng", "tieengs",
    "tieengs ", "tieengs v", "tieengs vi", "tieengs vie", "tieengs viee", "tieengsziej", "tieengsziej",
    "tieengsziej d", "tieengsziej dd", "tieengsziej ddu", "tieengsziej dduw", "tieengsziej dduwo",
    "tieengsziej dduwow", "tieengsziej dduwowc", "tieengsziej dduwowcj",
    "tieengsziej dduwowcj n", "tieengsziej dduwowcj ng", "tieengsziej dduwowcj ngu", "tieengsziej dduwowcj nguw",
    "tieengsziej dduwowcj nguwo", "tieengsziej dduwowcj nguwow", "tieengsziej dduwowcj nguwowi", "tieengsziej dduwowcj nguwowif",
];

// ============================================================================
// Helper Functions - skey-engine
// ============================================================================

/// Run skey-engine Telex conversion (full string)
fn skey_telex(input: &str) -> String {
    skey::engine::convert_telex(input, true, false)
}

/// Run skey-engine Telex conversion (incremental mode)
fn skey_telex_inc(input: &str) -> String {
    skey::engine::convert_telex(input, false, true)
}

/// Run skey-engine VNI conversion
fn skey_vni(input: &str) -> String {
    skey::engine::convert_vni(input, false, true)
}

/// Run skey-engine TeipVni conversion
fn skey_teipvni(input: &str) -> String {
    skey::engine::convert_teip_vni(input, false, true)
}

/// Run skey-engine charset encoding
fn skey_charset_encode(text: &str, charset: skey::charset::VietCharset) -> Vec<u8> {
    skey::charset::encode(text, charset)
}

/// Run skey-engine charset decoding
fn skey_charset_decode(data: &[u8], charset: skey::charset::VietCharset) -> String {
    skey::charset::decode(data, charset)
}

/// Run skey-engine tone removal
fn skey_remove_tone(text: &str) -> String {
    skey::charset::remove_tone(text)
}

// ============================================================================
// Helper Functions - vime-engine (Composition API)
// ============================================================================

/// Run vime-engine via Composition API (full string)
fn vime_composition(input: &str) -> String {
    let mut comp = Composition::new();
    let keymap = DefaultKeymap::telex();
    let tone = TonePlacement::Modern;
    for ch in input.chars() {
        comp.insert(&keymap, tone, ch);
    }
    comp.rendered(tone).iter().collect()
}

/// Run vime-engine via Composition API (incremental)
fn vime_composition_inc(input: &str) -> String {
    let mut comp = Composition::new();
    let keymap = DefaultKeymap::telex();
    let tone = TonePlacement::Modern;
    for ch in input.chars() {
        comp.insert(&keymap, tone, ch);
    }
    comp.rendered(tone).iter().collect()
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_single_char(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_char");
    
    for &input in SINGLE_CHARS {
        group.bench_function(BenchmarkId::new("skey_telex", input), |b| {
            b.iter(|| skey_telex(black_box(input)));
        });
        
        group.bench_function(BenchmarkId::new("vime_composition", input), |b| {
            b.iter(|| vime_composition(black_box(input)));
        });
    }
    group.finish();
}

fn bench_short_words(c: &mut Criterion) {
    let mut group = c.benchmark_group("short_words");
    
    for &input in SHORT_WORDS {
        group.bench_function(BenchmarkId::new("skey_telex", input), |b| {
            b.iter(|| skey_telex(black_box(input)));
        });
        
        group.bench_function(BenchmarkId::new("vime_composition", input), |b| {
            b.iter(|| vime_composition(black_box(input)));
        });
    }
    group.finish();
}

fn bench_long_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("long_text");
    
    for (i, &input) in SENTENCES.iter().enumerate() {
        let label = format!("sentence_{}", i);
        
        group.bench_function(BenchmarkId::new("skey_telex", label.clone()), |b| {
            b.iter(|| skey_telex(black_box(input)));
        });
        
        group.bench_function(BenchmarkId::new("vime_composition", label.clone()), |b| {
            b.iter(|| vime_composition(black_box(input)));
        });
    }
    group.finish();
}

fn bench_incremental_typing(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_typing");
    group.measurement_time(Duration::from_secs(10));
    
    for (seq_idx, seq) in INCREMENTAL_SEQUENCES.iter().enumerate() {
        let label = format!("seq_{}", seq_idx);
        
        // skey-engine incremental
        group.bench_function(BenchmarkId::new("skey_telex_inc", label.clone()), |b| {
            b.iter(|| {
                for input in *seq {
                    let _ = skey_telex_inc(black_box(input));
                }
            });
        });
        
        // vime-engine Composition incremental
        group.bench_function(BenchmarkId::new("vime_composition_inc", label.clone()), |b| {
            b.iter(|| {
                let mut comp = Composition::new();
                let keymap = DefaultKeymap::telex();
                let tone = TonePlacement::Modern;
                for input in *seq {
                    comp.insert(&keymap, tone, input.chars().last().unwrap());
                }
            });
        });
    }
    group.finish();
}

fn bench_rapid_accumulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rapid_accumulation");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("skey_telex", |b| {
        b.iter(|| {
            for s in RAPID_CHARS {
                let _ = skey_telex(black_box(s));
            }
        });
    });
    
    group.bench_function("vime_composition", |b| {
        b.iter(|| {
            let mut comp = Composition::new();
            let keymap = DefaultKeymap::telex();
            let tone = TonePlacement::Modern;
            for s in RAPID_CHARS {
                for ch in s.chars().last() {
                    comp.insert(&keymap, tone, ch);
                }
            }
        });
    });
    group.finish();
}

fn bench_vni(c: &mut Criterion) {
    let mut group = c.benchmark_group("vni");
    
    for &input in VNI_WORDS {
        group.bench_function(BenchmarkId::new("skey_vni", input), |b| {
            b.iter(|| skey_vni(black_box(input)));
        });
        
        group.bench_function(BenchmarkId::new("vime_composition_vni", input), |b| {
            b.iter(|| vime_composition(black_box(input)));
        });
    }
    
    group.finish();
}

fn bench_teipvni(c: &mut Criterion) {
    let mut group = c.benchmark_group("teipvni");
    
    for &input in VNI_WORDS {
        group.bench_function(BenchmarkId::new("skey_teipvni", input), |b| {
            b.iter(|| skey_teipvni(black_box(input)));
        });
    }
    group.finish();
}

fn bench_charset(c: &mut Criterion) {
    let mut group = c.benchmark_group("charset");
    
    // TCVN3
    group.bench_function("skey_encode_tcvn3", |b| {
        b.iter(|| skey_charset_encode(black_box(CHARS_TEXT), skey::charset::VietCharset::TCVN3));
    });
    
    let tcvn3_encoded = skey_charset_encode(CHARS_TEXT, skey::charset::VietCharset::TCVN3);
    group.bench_function("skey_decode_tcvn3", |b| {
        b.iter(|| skey_charset_decode(black_box(&tcvn3_encoded), skey::charset::VietCharset::TCVN3));
    });
    
    // VNIWin
    group.bench_function("skey_encode_vniwin", |b| {
        b.iter(|| skey_charset_encode(black_box(CHARS_TEXT), skey::charset::VietCharset::VNIWin));
    });
    
    let vniwin_encoded = skey_charset_encode(CHARS_TEXT, skey::charset::VietCharset::VNIWin);
    group.bench_function("skey_decode_vniwin", |b| {
        b.iter(|| skey_charset_decode(black_box(&vniwin_encoded), skey::charset::VietCharset::VNIWin));
    });
    
    // Tone removal
    group.bench_function("skey_remove_tone", |b| {
        b.iter(|| skey_remove_tone(black_box(CHARS_TEXT)));
    });
    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    targets = bench_single_char,
    bench_short_words,
    bench_long_text,
    bench_incremental_typing,
    bench_rapid_accumulation,
    bench_vni,
    bench_teipvni,
    bench_charset
);
criterion_main!(benches);