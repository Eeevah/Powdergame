// TE-5R1 Air donor scale + total-pressure scratch. Proposal/claim are f32.
struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32,boundary_mode:u32,_p0:u32,_p1:u32,_p2:u32 };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> air_mass_current:array<f32>;
@group(0) @binding(3) var<storage,read> air_energy_current:array<f32>;
@group(0) @binding(4) var<storage,read> chunk_state:array<u32>;
@group(0) @binding(5) var<storage,read> pressure_current:array<f32>;
@group(0) @binding(6) var<storage,read_write> donor_scale:array<f32>;
@group(0) @binding(7) var<storage,read_write> total_pressure:array<f32>;
const EMPTY:u32=0u;const SEALED:u32=0u;const STANDARD:f32=293.15;const RATE:f32=0.125;const FRACTION:f32=0.25;const DEAD:f32=0.001;const SAFETY:f32=0.999999;const PMAX:f32=1.0e6;
fn finite(v:f32)->bool{return v==v&&abs(v)<=3.402823e38;}fn inside(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);}fn at(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn chunk(i:u32)->u32{return((i/params.width)/params.chunk_size)*params.chunks_x+((i%params.width)/params.chunk_size);}fn face(a:u32,b:u32)->bool{return params.sleep_enabled==0u||chunk_state[chunk(a)]==0u||chunk_state[chunk(b)]==0u;}
fn dynamic(i:u32)->f32{let p=pressure_current[i];return clamp(select(0.0,p,finite(p)),0.0,PMAX);}fn air(i:u32)->f32{return select(0.0,air_energy_current[i]/STANDARD,material_current[i]==EMPTY&&air_mass_current[i]>0.0&&finite(air_energy_current[i]));}
fn combined(i:u32)->f32{return dynamic(i)+air(i);}fn raw(a:f32,b:f32)->f32{return RATE*max(a-b-DEAD,0.0);}
fn neighbor_total(i:u32,x:i32,y:i32)->f32{if(!inside(x,y)){return select(1.0,combined(i),params.boundary_mode==SEALED);}let n=at(x,y);if(material_current[n]!=EMPTY||!face(i,n)){return combined(i);}return combined(n);}
@compute @workgroup_size(64,1,1) fn air_flow_scale_main(@builtin(global_invocation_id) gid:vec3<u32>){let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}let own=combined(i);total_pressure[i]=own;donor_scale[i]=0.0;if(material_current[i]!=EMPTY){return;}let m=air_mass_current[i];let e=air_energy_current[i];if(!finite(m)||!finite(e)||m<0.0||e<0.0){return;}let x=i32(i%params.width);let y=i32(i/params.width);let sum=raw(own,neighbor_total(i,x-1,y))+raw(own,neighbor_total(i,x+1,y))+raw(own,neighbor_total(i,x,y-1))+raw(own,neighbor_total(i,x,y+1));if(sum>0.0){donor_scale[i]=min(1.0,FRACTION*m/sum)*SAFETY;}}
