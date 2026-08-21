// TE-3 identity writers cannot orphan latent state.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32 };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> material_next:array<u32>;
@group(0) @binding(3) var<storage,read> phase_energy_current:array<f32>;
@group(0) @binding(4) var<storage,read_write> phase_energy_next:array<f32>;
const ICE:u32=8u;const WATER:u32=4u;const STEAM:u32=6u;
fn family(m:u32)->bool{return m==ICE||m==WATER||m==STEAM;}
fn canonical(m:u32)->f32{if(m==ICE){return -80.0;}if(m==STEAM){return 480.0;}return 0.0;}
@compute @workgroup_size(64,1,1) fn phase_energy_hygiene_identity_main(@builtin(global_invocation_id) gid:vec3<u32>){
 let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}let a=material_current[i];let b=material_next[i];
 if(!family(b)){phase_energy_next[i]=0.0;}else if(a==b){phase_energy_next[i]=phase_energy_current[i];}else{phase_energy_next[i]=canonical(b);}
}
