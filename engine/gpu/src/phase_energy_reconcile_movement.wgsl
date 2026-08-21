// TE-3 Matter-owned phase energy follows the same accepted movement edge.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32 };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> claim:array<u32>;
@group(0) @binding(3) var<storage,read> phase_energy_current:array<f32>;
@group(0) @binding(4) var<storage,read> material_next:array<u32>;
@group(0) @binding(5) var<storage,read_write> phase_energy_next:array<f32>;
@group(0) @binding(6) var<storage,read> chunk_state:array<u32>;
const ICE:u32=8u;const WATER:u32=4u;const STEAM:u32=6u;
const KIND_SOURCE:u32=1u;const KIND_DEST:u32=2u;const VOID_PEER:u32=0x3FFFFFFFu;
fn family(m:u32)->bool{return m==ICE||m==WATER||m==STEAM;}
fn canonical(m:u32)->f32{if(m==ICE){return -80.0;}if(m==STEAM){return 480.0;}return 0.0;}
fn owned(index:u32)->f32{
 let word=claim[index];let kind=word&3u;let peer=word>>2u;
 if(kind==KIND_SOURCE&&peer==VOID_PEER){return 0.0;}
 if(kind==KIND_SOURCE&&peer<params.cell_count){let other=claim[peer];if((other&3u)==KIND_DEST&&(other>>2u)==index){return phase_energy_current[peer];}}
 if(kind==KIND_DEST&&peer<params.cell_count){let other=claim[peer];if((other&3u)==KIND_SOURCE&&(other>>2u)==index){return phase_energy_current[peer];}}
 return phase_energy_current[index];
}
@compute @workgroup_size(64,1,1) fn phase_energy_reconcile_movement_main(@builtin(global_invocation_id) gid:vec3<u32>){
 let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}
 let next=material_next[i];if(!family(next)){phase_energy_next[i]=0.0;return;}
 let value=owned(i);phase_energy_next[i]=select(canonical(next),value,family(material_current[i])||value!=0.0);
}
