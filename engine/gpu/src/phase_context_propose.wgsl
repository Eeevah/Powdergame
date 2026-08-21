// TE-3 immutable phase context. Reuses dead claim scratch after TE-2.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32,boundary_mode:u32,_p0:u32,_p1:u32,_p2:u32 };
struct ThermalTable { values:array<vec4<f32>,8> };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> temperature_current:array<f32>;
@group(0) @binding(3) var<storage,read> phase_energy_current:array<f32>;
@group(0) @binding(4) var<storage,read> air_mass_current:array<f32>;
@group(0) @binding(5) var<storage,read> air_energy_current:array<f32>;
@group(0) @binding(6) var<storage,read> chunk_state:array<u32>;
@group(0) @binding(7) var<storage,read_write> context_out:array<u32>;
@group(0) @binding(8) var<storage,read> class_table:array<u32>;
@group(0) @binding(9) var<uniform> thermal_table:ThermalTable;
const EMPTY:u32=0u;const WATER:u32=4u;const STEAM:u32=6u;const TABLE_LEN:u32=16u;const CLASS_GAS:u32=3u;
const CONTEXT_SKIP:u32=1u;const CONTEXT_GAS:u32=2u;const CONTEXT_SINK:u32=4u;const CONTEXT_SEED:u32=8u;
const AIR_ZERO:f32=273.15;const AIR_K:f32=0.025;const INTERFACE:f32=0.05;const DEAD:f32=0.01;
fn inside(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);}fn at(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn props(m:u32)->vec2<f32>{let p=thermal_table.values[m/2u];return select(p.xy,p.zw,(m&1u)!=0u);}
fn node(i:u32)->bool{let m=material_current[i];return(m!=EMPTY&&m<TABLE_LEN&&props(m).y>0.0)||(m==EMPTY&&air_mass_current[i]>0.0);}
fn temp(i:u32)->f32{if(material_current[i]!=EMPTY){return temperature_current[i];}return air_energy_current[i]/air_mass_current[i]-AIR_ZERO;}
fn conductance(a:u32,b:u32)->f32{if(!node(a)||!node(b)){return 0.0;}let ma=material_current[a];let mb=material_current[b];if(ma==EMPTY&&mb==EMPTY){return AIR_K;}if(ma==EMPTY){return min(props(mb).x,INTERFACE);}if(mb==EMPTY){return min(props(ma).x,INTERFACE);}return min(props(ma).x,props(mb).x);}
fn runnable_face(i:u32,n:u32)->bool{return conductance(i,n)>0.0&&abs(temp(n)-temp(i))>DEAD;}
fn removes(i:u32,n:u32)->bool{return conductance(i,n)>0.0&&temp(n)<temp(i)-DEAD;}
fn gas_facing(i:u32)->bool{let x=i32(i%params.width);let y=i32(i/params.width);let ds=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));for(var k=0u;k<4u;k++){let p=vec2(x,y)+ds[k];if(inside(p.x,p.y)){let m=material_current[at(p.x,p.y)];if(m==EMPTY||(m<TABLE_LEN&&class_table[m]==CLASS_GAS)){return true;}}}return false;}
fn surface_sink(i:u32)->bool{let x=i32(i%params.width);let y=i32(i/params.width);let self_t=temp(i);let ds=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));for(var k=0u;k<4u;k++){let p=vec2(x,y)+ds[k];if(inside(p.x,p.y)){let n=at(p.x,p.y);let m=material_current[n];if(m!=EMPTY&&m<TABLE_LEN&&class_table[m]!=CLASS_GAS&&temp(n)<=80.0&&temp(n)<=self_t-10.0&&removes(i,n)){return true;}}}return false;}
fn remove_work(i:u32)->bool{let x=i32(i%params.width);let y=i32(i/params.width);let ds=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));for(var k=0u;k<4u;k++){let p=vec2(x,y)+ds[k];if(inside(p.x,p.y)&&removes(i,at(p.x,p.y))){return true;}}return false;}
fn any_work(i:u32)->bool{let x=i32(i%params.width);let y=i32(i/params.width);let ds=array<vec2<i32>,4>(vec2(-1,0),vec2(1,0),vec2(0,-1),vec2(0,1));for(var k=0u;k<4u;k++){let p=vec2(x,y)+ds[k];if(inside(p.x,p.y)&&runnable_face(i,at(p.x,p.y))){return true;}}return false;}
fn eligible(i:u32)->bool{return material_current[i]==STEAM&&phase_energy_current[i]==480.0&&temperature_current[i]<70.0&&!surface_sink(i)&&remove_work(i);}
fn key(i:u32)->vec3<u32>{let x=i%params.width;let y=i/params.width;var tag=0x54453344u;var h=x^(y*0x9E3779B9u)^(tag*0x85EBCA6Bu);h=(h^(h>>16u))*0x7FEB352Du;h=(h^(h>>15u))*0x846CA68Bu;h=h^(h>>16u);return vec3(h,y,x);}
fn less(a:vec3<u32>,b:vec3<u32>)->bool{return a.x<b.x||(a.x==b.x&&(a.y<b.y||(a.y==b.y&&a.z<b.z)));}
fn seed(i:u32)->bool{if(!eligible(i)){return false;}let x=i32(i%params.width);let y=i32(i/params.width);let mine=key(i);for(var dy:i32=-2;dy<=2;dy++){for(var dx:i32=-2;dx<=2;dx++){if(dx==0&&dy==0){continue;}let p=vec2(x+dx,y+dy);if(!inside(p.x,p.y)){continue;}let n=at(p.x,p.y);let e=phase_energy_current[n];if(material_current[n]==STEAM&&e>0.0&&e<480.0&&any_work(n)){return false;}if(eligible(n)&&less(key(n),mine)){return false;}}}return true;}
@compute @workgroup_size(64,1,1) fn phase_context_propose_main(@builtin(global_invocation_id) gid:vec3<u32>){let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}if(params.sleep_enabled!=0u){let c=((i/params.width)/params.chunk_size)*params.chunks_x+((i%params.width)/params.chunk_size);if(chunk_state[c]!=0u){context_out[i]=CONTEXT_SKIP;return;}}var bits=0u;if(material_current[i]==WATER&&gas_facing(i)){bits|=CONTEXT_GAS;}if(material_current[i]==STEAM&&surface_sink(i)){bits|=CONTEXT_SINK;}if(seed(i)){bits|=CONTEXT_SEED;}context_out[i]=bits;}
