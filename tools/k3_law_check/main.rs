//! Does the kernel's K3 engine satisfy Kleene's three-valued algebra?
//! If any law fails, every quorum/verdict built on the engine is invalid.
//! Compiles the REAL k3_logic.rs from the kernel (not a reimplementation).
extern crate alloc;
mod k3_logic;
use k3_logic::KleeneTernaryEngine as E;

const V: [i8; 3] = [-1, 0, 1];

fn kand(a: i8, b: i8) -> i8 { if a == -1 || b == -1 { -1 } else if a == 1 && b == 1 { 1 } else { 0 } }
fn kor(a: i8, b: i8) -> i8 { if a == 1 || b == 1 { 1 } else if a == -1 && b == -1 { -1 } else { 0 } }
fn knot(a: i8) -> i8 { -a }

fn main() {
    let mut pass = 0u32; let mut fail = 0u32; let mut fails: Vec<String> = Vec::new();

    // 1. operators match the Kleene truth tables
    for &a in &V { for &b in &V {
        if E::kleene_and(a,b) != kand(a,b) { fail+=1; fails.push(format!("AND({},{})={} want {}",a,b,E::kleene_and(a,b),kand(a,b))); } else { pass+=1 }
        if E::kleene_or(a,b)  != kor(a,b)  { fail+=1; fails.push(format!("OR({},{})={} want {}",a,b,E::kleene_or(a,b),kor(a,b))); } else { pass+=1 }
    }}
    // 2. commutativity
    for &a in &V { for &b in &V {
        if E::kleene_and(a,b) != E::kleene_and(b,a) { fail+=1; fails.push(format!("AND not commutative {}/{}",a,b)); } else { pass+=1 }
        if E::kleene_or(a,b)  != E::kleene_or(b,a)  { fail+=1; fails.push(format!("OR not commutative {}/{}",a,b)); } else { pass+=1 }
    }}
    // 3. idempotence
    for &a in &V {
        if E::kleene_and(a,a) != a { fail+=1; fails.push(format!("AND not idempotent {}",a)); } else { pass+=1 }
        if E::kleene_or(a,a)  != a { fail+=1; fails.push(format!("OR not idempotent {}",a)); } else { pass+=1 }
    }
    // 4. identity / annihilator
    for &a in &V {
        if E::kleene_or(a,-1)  != a { fail+=1; fails.push(format!("OR identity(-1) failed at {}",a)); } else { pass+=1 }
        if E::kleene_and(a,1)  != a { fail+=1; fails.push(format!("AND identity(+1) failed at {}",a)); } else { pass+=1 }
        if E::kleene_and(a,-1) != -1 { fail+=1; fails.push(format!("AND annihilator(-1) failed at {}",a)); } else { pass+=1 }
        if E::kleene_or(a,1)   != 1  { fail+=1; fails.push(format!("OR annihilator(+1) failed at {}",a)); } else { pass+=1 }
    }
    // 5. De Morgan
    for &a in &V { for &b in &V {
        if E::kleene_and(a,b) != knot(kor(knot(a),knot(b))) { fail+=1; fails.push(format!("DeMorgan AND {}/{}",a,b)); } else { pass+=1 }
        if E::kleene_or(a,b)  != knot(kand(knot(a),knot(b))) { fail+=1; fails.push(format!("DeMorgan OR {}/{}",a,b)); } else { pass+=1 }
    }}
    // 6. absorption
    for &a in &V { for &b in &V {
        if E::kleene_and(a, E::kleene_or(a,b)) != a { fail+=1; fails.push(format!("absorption-1 {}/{}",a,b)); } else { pass+=1 }
        if E::kleene_or(a, E::kleene_and(a,b)) != a { fail+=1; fails.push(format!("absorption-2 {}/{}",a,b)); } else { pass+=1 }
    }}
    // 7. the SAFETY property the platform relies on
    for &b in &V {
        if E::kleene_and(-1,b) != -1 { fail+=1; fails.push(format!("RETRACT bound something at b={}",b)); } else { pass+=1 }
    }
    // 8. UNKNOWN must not collapse to a decision
    if E::kleene_and(0,0) != 0 { fail+=1; fails.push("AND(0,0) collapsed".into()) } else { pass+=1 }
    if E::kleene_or(0,0)  != 0 { fail+=1; fails.push("OR(0,0) collapsed".into()) } else { pass+=1 }
    // 9. NOT involution
    for &a in &V { if E::kleene_not(E::kleene_not(a)) != a { fail+=1; fails.push(format!("NOT not involutive {}",a)); } else { pass+=1 } }

    println!("K3 KLEENE LAW VERIFICATION - real trm_uefi_kernel/src/k3_logic.rs");
    println!("  checks passed {}   failed {}", pass, fail);
    if fails.is_empty() {
        println!("\n  ALL KLEENE LAWS HOLD on the engine's own truth tables.");
    } else {
        println!("\n  FAILURES (first 20):");
        for f in fails.iter().take(20) { println!("    {}", f); }
    }
}
