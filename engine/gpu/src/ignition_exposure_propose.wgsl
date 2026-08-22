// TE-4I combustion-stage snapshot -> compact ignition context in proposal.
// Full-world/full-write; no persistent state and no sleeping-cell skip.
struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
};
struct CombDesc {
    is_combustible: u32, ignition: f32, sustain: f32, chemical_delta_t: f32,
    burn_duration: u32, budget_decay: u32, thermal_rates: u32, flame_rates: u32,
};
struct CombTable { table: array<CombDesc, 16> };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> flags_current: array<u32>;
@group(0) @binding(4) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(5) var<uniform> combustion_table: CombTable;
@group(0) @binding(6) var<storage, read_write> proposal: array<u32>;
const EMPTY:u32=0u; const COMBUSTING:u32=1u; const FLAME:u32=2u;
const CONTEXT_IGNITE:u32=1u<<6u; const CONTEXT_AIR:u32=1u<<7u;
const EXPOSURE_LOW:u32=0x0000000Cu; const EXPOSURE_HIGH:u32=0xF0000000u;
fn inside(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);}
fn at(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn exposure(f:u32)->u32{return((f>>2u)&3u)|(((f>>28u)&15u)<<2u);}
fn air_access(x:i32,y:i32)->bool{
    let d=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));
    for(var k=0u;k<4u;k++){let p=vec2(x,y)+d[k];if(inside(p.x,p.y)){let i=at(p.x,p.y);if(material_current[i]==EMPTY&&air_mass_current[i]>0.0){return true;}}}
    return false;
}
fn previous_flames(x:i32,y:i32)->u32{
    let d=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));var n=0u;
    for(var k=0u;k<4u;k++){let p=vec2(x,y)+d[k];if(inside(p.x,p.y)&&((flags_current[at(p.x,p.y)]&FLAME)!=0u)){n+=1u;}}
    return n;
}
@compute @workgroup_size(64,1,1)
fn ignition_exposure_propose_main(@builtin(global_invocation_id) gid:vec3<u32>){
    let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}
    let m=material_current[i];if(m==EMPTY||m>=16u){proposal[i]=0u;return;}
    let desc=combustion_table.table[m];if(desc.is_combustible==0u){proposal[i]=0u;return;}
    let x=i32(i%params.width);let y=i32(i/params.width);let air=air_access(x,y);
    if((flags_current[i]&COMBUSTING)!=0u){proposal[i]=select(0u,CONTEXT_AIR,air);return;}
    let budget=desc.budget_decay&255u;let decay=(desc.budget_decay>>8u)&255u;
    let base=desc.thermal_rates&255u;let max_rate=(desc.thermal_rates>>8u)&255u;let bucket=(desc.thermal_rates>>16u)&255u;
    let bonus=desc.flame_rates&255u;let bonus_cap=(desc.flame_rates>>8u)&255u;
    let t=temperature_current[i];let eligible=air&&t>=desc.ignition;
    var next=exposure(flags_current[i]);
    if(eligible){
        let buckets=u32(floor(max(0.0,t-desc.ignition)/f32(bucket)));
        let thermal=min(max_rate,base+buckets);let flame=min(bonus_cap,previous_flames(x,y)*bonus);
        next=min(budget,next+thermal+flame);
    }else{if(next>decay){next-=decay;}else{next=0u;}}
    let ignite=next>=budget;
    var context=select(next,0u,ignite);context|=select(0u,CONTEXT_IGNITE,ignite);context|=select(0u,CONTEXT_AIR,air);
    proposal[i]=context;
}
