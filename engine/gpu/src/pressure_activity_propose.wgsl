// TE-5R1 sole ACTIVITY_PRESSURE owner. Full-world; no chunk-state input.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,thermal_eps:f32,pressure_eps:f32,_p0:u32,_p1:u32,_p2:u32 };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> phase_energy_current:array<f32>;
@group(0) @binding(3) var<storage,read> pressure_current:array<f32>;
@group(0) @binding(4) var<storage,read> movement_class_table:array<u32>;
@group(0) @binding(5) var<storage,read_write> cell_activity:array<u32>;
const EMPTY:u32=0u;const STEAM:u32=6u;const TABLE_LEN:u32=16u;const LIQUID:u32=2u;const GAS:u32=3u;const PRESSURE_BIT:u32=4u;
const D:f32=0.20;const R:f32=0.02;const LV:f32=480.0;const FULL:f32=100.0;const PMAX:f32=1.0e6;
fn finite(v:f32)->bool{return v==v&&abs(v)<=1.0e20;}fn clean(v:f32)->f32{return clamp(select(0.0,v,finite(v)),0.0,PMAX);}
fn inside(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);}fn at(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn node(m:u32)->bool{if(m==EMPTY){return true;}if(m>=TABLE_LEN){return false;}let c=movement_class_table[m];return c==LIQUID||c==GAS;}
fn steam_load_target(m:u32,e:f32)->f32{if(m!=STEAM){return 0.0;}if(!finite(e)||e<0.0||e>LV){return 0.0;}return FULL*e/LV;}
fn edge(q:f32,x:i32,y:i32)->f32{if(!inside(x,y)){return 0.0;}let n=at(x,y);if(!node(material_current[n])){return 0.0;}return clean(pressure_current[n])-q;}
fn predicted(i:u32)->f32{let m=material_current[i];if(!node(m)){return 0.0;}let q=clean(pressure_current[i]);let x=i32(i%params.width);let y=i32(i/params.width);let sum=edge(q,x-1,y)+edge(q,x+1,y)+edge(q,x,y-1)+edge(q,x,y+1);return clean(q+D*sum+R*(steam_load_target(m,phase_energy_current[i])-q));}
@compute @workgroup_size(64,1,1) fn pressure_activity_propose_main(@builtin(global_invocation_id) gid:vec3<u32>){let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}let q=select(0.0,clean(pressure_current[i]),node(material_current[i]));if(abs(predicted(i)-q)>params.pressure_eps){cell_activity[i]|=PRESSURE_BIT;}}
