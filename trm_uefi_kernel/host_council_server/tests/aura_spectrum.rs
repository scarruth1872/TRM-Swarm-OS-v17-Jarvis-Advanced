//! aura_spectrum — does a wavelength actually identify a concept?
//!
//! Gate, stated before the run (G73):
//!   (a) all 100 concepts get distinct (band, slot) lines for the platform anchor
//!   (b) every concept identifies from its own name -> 100/100 round-trip
//!   (c) a different anchor moves the lines (identity is anchored, not decorative)
//!   (d) unrelated real directive prose does NOT identify a concept (no nearest-neighbour forcing)
//!   (e) lambda = c / f for every line
//!   (f) the optimistic "infinite combos" claim is reported as the finite coordinate count it is

use trm_uefi_kernel::aura_spectrum::{
    concept_count, identify, line_for, nominal_coordinates, spectrum, Cat, CONCEPTS,
    FRACTAL_DH, MIN_SCORE, MONAD_TRUTH_ANCHOR, QUARTER_TONES_PER_BAND, SPEED_OF_LIGHT,
};

#[test]
fn aura_spectrum_measurements() {
    let anchor = MONAD_TRUTH_ANCHOR;
    let lines = spectrum(anchor);
    println!("\n=== AURA concept spectrum (anchor 0x{anchor:016X}) ===");
    println!("catalog concepts      {}", concept_count());
    println!("distinct band_hz      {}", {
        let mut v: Vec<u64> = CONCEPTS.iter().map(|c| c.band_hz as u64).collect();
        v.sort_unstable();
        v.dedup();
        v.len()
    });

    // (a) uniqueness
    let mut keys: Vec<(u64, u16)> = lines.iter().map(|l| (l.band_hz as u64, l.slot)).collect();
    let total = keys.len();
    keys.sort_unstable();
    keys.dedup();
    println!("distinct (band,slot)  {} of {}", keys.len(), total);
    assert_eq!(keys.len(), total, "two concepts share a spectral line");
    assert_eq!(total, 100);

    // (e) lambda = c / f
    for l in &lines {
        let expect = SPEED_OF_LIGHT / l.f_line_hz;
        assert!((l.lambda_m - expect).abs() / expect < 1e-12, "lambda mismatch for {}", l.id);
    }

    // closest pair, to state the resolution limit instead of implying magic
    let mut best = (f64::MAX, "", "");
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            let a = &lines[i];
            let b = &lines[j];
            let d = match a.lambda_m > b.lambda_m {
                true => a.lambda_m - b.lambda_m,
                false => b.lambda_m - a.lambda_m,
            };
            if d < best.0 {
                best = (d, a.id, b.id);
            }
        }
    }
    println!(
        "closest line pair     {} vs {} : {:.3} m apart ({:.4} % of band)",
        best.1,
        best.2,
        best.0,
        100.0 * best.0 / (lines.iter().map(|l| l.lambda_m).fold(0.0, f64::max))
    );

    // (b) round-trip: each concept identifies from its own name
    let mut ok = 0;
    let mut misses: Vec<(&str, Option<&str>)> = Vec::new();
    for c in CONCEPTS.iter() {
        match identify(c.name, anchor) {
            Some(i) if i.id == c.id => ok += 1,
            other => misses.push((c.id, other.map(|i| i.id))),
        }
    }
    println!("name round-trip       {ok}/100");
    for m in &misses {
        println!("   miss: {} -> {:?}", m.0, m.1);
    }

    // (c) anchor sensitivity
    let alt = anchor ^ 0x0000_0000_00FF_FFFF;
    let alt_lines = spectrum(alt);
    let moved = lines
        .iter()
        .zip(alt_lines.iter())
        .filter(|(a, b)| a.slot != b.slot)
        .count();
    println!("lines that move with a changed anchor: {moved}/100");
    assert!(moved >= 90, "anchor barely matters ({moved}/100 moved)");

    // identify() must refuse a non-platform anchor
    let foreign = identify("Quantum Coherence", alt).expect("still identifies");
    println!(
        "foreign-anchor identify: anchored={} confidence={:.3}",
        foreign.anchored, foreign.confidence
    );
    assert!(!foreign.anchored);
    assert_eq!(foreign.confidence, 0.0);

    // (d) negative control on real directive prose
    let corpus = std::fs::read_to_string("../../evidence/BLIND_DIRECTIVES.jsonl").unwrap_or_default();
    let mut checked = 0usize;
    let mut forced: Vec<(String, &str)> = Vec::new();
    for line in corpus.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Parse, do not slice: escaped quotes inside JSON mean a string-slice "text" swallows the
        // following rows, which turned 80 separate directives into one long document and made the
        // negative control report matches that came from other rows' vocabulary.
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let text = match v.get("text").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => continue,
        };
        checked += 1;
        if let Some(i) = identify(&text, anchor) {
            println!("   forced score {:.1} slot {} :: {:?}", i.score, i.slot, text);
            forced.push((text, i.id));
        }
    }
    println!("unrelated directives checked {checked}, forced identifications {}", forced.len());
    for f in forced.iter().take(8) {
        println!("   forced: {:?} -> {}", truncate(&f.0, 60), f.1);
    }

    // fractal coupling: width comes from the stream's measured Higuchi dimension
    println!("\nfractal stream coupling (width in quarter-tones):");
    for (name, dh) in FRACTAL_DH.iter() {
        println!("   {:<26} D_H {:.3} -> width {:.2} qt", name, dh, (dh - 1.0) * 12.0);
    }
    let one = line_for("MonadTruth", anchor).expect("concept present");
    println!(
        "\nsample line  MonadTruth: band {:.0} Hz, slot {}/{}, f_line {:.4} Hz, lambda {:.1} m",
        one.band_hz, one.slot, QUARTER_TONES_PER_BAND, one.f_line_hz, one.lambda_m
    );

    // (f) the honest size of the coordinate space
    println!(
        "\nnominal coordinates   {} (100 concepts x {} slots x {} streams x 3 polarities)",
        nominal_coordinates(),
        QUARTER_TONES_PER_BAND,
        FRACTAL_DH.len()
    );
    println!("realised by one anchor 100");
    println!("acceptance floor MIN_SCORE {MIN_SCORE}");

    // report the partial-credit state rather than asserting it green
    println!("\nGATE: (a) {} (b) {}/100 (c) {}/100 moved (d) {} forced of {}",
        if keys.len() == total { "PASS" } else { "FAIL" },
        ok, moved,
        forced.len(), checked);
    assert_eq!(keys.len(), total);
    assert_eq!(ok, 100, "round-trip incomplete — listed above");
    assert!(forced.is_empty(), "a concept was forced onto unrelated prose");
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    s.chars().take(n).collect::<String>() + "..."
}

/// Every category must resolve to a real catalogue entry, so the spectrum cannot silently drop one.
#[test]
fn every_category_is_representable() {
    let cats = [
        Cat::Foundation, Cat::Consciousness, Cat::Physics, Cat::Biology, Cat::Geometry,
        Cat::Transmutation, Cat::Cosmos, Cat::Computing, Cat::Architecture, Cat::Soul,
    ];
    for c in cats {
        assert!(
            CONCEPTS.iter().any(|k| k.category == c),
            "no concept for {}",
            c.as_str()
        );
    }
}

/// The DSL binds concept lines, and the field is a function of the input rather than an accumulator.
#[test]
fn aura_dsl_binds_lines_and_field_is_content_derived() {
    use trm_uefi_kernel::aura_spectrum::{evaluate, Op};
    let a = MONAD_TRUTH_ANCHOR;

    let script = "MonadTruth ~> Coherence\nQuantum Coherence ~|~ Harmonic Resonance\n@ledger_flush";
    let ev = evaluate(script, a);
    println!("\n=== AURA DSL ===");
    for st in &ev.statements {
        println!(
            "  {} concepts {:?} lines {} verdict {:+}",
            format!("{:?}", st.op), st.concepts, st.lines.len(), st.verdict
        );
    }
    println!(
        "  field energy {:.3} coherence {:.3} density {:.1} active {:?}",
        ev.field.energy, ev.field.coherence, ev.field.density, ev.field.active
    );
    println!("  verdict {:+} anchored {}", ev.verdict, ev.anchored);

    // the operators parse as before, and at least one statement bound two real concepts
    assert_eq!(ev.statements[0].op, Op::ResonantBridge);
    assert_eq!(ev.statements[1].op, Op::Superposition);
    assert_eq!(ev.statements[2].op, Op::Tool);
    assert!(ev.statements.iter().any(|s| s.concepts.len() >= 2), "no statement bound two concepts");
    assert!(ev.statements.iter().all(|s| s.lines.iter().all(|l| l.lambda_m > 0.0)));
    // the tool name is NOT forced onto a concept
    assert!(ev.statements[2].concepts.is_empty(), "a tool name was treated as a concept");

    // field is content-derived: same script -> identical field, and it does not accumulate
    let ev2 = evaluate(script, a);
    assert_eq!(ev.field.energy, ev2.field.energy);
    assert_eq!(ev.field.coherence, ev2.field.coherence);
    assert_eq!(ev.field.density, ev2.field.density);

    // empty input -> empty field and an UNKNOWN verdict, not an optimistic +1
    let empty = evaluate("", a);
    assert_eq!(empty.verdict, 0);
    assert_eq!(empty.field.density, 0.0);
    assert_eq!(empty.field.energy, 0.0);

    // prose with no concept binds nothing and stays unknown
    let prose = evaluate("Deploy the kernel to prod tonight", a);
    println!("  prose-only: verdict {:+} concepts {:?}", prose.verdict,
             prose.statements.iter().map(|s| s.concepts.len()).collect::<Vec<_>>());
    assert_eq!(prose.verdict, 0);

    // an explicit rejection still wins
    let rej = evaluate("Monad Truth Anchor ~> REJECT", a);
    assert_eq!(rej.verdict, -1);
}

/// Coherence means interference between lines, not how well each name matched itself.
#[test]
fn coherence_is_interference_between_lines() {
    use trm_uefi_kernel::aura_spectrum::{
        evaluate, fractal_states, interference, line_width_from_dh,
    };
    let a = MONAD_TRUTH_ANCHOR;

    // ratio 1:1 and 2:1 are consonant; 1.5 is half-way; a distant ratio is not consonant at all
    assert!((interference(528.0, 528.0) - 1.0).abs() < 1e-12);
    assert!((interference(264.0, 528.0) - 1.0).abs() < 1e-12, "octave must read 1.0");
    assert!((interference(352.0, 528.0) - 0.5).abs() < 1e-12, "3:2 must read 0.5");
    assert_eq!(interference(1.0, 9.0), 0.0);

    let ev = evaluate("MonadTruth ~> Coherence", a);
    println!("\n=== coherence as interference ===");
    for st in &ev.statements {
        for l in &st.lines {
            println!("   {} f_line {:.3} Hz", l.id, l.f_line_hz);
        }
    }
    println!(
        "   coherence {:.4} (interference)  match_confidence {:.4} (matcher strength)  density {:.1}",
        ev.field.coherence, ev.field.match_confidence, ev.field.density
    );
    let f = ev.statements[0].lines[0].f_line_hz;
    let g = ev.statements[0].lines[1].f_line_hz;
    assert!(
        (ev.field.coherence - interference(f, g)).abs() < 1e-12,
        "field coherence is not the pairwise interference"
    );
    // the two measures must be different quantities, not the same number twice
    assert!(
        (ev.field.coherence - ev.field.match_confidence).abs() > 1e-9,
        "coherence and match_confidence collapsed to one number"
    );

    // one concept has no pair -> no interference -> 0.0, not a default 1.0
    let one = evaluate("MonadTruth ~> Unity", a);
    if one.field.density == 1.0 {
        assert_eq!(one.field.coherence, 0.0);
    }

    // the fractal trit is degenerate at boot, and says so
    let (states, distinct, informative) = fractal_states();
    println!("   fractal states: {distinct} distinct, informative={informative}");
    for (n, dh, st) in &states {
        println!("      {n:<26} D_H {dh:.3} state {st:+}");
    }
    assert!(!informative && distinct == 1, "fabric trit changed shape — re-measure before claiming");
    assert!(line_width_from_dh(1.0) == 0.0);
    assert!((line_width_from_dh(1.5) - 6.0).abs() < 1e-12);
}
