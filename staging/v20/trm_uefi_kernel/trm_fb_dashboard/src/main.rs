mod fb;
mod font;
mod input;
mod net;
mod quic;
mod render;
mod sys;

use render::Data;

fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    for i in 0..=(hay.len() - needle.len()) {
        if &hay[i..i + needle.len()] == needle {
            return Some(i);
        }
    }
    None
}

fn parse_u64(hay: &[u8], start: usize) -> u64 {
    let mut v = 0u64;
    let mut i = start;
    while i < hay.len() && hay[i].is_ascii_digit() {
        v = v.saturating_mul(10).saturating_add((hay[i] - b'0') as u64);
        i += 1;
    }
    v
}

fn main() {
    let net_arg = std::env::args().nth(1).unwrap_or_default();
    // Frames land in the boot log too, so a later read of /mnt/rw/trm_boot.log proves what the screen
    // drew -- a screen that shows a dashboard but leaves no record of it is a claim, not a measurement.
    // Best-effort: the renderer is a display process and its log is secondary to its pixels.
    let blog = sys::open(b"/mnt/rw/trm_boot.log\0", sys::O_WRONLY | sys::O_APPEND).unwrap_or(-1);
    let say = |msg: &str| {
        let _ = sys::write(1, msg.as_bytes());
        if blog >= 0 {
            let _ = sys::write(blog, msg.as_bytes());
        }
    };
    say(&format!(
        "DASHBOARD RENDERER: started (pid {})\n",
        std::process::id()
    ));

    let mut fb = None;
    for _ in 0..10 {
        if let Some(f) = fb::Framebuffer::open() {
            say(&format!("DASHBOARD RENDERER: framebuffer {}\n", f.describe()));
            fb = Some(f);
            break;
        }
        sys::nanosleep(1, 0);
    }
    if fb.is_none() {
        say("DASHBOARD RENDERER: no /dev/fb0 after 10s -- text summary only\n");
    }
    let (iw, ih) = match &fb {
        Some(f) => (f.width as i32, f.height as i32),
        None => (1280, 800),
    };
    let mut inp = input::Input::open(iw, ih);
    say(&format!("DASHBOARD RENDERER: {} input device(s)\n", inp.count()));
    for n in inp.names() {
        say(&format!("DASHBOARD RENDERER: input: {}\n", n));
    }

    let mut d = Data::new();
    let nlen = net_arg.len().min(d.net.len());
    d.net[..nlen].copy_from_slice(&net_arg.as_bytes()[..nlen]);
    d.net_len = nlen;
    if d.net_len == 0 {
        let fb_str = b"loopback (127.0.0.1:5183/ui/)";
        let l = fb_str.len().min(d.net.len());
        d.net[..l].copy_from_slice(&fb_str[..l]);
        d.net_len = l;
    }

    let mut infra = Vec::new();
    let mut k3 = Vec::new();
    loop {
        let a = net::get("/infrastructure/status", &mut infra);
        let b = net::get("/k3/status", &mut k3);
        d.tick += 1;
        if a {
            d.experts = net::count_sub(net::body(&infra), b"\"status\":\"ONLINE\"");
        }
        if b {
            let body = net::body(&k3);
            let alg = b"\"algorithm\":\"";
            if let Some(p) = find_sub(body, alg) {
                let s = p + alg.len();
                let mut n = 0;
                let mut i = s;
                while i < body.len() && body[i] != b'"' && body[i] != b' ' && n < d.engine.len() {
                    d.engine[n] = body[i];
                    n += 1;
                    i += 1;
                }
                d.engine_len = n;
            }
            if let Some(fi) = find_sub(body, b"\"fabric\"") {
                let rest = &body[fi..];
                if let Some(bp) = find_sub(rest, b"\"bytes\":") {
                    d.fabric = parse_u64(rest, bp + 8);
                }
            }
        }
        let engine = core::str::from_utf8(&d.engine[..d.engine_len]).unwrap_or("?");
        say(&format!(
            "DASHBOARD: frame {} experts={} engine={} fabric={}B\n",
            d.tick, d.experts, engine, d.fabric
        ));
        if let Some(f) = fb.as_mut() {
            inp.poll();
            if inp.click {
                let rects = render::panel_rects(f.width, f.height);
                let mut hit = 4usize;
                for (i, r) in rects.iter().enumerate() {
                    if (inp.cx as usize) >= r.x
                        && (inp.cx as usize) < r.x + r.w
                        && (inp.cy as usize) >= r.y
                        && (inp.cy as usize) < r.y + r.h
                    {
                        hit = i;
                    }
                }
                if hit < 4 {
                    d.sel = hit;
                    say(&format!(
                        "DASHBOARD: selected panel {} at ({}, {})\n",
                        hit, inp.cx, inp.cy
                    ));
                }
            }
            if inp.submit {
                say(&format!("DASHBOARD: input submitted: \"{}\"\n", inp.text));
                let q = inp.text.clone();
                let mut ans = Vec::new();
                if quic::ask(&q, &mut ans) {
                    let l = ans.len().min(d.answer.len());
                    d.answer[..l].copy_from_slice(&ans[..l]);
                    d.answer_len = l;
                    let a = core::str::from_utf8(&d.answer[..d.answer_len]).unwrap_or("?");
                    say(&format!("DASHBOARD: trm:// answer: \"{a}\"\n"));
                } else {
                    d.answer_len = 0;
                    say("DASHBOARD: no reply over trm://\n");
                }
                inp.text.clear();
            }
            let tl = inp.text.len().min(d.input.len());
            d.input[..tl].copy_from_slice(&inp.text.as_bytes()[..tl]);
            d.input_len = tl;
            d.cx = inp.cx as usize;
            d.cy = inp.cy as usize;
            render::draw(f, &d);
        }
        sys::nanosleep(2, 0);
    }
}
