// TE-3 local enthalpy repartition. Family transitions are 1:1 and always
// emit NO_PROPOSAL; generic expansion remains structurally separate.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32 };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> temperature_current:array<f32>;
@group(0) @binding(3) var<storage,read> phase_energy_current:array<f32>;
@group(0) @binding(4) var<storage,read> phase_context:array<u32>;
@group(0) @binding(5) var<storage,read_write> material_next:array<u32>;
@group(0) @binding(6) var<storage,read_write> temperature_next:array<f32>;
@group(0) @binding(7) var<storage,read_write> phase_energy_next:array<f32>;
@group(0) @binding(8) var<storage,read_write> proposal:array<u32>;
const EMPTY:u32=0u;const WATER:u32=4u;const STEAM:u32=6u;const ICE:u32=8u;
const SKIP:u32=1u;const GAS:u32=2u;const SINK:u32=4u;const SEED:u32=8u;const NO_PROPOSAL:u32=0u;
const LF:f32=80.0;const LV:f32=480.0;const CW:f32=2.5;const CI:f32=2.0;const CS:f32=0.8;const BOIL_H:f32=250.0;const STEAM_H:f32=730.0;
const REF:f32=20.0;const TMIN:f32=-250.0;const TMAX:f32=2000.0;
fn sane(t:f32)->f32{if(t!=t||t>1.0e20||t< -1.0e20){return REF;}return clamp(t,TMIN,TMAX);}
fn family(m:u32)->bool{return m==ICE||m==WATER||m==STEAM;}
fn valid(m:u32,e:f32)->bool{if(e!=e||e>1.0e20||e< -1.0e20){return false;}if(m==ICE){return e>=-LF&&e<=0.0;}if(m==WATER){return e>=-LF&&e<=LV;}if(m==STEAM){return e>=0.0&&e<=LV;}return e==0.0;}
fn enthalpy(m:u32,t:f32,e:f32)->f32{if(m==ICE){return CI*t+e;}if(m==WATER){return CW*t+e;}if(m==STEAM){return BOIL_H+CS*(t-100.0)+e;}return 0.0;}
fn store(i:u32,m:u32,t:f32,e:f32){material_next[i]=m;temperature_next[i]=sane(t);phase_energy_next[i]=e;proposal[i]=NO_PROPOSAL;}
fn water_state(i:u32,h:f32,gas:bool,freeze:bool){if(freeze&&h<=-LF){store(i,ICE,(h+LF)/CI,-LF);return;}if(freeze&&h<0.0){store(i,WATER,0.0,h);return;}if(gas&&h>=STEAM_H){store(i,STEAM,100.0+(h-STEAM_H)/CS,LV);return;}if(gas&&h>BOIL_H){store(i,WATER,100.0,h-BOIL_H);return;}store(i,WATER,h/CW,0.0);}
@compute @workgroup_size(64,1,1) fn phase_main(@builtin(global_invocation_id) gid:vec3<u32>){
 let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}proposal[i]=NO_PROPOSAL;let m=material_current[i];let t=sane(temperature_current[i]);let e=phase_energy_current[i];
 if((phase_context[i]&SKIP)!=0u){store(i,m,t,e);return;}if(!valid(m,e)){store(i,m,t,e);return;}if(!family(m)){store(i,m,t,0.0);return;}
 let h=enthalpy(m,t,e);let gas=(phase_context[i]&GAS)!=0u;
 if(m==ICE){let started=e>-LF||t>2.0;if(!started){store(i,m,t,e);return;}if(h< -LF){store(i,ICE,(h+LF)/CI,-LF);return;}if(h<0.0){store(i,ICE,0.0,h);return;}water_state(i,h,gas,false);return;}
 if(m==WATER){if(e<0.0){water_state(i,h,false,true);return;}if(e>0.0){if(h<BOIL_H){store(i,WATER,h/CW,0.0);}else if(h<STEAM_H){store(i,WATER,100.0,h-BOIL_H);}else if(gas){store(i,STEAM,100.0+(h-STEAM_H)/CS,LV);}else{store(i,WATER,(h-LV)/CW,LV);}return;}if(t< -2.0){water_state(i,h,false,true);return;}if(t>100.0&&gas){water_state(i,h,true,false);return;}store(i,WATER,t,0.0);return;}
 let condense=e<LV||(((phase_context[i]&(SINK|SEED))!=0u)&&t<95.0);if(!condense||h>=STEAM_H){store(i,STEAM,100.0+(h-STEAM_H)/CS,LV);return;}if(h>BOIL_H){store(i,STEAM,100.0,h-BOIL_H);return;}water_state(i,h,false,true);
}
