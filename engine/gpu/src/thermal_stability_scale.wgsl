struct Params {
    cell_count: u32, threads_x: u32, width: u32, height: u32,
    chunk_size: u32, chunks_x: u32, chunks_y: u32, sleep_enabled: u32,
    boundary_mode: u32, _pad0: u32, _pad1: u32, _pad2: u32,
};
struct ThermalTable { values: array<vec4<f32>, 8> };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> material_current: array<u32>;
@group(0) @binding(2) var<storage, read> temperature_current: array<f32>;
@group(0) @binding(3) var<storage, read> air_mass_current: array<f32>;
@group(0) @binding(4) var<storage, read> air_energy_current: array<f32>;
@group(0) @binding(5) var<uniform> thermal_table: ThermalTable;
@group(0) @binding(6) var<storage, read> chunk_state: array<u32>;
@group(0) @binding(7) var<storage, read_write> thermal_lambda: array<f32>;
const EMPTY: u32 = 0u; const TABLE_LEN: u32 = 16u; const SEALED: u32 = 0u;
const AIR_K: f32 = 0.025; const MATTER_AIR_G: f32 = 0.05;
const BASE: f32 = 0.12; const MAX_MIX: f32 = 0.25;
fn in_domain(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);}
fn index_of(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn chunk_of(i:u32)->u32{return ((i/params.width)/params.chunk_size)*params.chunks_x+((i%params.width)/params.chunk_size);}
fn enabled(a:u32,b:u32)->bool{return params.sleep_enabled==0u||chunk_state[chunk_of(a)]==0u||chunk_state[chunk_of(b)]==0u;}
fn props(m:u32)->vec2<f32>{let packed=thermal_table.values[m/2u];return select(packed.xy,packed.zw,(m&1u)!=0u);}
fn has_node(i:u32)->bool { let m=material_current[i]; return (m!=EMPTY&&m<TABLE_LEN&&props(m).y>0.0)||(m==EMPTY&&air_mass_current[i]>0.0); }
fn capacity(i:u32)->f32 { let m=material_current[i]; return select(air_mass_current[i],props(m).y,m!=EMPTY); }
fn conductance(i:u32,j:u32)->f32 { let a=material_current[i];let b=material_current[j]; if(!has_node(i)||!has_node(j)){return 0.0;} if(a==EMPTY&&b==EMPTY){return AIR_K;} if(a==EMPTY){return min(props(b).x,MATTER_AIR_G);} if(b==EMPTY){return min(props(a).x,MATTER_AIR_G);} return min(props(a).x,props(b).x); }
fn face_g(i:u32,nx:i32,ny:i32)->f32 { if(!in_domain(nx,ny)){if(material_current[i]!=EMPTY||params.boundary_mode==SEALED){return 0.0;}return AIR_K;} let n=index_of(nx,ny); return select(0.0,conductance(i,n),enabled(i,n)); }
@compute @workgroup_size(64,1,1)
fn thermal_stability_scale_main(@builtin(global_invocation_id) gid:vec3<u32>){
 let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;} thermal_lambda[i]=0.0;if(!has_node(i)){return;}
 let x=i32(i%params.width);let y=i32(i/params.width);let sum=face_g(i,x-1,y)+face_g(i,x+1,y)+face_g(i,x,y-1)+face_g(i,x,y+1);
 if(sum>0.0){thermal_lambda[i]=min(1.0,MAX_MIX*capacity(i)/(BASE*sum));}
}
