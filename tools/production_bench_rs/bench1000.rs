//! 1000-task verifiable benchmark - NATIVE RUST.
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;
const CAPS: [(&str,&str);10] = [("France","Paris"),("Japan","Tokyo"),("Italy","Rome"),("Spain","Madrid"),("Egypt","Cairo"),("Kenya","Nairobi"),("Norway","Oslo"),("Peru","Lima"),("Cuba","Havana"),("Nepal","Kathmandu")];
const WORDS: [&str;10] = ["signal","matrix","photon","kernel","vector","anchor","lattice","qubit","sensor","fusion"];
fn rev(s:&str)->String{ s.chars().rev().collect() }
fn gen(n:usize)->Vec<(String,String)>{
  let mut v=Vec::with_capacity(n); let mut s:u64=7;
  let mut nx=|s:&mut u64|->u64{ *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (*s>>33) };
  for i in 0..n{
    match i%4{
      0=>{let a=nx(&mut s)%18+2; let b=nx(&mut s)%18+2;
           v.push((format!("What is {} times {}? Reply with only the number.",a,b), (a*b).to_string()));}
      1=>{let c=&CAPS[(nx(&mut s)%10) as usize];
           v.push((format!("What is the capital of {}? Reply with only the city name.",c.0), c.1.to_string()));}
      2=>{let w=WORDS[(nx(&mut s)%10) as usize];
           v.push((format!("How many letters are in the word {}? Reply with only the number.",w), w.len().to_string()));}
      _=>{let w=WORDS[(nx(&mut s)%10) as usize];
           v.push((format!("Reverse the letters of the word {}. Reply with only the reversed word.",w), rev(w)));}
    }
  }
  v
}
fn esc(s:&str)->String{ s.replace('\\',"\\\\").replace('"',"\\\"") }
fn json_field(body:&str,k:&str)->Option<String>{
  let pat=format!("\"{}\":\"",k); let i=body.find(&pat)?+pat.len();
  let mut o=String::new(); let mut it=body[i..].chars();
  while let Some(c)=it.next(){ if c=='\\'{ if let Some(d)=it.next(){ match d{'n'=>o.push('\n'),'t'=>o.push('\t'),_=>o.push(d)} } } else if c=='"'{break} else {o.push(c)} }
  Some(o)
}
fn ask(msg:&str)->Result<(String,bool),String>{
  let b=format!("{{\"message\":\"{}\"}}",esc(msg));
  let mut c=TcpStream::connect(("127.0.0.1",8022u16)).map_err(|e|e.to_string())?;
  c.set_read_timeout(Some(std::time::Duration::from_secs(180))).ok();
  let r=format!("POST /swarm/chat HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",b.len(),b);
  c.write_all(r.as_bytes()).map_err(|e|e.to_string())?;
  let mut out=String::new(); c.read_to_string(&mut out).map_err(|e|e.to_string())?;
  let body=out.splitn(2,"\r\n\r\n").nth(1).ok_or("no body")?;
  let a=json_field(body,"response").unwrap_or_default();
  let t=body.contains("\"templated\":true") || body.contains("\"templated\": true");
  Ok((a,t))
}
fn norm(s:&str)->String{ s.chars().filter(|c|c.is_alphanumeric()).collect::<String>().to_lowercase() }
fn main(){
  let a:Vec<String>=std::env::args().collect();
  let n:usize=a.get(1).and_then(|x|x.parse().ok()).unwrap_or(1000);
  let w:usize=a.get(2).and_then(|x|x.parse().ok()).unwrap_or(4);
  let tasks=Arc::new(gen(n));
  let idx=Arc::new(Mutex::new(0usize));
  let res:Arc<Mutex<Vec<(bool,f64,bool,String,String)>>>=Arc::new(Mutex::new(Vec::new()));
  let t0=Instant::now();
  let mut hs=Vec::new();
  for _ in 0..w{
    let (t,i,r)=((*&tasks).clone(),idx.clone(),res.clone());
    hs.push(std::thread::spawn(move||{
      loop{
        let k={let mut g=i.lock().unwrap(); if *g>=t.len(){break} let v=*g; *g+=1; v};
        let (q,want)=&t[k];
        let s=Instant::now();
        let (ans,okflag,got,tmpl)=match ask(q){Ok((x,tpl))=>(norm(&x)==norm(want)||norm(&x).starts_with(&norm(want)),true,x,tpl),Err(e)=>(false,false,e,false)};
        r.lock().unwrap().push((ans,s.elapsed().as_secs_f64(),tmpl,got.chars().take(30).collect(),want.clone()));
      }
    }));
  }
  for h in hs{h.join().ok();}
  let wall=t0.elapsed().as_secs_f64();
  let r=res.lock().unwrap();
  let total=r.len();
  let good=r.iter().filter(|x|x.0).count();
  let tmpl=r.iter().filter(|x|x.2).count();
  let errs=r.iter().filter(|x|x.3.contains("os error")||x.3.contains("refused")||x.3.contains("timed out")).count();
  let mut lat:Vec<f64>=r.iter().map(|x|x.1).collect(); lat.sort_by(|a,b|a.partial_cmp(b).unwrap());
  let q=|f:f64| if lat.is_empty(){0.0}else{lat[((lat.len() as f64*f) as usize).min(lat.len()-1)]};
  println!("1000-TASK BENCHMARK (NATIVE RUST, W={})",w);
  println!("  tasks       {}",total);
  println!("  CORRECT     {}  ({:.1}%)",good,100.0*good as f64/total.max(1) as f64);
  println!("  transport errors  {}",errs);
  println!("  templated   {}  (0 = every answer from the real model)",tmpl);
  println!("  wall        {:.1}s   {:.2} tasks/s",wall,total as f64/wall.max(0.001));
  println!("  latency     p50 {:.2}s  p95 {:.2}s  max {:.2}s",q(0.5),q(0.95),lat.last().copied().unwrap_or(0.0));
  for (nm,i) in [("arith",0usize),("capitals",1),("lettercount",2),("reverse",3)]{
    let sub:Vec<_>=r.iter().enumerate().filter(|(k,_)|k%4==i).map(|(_,x)|x).collect();
    if !sub.is_empty(){ let g=sub.iter().filter(|x|x.0).count(); println!("    {:12} {}/{} = {:.0}%",nm,g,sub.len(),100.0*g as f64/sub.len() as f64); }
  }
  for x in r.iter().filter(|x|!x.0).take(5){ println!("    MISS want={:?} got={:?}",x.4,x.3); }
}
