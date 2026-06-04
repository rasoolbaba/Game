// noo_core.rs — SHARED deterministic core for the INTEGRATION milestone ("Noosphere"):
// neural life (INV-0004) living inside the carbon world's biomes (food/temperature matter),
// PLUS an emergent-belief layer (INV-0005): each creature carries a 3-vector "belief" that
// updates by (a) STRONG social conformity to the local average belief, (b) a WEAK imprint
// toward the biome's archetype, with belief inherited at reproduction, and a survival coupling
// (cohesion: being near similar-belief neighbors gives a small energy bonus).
//
// Design intent (to answer the red team's "is it just biome-painting?"): conformity >> imprint,
// initial beliefs RANDOM -> symmetry-breaking & path-dependence. Same biomes + different seed
// should yield DIFFERENT dominant faiths; faith borders need not match biome borders.
//
// no_std-compatible: only `core`, fixed arrays, NO heap/float/I-O. Included by native + wasm.

pub const SCALE: i64 = 65536;
pub const GW: i32 = 64;
pub const GH: i32 = 40;
pub const NCELL: usize = (GW * GH) as usize;
pub const IN: usize = 4; pub const HID: usize = 6; pub const ACT: usize = 5;
pub const B: usize = 3; // belief dimensions (-> RGB)
pub const CAP: usize = 900;

// world thresholds
const SEA: i64 = 27525; const COAST: i64 = 29490; const MOUNT: i64 = 47185;
const COLD: i64 = 21626; const DRY: i64 = 21626; const MID: i64 = 32768; const WET: i64 = 40632;
const GAIN_E: i64 = (14 * SCALE) / 10; const GAIN_M: i64 = (18 * SCALE) / 10;
const OCTAVES: i32 = 5; const BASE_SPACING: i64 = 34;
// biome classes: 0 ocean(impassable),1 water/coast,2 desert,3 grass,4 forest,5 rainforest,6 tundra,7 mountain
pub const NB: usize = 8;
const FERT: [i64; NB] = [0, (3*SCALE)/100, (2*SCALE)/100, (9*SCALE)/100, (13*SCALE)/100, (18*SCALE)/100, (2*SCALE)/100, 0];
// per-biome belief archetype (weak attractor)
const ARCH: [[i64; B]; NB] = [
    [SCALE/2, SCALE/2, SCALE/2],                 // ocean (unused)
    [(30*SCALE)/100,(50*SCALE)/100,(70*SCALE)/100], // water
    [(85*SCALE)/100,(15*SCALE)/100,(15*SCALE)/100], // desert
    [(20*SCALE)/100,(80*SCALE)/100,(30*SCALE)/100], // grass
    [(15*SCALE)/100,(70*SCALE)/100,(50*SCALE)/100], // forest
    [(10*SCALE)/100,(50*SCALE)/100,(85*SCALE)/100], // rainforest
    [(40*SCALE)/100,(30*SCALE)/100,(90*SCALE)/100], // tundra
    [(60*SCALE)/100,(60*SCALE)/100,(60*SCALE)/100], // mountain
];

const STARTE: i64 = 5 * SCALE; const MAXE: i64 = 12 * SCALE;
const METAB: i64 = (8 * SCALE) / 100; const COLDPEN: i64 = (6 * SCALE) / 100; const REPRO: i64 = 8 * SCALE;
const FOOD_MAX: i64 = 3 * SCALE; const BITE: i64 = 2 * SCALE;
const CONFORMITY: i64 = (18 * SCALE) / 100; pub const IMPRINT: i64 = (1 * SCALE) / 100;
const COH: i64 = (5 * SCALE) / 100; const BELIEF_MUT: i64 = (4 * SCALE) / 100;
const RB: i32 = 2; const RF: i32 = 4;
const GOLDEN: u64 = 0x9E3779B97F4A7C15;

#[inline] fn fmul(a: i64, b: i64) -> i64 { a.wrapping_mul(b) >> 16 }
#[inline] fn clampu(v: i64) -> i64 { if v < 0 { 0 } else if v > SCALE { SCALE } else { v } }
#[inline] fn contrast(v: i64, g: i64) -> i64 { clampu(SCALE/2 + fmul(v - SCALE/2, g)) }
#[inline] fn idx(x: i32, y: i32) -> usize { (y * GW + x) as usize }

#[derive(Clone, Copy)] pub struct Rng { pub s: u64 }
impl Rng {
    #[inline] fn next(&mut self) -> u64 { let mut x=self.s; x^=x<<13; x^=x>>7; x^=x<<17; self.s=x; x }
    #[inline] fn rangei(&mut self, n: i64) -> i64 { (self.next() % (n as u64)) as i64 }
    #[inline] fn weight(&mut self) -> i64 { (self.next() % ((2*SCALE) as u64)) as i64 - SCALE }
    #[inline] fn unit(&mut self) -> i64 { (self.next() % (SCALE as u64)) as i64 } // [0,SCALE)
}

fn hash2(xi: i32, yi: i32, seed: u64) -> u64 {
    let mut h = seed ^ (xi as i64 as u64).wrapping_mul(GOLDEN) ^ (yi as i64 as u64).wrapping_mul(0xC2B2AE3D27D4EB4F);
    h ^= h>>33; h = h.wrapping_mul(0xFF51AFD7ED558CCD); h ^= h>>33; h = h.wrapping_mul(0xC4CEB9FE1A85EC53); h ^= h>>33; h
}
#[inline] fn lattice(xi: i32, yi: i32, s: u64) -> i64 { (hash2(xi, yi, s) & 0xFFFF) as i64 }
#[inline] fn smooth(t: i64) -> i64 { fmul(fmul(t, t), 3*SCALE - 2*t) }
#[inline] fn lerp(a: i64, b: i64, t: i64) -> i64 { a + fmul(b - a, t) }
fn vnoise(px: i64, py: i64, s: u64) -> i64 {
    let xi=(px>>16) as i32; let yi=(py>>16) as i32; let tx=px&0xFFFF; let ty=py&0xFFFF;
    let v00=lattice(xi,yi,s); let v10=lattice(xi+1,yi,s); let v01=lattice(xi,yi+1,s); let v11=lattice(xi+1,yi+1,s);
    let sx=smooth(tx); let sy=smooth(ty); lerp(lerp(v00,v10,sx), lerp(v01,v11,sx), sy)
}
fn fbm(x: i32, y: i32, seed: u64) -> i64 {
    let mut sum=0i64; let mut norm=0i64; let mut o=0;
    while o<OCTAVES { let sp=if (BASE_SPACING>>o)<1 {1} else {BASE_SPACING>>o};
        let px=((x as i64)*SCALE)/sp; let py=((y as i64)*SCALE)/sp; let amp=SCALE>>o;
        sum += fmul(vnoise(px,py,seed.wrapping_add((o as u64).wrapping_mul(0x9E3779B1))), amp); norm+=amp; o+=1; }
    (sum*SCALE)/norm
}
fn classify(e: i64, t: i64, m: i64) -> u8 {
    if e < SEA { return 0; }
    if e < COAST { return 1; }
    if e >= MOUNT { return 7; }
    if t < COLD { return 6; }
    if m < DRY { 2 } else if m < MID { 3 } else if m < WET { 4 } else { 5 }
}

#[derive(Clone, Copy)] pub struct Brain { w1:[[i64;IN];HID], b1:[i64;HID], w2:[[i64;HID];ACT], b2:[i64;ACT] }
impl Brain {
    const ZERO: Brain = Brain{ w1:[[0;IN];HID], b1:[0;HID], w2:[[0;HID];ACT], b2:[0;ACT] };
    fn random(seed: u64) -> Brain { let mut r=Rng{s:seed|1}; let mut b=Brain::ZERO;
        let mut j=0; while j<HID { let mut i=0; while i<IN { b.w1[j][i]=r.weight(); i+=1; } b.b1[j]=r.weight(); j+=1; }
        let mut k=0; while k<ACT { let mut j2=0; while j2<HID { b.w2[k][j2]=r.weight(); j2+=1; } b.b2[k]=r.weight(); k+=1; } b }
    fn mutate(&self, r: &mut Rng) -> Brain { let mut c=*self; let m=SCALE/16;
        let mut j=0; while j<HID { let mut i=0; while i<IN { c.w1[j][i]+=r.rangei(2*m)-m; i+=1; } c.b1[j]+=r.rangei(2*m)-m; j+=1; }
        let mut k=0; while k<ACT { let mut j2=0; while j2<HID { c.w2[k][j2]+=r.rangei(2*m)-m; j2+=1; } c.b2[k]+=r.rangei(2*m)-m; k+=1; } c }
    fn decide(&self, inp:[i64;IN]) -> usize { let mut h=[0i64;HID];
        let mut j=0; while j<HID { let mut s=self.b1[j]; let mut i=0; while i<IN { s+=fmul(self.w1[j][i],inp[i]); i+=1; } h[j]=if s>0{s}else{0}; j+=1; }
        let mut best=0usize; let mut bv=i64::MIN; let mut k=0; while k<ACT { let mut s=self.b2[k]; let mut j2=0; while j2<HID { s+=fmul(self.w2[k][j2],h[j2]); j2+=1; } if s>bv{bv=s;best=k;} k+=1; } best }
}

#[derive(Clone, Copy)] pub struct Creature { pub x:i32, pub y:i32, pub energy:i64, brain:Brain, pub belief:[i64;B], pub genome:u64, pub alive:bool }
impl Creature { const BLANK: Creature = Creature{ x:0,y:0,energy:0, brain:Brain::ZERO, belief:[0;B], genome:0, alive:false }; }

pub struct World {
    pub biome:[u8;NCELL], pub temp:[i32;NCELL], pub food:[i64;NCELL],
    bs:[[i64;NCELL];B], bc:[i32;NCELL],
    pub cre:[Creature;CAP], pub count:usize,
    pub rng:Rng, genome_ctr:u64, pub tick:u64, pub births:u64, pub deaths:u64, pub eaten:u64, pub imprint:i64,
}

impl World {
    pub const ZERO: World = World {
        biome:[0;NCELL], temp:[0;NCELL], food:[0;NCELL],
        bs:[[0;NCELL];B], bc:[0;NCELL],
        cre:[Creature::BLANK;CAP], count:0,
        rng:Rng{s:0}, genome_ctr:0, tick:0, births:0, deaths:0, eaten:0, imprint:0,
    };

    // world_seed = terrain (held fixed for path-dependence tests); sim_seed = creatures/RNG;
    // imprint = biome-archetype pull strength (pass 0 for the T8 ablation / "beta=0").
    pub fn init(&mut self, world_seed: u64, sim_seed: u64, imprint: i64) {
        *self = World::ZERO;
        self.imprint = imprint;
        // generate biome world from world_seed (terrain independent of creatures)
        let half=(GH as i64)/2; let mut y=0;
        while y<GH { let dist=((y as i64)-half).abs(); let lat=SCALE-(dist*SCALE)/half; let mut x=0;
            while x<GW { let i=idx(x,y);
                let e=contrast(fbm(x,y,world_seed), GAIN_E);
                let m=contrast(fbm(x,y,world_seed^0xA5A55A5ADEADBEEF), GAIN_M);
                let above=if e>SEA {e-SEA} else {0};
                let t=clampu(lat - fmul((12*SCALE)/10, above));
                self.biome[i]=classify(e,t,m); self.temp[i]=t as i32;
                self.food[i]= if FERT[self.biome[i] as usize]>0 { FOOD_MAX/2 } else { 0 };
                x+=1; } y+=1; }
        // seed creatures from sim_seed with RANDOM beliefs (symmetry-breaking -> path-dependence)
        self.rng = Rng{ s: sim_seed | 1 };
        let mut k=0; let mut tries=0;
        while k<60 && tries<5000 { tries+=1;
            let x=(self.rng.next()%(GW as u64)) as i32; let y=(self.rng.next()%(GH as u64)) as i32;
            let b=self.biome[idx(x,y)]; if b==0 || b==7 { continue; }
            self.genome_ctr=self.genome_ctr.wrapping_add(GOLDEN);
            let g=sim_seed ^ self.genome_ctr;
            let mut br=Rng{ s: g.wrapping_mul(0xD1B54A32D192ED03) | 1 };
            let belief=[br.unit(), br.unit(), br.unit()];
            self.cre[self.count]=Creature{ x,y, energy:STARTE, brain:Brain::random(g), belief, genome:g, alive:true };
            self.count+=1; k+=1;
        }
    }

    // local belief average + neighbor count from the per-cell field (radius RB)
    fn local_belief(&self, x: i32, y: i32) -> ([i64;B], i32) {
        let mut s=[0i64;B]; let mut c=0i32;
        let mut dy=-RB; while dy<=RB { let ny=y+dy; if ny>=0 && ny<GH { let mut dx=-RB; while dx<=RB { let nx=x+dx;
            if nx>=0 && nx<GW { let j=idx(nx,ny); let cc=self.bc[j]; if cc>0 { c+=cc; let mut k=0; while k<B { s[k]+=self.bs[k][j]; k+=1; } } }
            dx+=1; } } dy+=1; }
        if c>0 { let mut a=[0i64;B]; let mut k=0; while k<B { a[k]=s[k]/(c as i64); k+=1; } (a,c) } else { ([0;B],0) }
    }

    pub fn step(&mut self) {
        // regrow food
        let mut i=0; while i<NCELL { let f=FERT[self.biome[i] as usize]; if f>0 { self.food[i]+=f; if self.food[i]>FOOD_MAX {self.food[i]=FOOD_MAX;} } i+=1; }
        // rebuild belief field from start-of-tick positions
        i=0; while i<NCELL { self.bc[i]=0; self.bs[0][i]=0; self.bs[1][i]=0; self.bs[2][i]=0; i+=1; }
        let mut c=0; while c<self.count { if self.cre[c].alive { let j=idx(self.cre[c].x,self.cre[c].y); self.bc[j]+=1; let mut k=0; while k<B { self.bs[k][j]+=self.cre[c].belief[k]; k+=1; } } c+=1; }

        let n=self.count; let mut child=0usize;
        let mut ci=0;
        while ci<n {
            if !self.cre[ci].alive { ci+=1; continue; }
            let (cx,cy)=(self.cre[ci].x, self.cre[ci].y);
            // perceive: best-food direction within RF (passable cells)
            let mut bestf=-1i64; let mut bdx=0i32; let mut bdy=0i32;
            let mut dy=-RF; while dy<=RF { let ny=cy+dy; if ny>=0 && ny<GH { let mut dx=-RF; while dx<=RF { let nx=cx+dx;
                if nx>=0 && nx<GW { let j=idx(nx,ny); if self.food[j]>bestf { bestf=self.food[j]; bdx=dx; bdy=dy; } } dx+=1; } } dy+=1; }
            let dirx=if bdx==0 {0} else {(bdx as i64*SCALE)/(RF as i64)};
            let diry=if bdy==0 {0} else {(bdy as i64*SCALE)/(RF as i64)};
            let en=(self.cre[ci].energy*SCALE)/MAXE;
            let tn=self.temp[idx(cx,cy)] as i64;
            let a=self.cre[ci].brain.decide([dirx,diry,en,tn]);
            let (mut nx, mut ny)=match a {1=>(cx,cy-1),2=>(cx,cy+1),3=>(cx-1,cy),4=>(cx+1,cy),_=>(cx,cy)};
            if nx<0||nx>=GW||ny<0||ny>=GH || self.biome[idx(nx,ny)]==0 { nx=cx; ny=cy; } // block ocean/edges
            self.cre[ci].x=nx; self.cre[ci].y=ny;
            // eat
            let j=idx(nx,ny); if self.food[j]>0 { let bite=if self.food[j]<BITE {self.food[j]} else {BITE}; self.food[j]-=bite; self.cre[ci].energy+=bite; self.eaten+=1; if self.cre[ci].energy>MAXE {self.cre[ci].energy=MAXE;} }
            // belief: conformity + biome imprint
            let (navg,ncount)=self.local_belief(cx,cy);
            let arch=ARCH[self.biome[idx(nx,ny)] as usize];
            if ncount>0 { let mut k=0; while k<B { self.cre[ci].belief[k]=clampu(self.cre[ci].belief[k] + fmul(CONFORMITY, navg[k]-self.cre[ci].belief[k]) + fmul(self.imprint, arch[k]-self.cre[ci].belief[k])); k+=1; } }
            else { let mut k=0; while k<B { self.cre[ci].belief[k]=clampu(self.cre[ci].belief[k] + fmul(self.imprint, arch[k]-self.cre[ci].belief[k])); k+=1; } }
            // cohesion: energy bonus for matching local faith (survival coupling)
            if ncount>1 { let mut d=0i64; let mut k=0; while k<B { let diff=self.cre[ci].belief[k]-navg[k]; d+= if diff<0 {-diff} else {diff}; k+=1; } d/=B as i64;
                let sim=SCALE-d; self.cre[ci].energy += fmul(COH, sim - SCALE/2); }
            // metabolism (cold costs more)
            let mut cost=METAB; if (self.temp[idx(nx,ny)] as i64) < COLD { cost+=COLDPEN; }
            self.cre[ci].energy -= cost;
            if self.cre[ci].energy<=0 { self.cre[ci].alive=false; self.deaths+=1; ci+=1; continue; }
            // reproduce
            if self.cre[ci].energy>=REPRO && n+child<CAP {
                let half=self.cre[ci].energy/2; self.cre[ci].energy=half;
                self.genome_ctr=self.genome_ctr.wrapping_add(GOLDEN);
                let g=self.cre[ci].genome ^ self.genome_ctr ^ self.rng.next();
                let mut mr=Rng{s:g|1};
                let mut belief=self.cre[ci].belief; let mut k=0; while k<B { belief[k]=clampu(belief[k]+mr.rangei(2*BELIEF_MUT)-BELIEF_MUT); k+=1; }
                // place child on a passable adjacent cell (else on parent cell)
                let mut px=nx+1; let mut py=ny; if px>=GW||self.biome[idx(px,py)]==0 { px=nx; py=ny; }
                self.cre[n+child]=Creature{ x:px,y:py, energy:half, brain:self.cre[ci].brain.mutate(&mut mr), belief, genome:g, alive:true };
                child+=1; self.births+=1;
            }
            ci+=1;
        }
        self.count=n+child;
        let mut w=0; let mut r=0; while r<self.count { if self.cre[r].alive { if w!=r { self.cre[w]=self.cre[r]; } w+=1; } r+=1; }
        self.count=w; self.tick+=1;
    }

    // faith bucket of a creature: 3-bit (each belief channel high/low) -> 0..7
    pub fn faith(&self, i: usize) -> u8 {
        let b=&self.cre[i].belief; let mut f=0u8;
        if b[0]>SCALE/2 {f|=1;} if b[1]>SCALE/2 {f|=2;} if b[2]>SCALE/2 {f|=4;} f
    }

    pub fn fingerprint(&self) -> u64 {
        let prime:u64=1099511628211; let mut h:u64=14695981039346656037; let mut i=0;
        while i<self.count { h=(h^(self.cre[i].x as u64)).wrapping_mul(prime); h=(h^(self.cre[i].y as u64)).wrapping_mul(prime);
            h=(h^(self.cre[i].energy as u64)).wrapping_mul(prime); let mut k=0; while k<B { h=(h^(self.cre[i].belief[k] as u64)).wrapping_mul(prime); k+=1; }
            h=(h^self.cre[i].genome).wrapping_mul(prime); i+=1; } h
    }
}
