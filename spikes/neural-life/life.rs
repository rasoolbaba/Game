// life.rs — NEURAL-LIFE PROTOTYPE (throwaway spike, not final game code).
// Demonstrates INV-0004: deterministic fixed-point neural creatures with biochemical
// drive (energy/hunger) + evolution-at-reproduction, on the proven integer core.
// std only (crates.io blocked here). Compile: rustc -O life.rs -o life
//
// Determinism: ALL state is integer / Q16.16 fixed-point + xorshift64. No floats in the
// sim. Same seed -> bit-identical run (asserted by running twice and comparing the hash).

const SCALE: i64 = 65536; // Q16.16
const W: i32 = 28;
const H: i32 = 28;
const IN: usize = 4;   // [dirX, dirY, energyNorm, bias]
const HID: usize = 6;  // hidden layer (ReLU)
const ACT: usize = 5;  // stay, up, down, left, right
const SENSE_R: i32 = 7;
const MAX_ENERGY: i64 = 10 * SCALE;
const START_ENERGY: i64 = 5 * SCALE;
const METAB: i64 = SCALE / 10;          // 0.1 / tick
const FOOD_ENERGY: i64 = 3 * SCALE;
const REPRO_THRESH: i64 = 8 * SCALE;
const FOOD_PER_TICK: i32 = 6;
const CREATURE_CAP: usize = 600;

#[inline] fn fmul(a: i64, b: i64) -> i64 { (a.wrapping_mul(b)) >> 16 }

struct Rng { s: u64 }
impl Rng {
    #[inline] fn next(&mut self) -> u64 { let mut x=self.s; x^=x<<13; x^=x>>7; x^=x<<17; self.s=x; x }
    #[inline] fn rangei(&mut self, n: i64) -> i64 { (self.next() % (n as u64)) as i64 } // [0,n)
    // small fixed-point weight in [-1,1)
    #[inline] fn weight(&mut self) -> i64 { (self.next() % (2*SCALE as u64)) as i64 - SCALE }
}

#[derive(Clone)]
struct Brain { w1:[[i64;IN];HID], b1:[i64;HID], w2:[[i64;HID];ACT], b2:[i64;ACT] }
impl Brain {
    fn random(seed: u64) -> Brain {
        let mut r = Rng{ s: seed | 1 };
        let mut b = Brain{ w1:[[0;IN];HID], b1:[0;HID], w2:[[0;HID];ACT], b2:[0;ACT] };
        for j in 0..HID { for i in 0..IN { b.w1[j][i]=r.weight(); } b.b1[j]=r.weight(); }
        for k in 0..ACT { for j in 0..HID { b.w2[k][j]=r.weight(); } b.b2[k]=r.weight(); }
        b
    }
    fn mutate(&self, r: &mut Rng) -> Brain {
        let mut c = self.clone();
        let m = SCALE / 16; // mutation amplitude ~0.06
        for j in 0..HID { for i in 0..IN { c.w1[j][i]+= r.rangei((2*m) as i64) - m; } c.b1[j]+= r.rangei((2*m) as i64)-m; }
        for k in 0..ACT { for j in 0..HID { c.w2[k][j]+= r.rangei((2*m) as i64) - m; } c.b2[k]+= r.rangei((2*m) as i64)-m; }
        c
    }
    fn decide(&self, inp:[i64;IN]) -> usize {
        let mut h=[0i64;HID];
        for j in 0..HID { let mut s=self.b1[j]; for i in 0..IN { s+=fmul(self.w1[j][i],inp[i]); } h[j]= if s>0 {s} else {0}; } // ReLU
        let mut best=0usize; let mut bestv=i64::MIN;
        for k in 0..ACT { let mut s=self.b2[k]; for j in 0..HID { s+=fmul(self.w2[k][j],h[j]); } if s>bestv {bestv=s; best=k;} }
        best
    }
}

#[derive(Clone)]
struct Creature { x:i32, y:i32, energy:i64, brain:Brain, genome:u64, alive:bool }

#[inline] fn idx(x:i32,y:i32)->usize { (y*W+x) as usize }
#[inline] fn wrap(v:i32,m:i32)->i32 { ((v % m) + m) % m }

fn main() {
    let args:Vec<String>=std::env::args().collect();
    let ticks:u64 = args.get(1).and_then(|s|s.parse().ok()).unwrap_or(400);
    let seed:u64   = args.get(2).and_then(|s|s.parse().ok()).unwrap_or(0x9E3779B97F4A7C15);
    let snapshots = args.get(3).map(|s| s=="snap").unwrap_or(false);

    let mut world = Rng{ s: seed | 1 };
    let mut food = vec![false; (W*H) as usize];
    // seed initial food
    for _ in 0..(W*H/5) { let c=(world.next() % (W*H) as u64) as usize; food[c]=true; }

    // initial population
    let mut creatures: Vec<Creature> = Vec::new();
    let mut genome_ctr: u64 = 0;
    for _ in 0..30 {
        genome_ctr = genome_ctr.wrapping_add(0x9E3779B97F4A7C15);
        let g = seed ^ genome_ctr;
        let x=(world.next()%(W as u64)) as i32; let y=(world.next()%(H as u64)) as i32;
        creatures.push(Creature{ x,y, energy:START_ENERGY, brain:Brain::random(g), genome:g, alive:true });
    }

    let mut pop_series: Vec<usize> = Vec::new();
    let mut births=0u64; let mut deaths=0u64; let mut total_eaten=0u64;

    for t in 0..ticks {
        // regrow food
        for _ in 0..FOOD_PER_TICK { let c=(world.next()%(W*H) as u64) as usize; food[c]=true; }

        let n = creatures.len(); // only pre-existing creatures act this tick
        let mut new_children: Vec<Creature> = Vec::new();
        for i in 0..n {
            if !creatures[i].alive { continue; }
            let (cx,cy)=(creatures[i].x, creatures[i].y);
            // perceive: nearest food within radius (Manhattan scan), direction normalized by SENSE_R
            let (mut bestd, mut bdx, mut bdy)=(i32::MAX,0i32,0i32);
            for dy in -SENSE_R..=SENSE_R { for dx in -SENSE_R..=SENSE_R {
                let nx=wrap(cx+dx,W); let ny=wrap(cy+dy,H);
                if food[idx(nx,ny)] { let d=dx.abs()+dy.abs(); if d<bestd { bestd=d; bdx=dx; bdy=dy; } }
            }}
            let dirx = if bestd==i32::MAX {0} else { (bdx as i64 * SCALE)/(SENSE_R as i64) };
            let diry = if bestd==i32::MAX {0} else { (bdy as i64 * SCALE)/(SENSE_R as i64) };
            let en = (creatures[i].energy * SCALE) / MAX_ENERGY; // [0,1]
            let inp=[dirx,diry,en,SCALE];
            let a = creatures[i].brain.decide(inp);
            let (nx,ny)= match a {1=>(cx,wrap(cy-1,H)),2=>(cx,wrap(cy+1,H)),3=>(wrap(cx-1,W),cy),4=>(wrap(cx+1,W),cy),_=>(cx,cy)};
            creatures[i].x=nx; creatures[i].y=ny;
            // eat
            if food[idx(nx,ny)] { food[idx(nx,ny)]=false; creatures[i].energy += FOOD_ENERGY; total_eaten+=1;
                if creatures[i].energy>MAX_ENERGY {creatures[i].energy=MAX_ENERGY;} }
            // metabolism
            creatures[i].energy -= METAB;
            if creatures[i].energy <= 0 { creatures[i].alive=false; deaths+=1; continue; }
            // reproduce
            if creatures[i].energy >= REPRO_THRESH && creatures.len()+new_children.len() < CREATURE_CAP {
                let half = creatures[i].energy/2; creatures[i].energy=half;
                genome_ctr = genome_ctr.wrapping_add(0x9E3779B97F4A7C15);
                let g = creatures[i].genome ^ genome_ctr ^ (world.next());
                let mut mr = Rng{ s: g | 1 };
                let child = Creature{ x:wrap(cx+1,W), y:cy, energy:half, brain:creatures[i].brain.mutate(&mut mr), genome:g, alive:true };
                new_children.push(child); births+=1;
            }
        }
        creatures.extend(new_children);
        creatures.retain(|c| c.alive);
        pop_series.push(creatures.len());

        if snapshots && (t==0 || t==ticks/2 || t==ticks-1) {
            print_snapshot(t, &creatures, &food);
        }
        if creatures.is_empty() { pop_series.push(0); break; }
    }

    // deterministic fingerprint over final creature states
    let mut acc:u64 = 14695981039346656037; let prime:u64=1099511628211;
    let fold=|acc:&mut u64, v:u64| { *acc = (*acc ^ v).wrapping_mul(prime); };
    let mut fp=acc;
    for c in &creatures { fold(&mut fp, c.x as u64); fold(&mut fp, c.y as u64); fold(&mut fp, c.energy as u64); fold(&mut fp, c.genome); }
    acc = fp;

    let peak = pop_series.iter().cloned().max().unwrap_or(0);
    let final_pop = creatures.len();
    println!("{{\"impl\":\"neural-life\",\"ticks\":{},\"seed\":{},\"final_pop\":{},\"peak_pop\":{},\"births\":{},\"deaths\":{},\"eaten\":{},\"fingerprint\":\"{}\"}}",
        ticks, seed, final_pop, peak, births, deaths, total_eaten, acc);
}

fn print_snapshot(t:u64, creatures:&[Creature], food:&[bool]) {
    let mut grid = vec![b'.'; (W*H) as usize];
    for (c,&f) in food.iter().enumerate() { if f { grid[c]=b','; } }
    for cr in creatures { if cr.alive { grid[idx(cr.x,cr.y)]=b'@'; } }
    println!("--- tick {} | pop {} ---", t, creatures.len());
    for y in 0..H { let mut line=String::new(); for x in 0..W { line.push(grid[idx(x,y)] as char); } println!("{}", line); }
}
