struct Params { cell_count:u32,threads_x:u32,width:u32,height:u32,chunk_size:u32,chunks_x:u32,chunks_y:u32,sleep_enabled:u32,boundary_mode:u32,_pad0:u32,_pad1:u32,_pad2:u32 };
struct ThermalTable { values:array<vec4<f32>,8> };
@group(0) @binding(0) var<uniform> params:Params;
@group(0) @binding(1) var<storage,read> material_current:array<u32>;
@group(0) @binding(2) var<storage,read> temperature_current:array<f32>;
@group(0) @binding(3) var<storage,read> air_mass_current:array<f32>;
@group(0) @binding(4) var<storage,read> air_energy_current:array<f32>;
@group(0) @binding(5) var<uniform> thermal_table:ThermalTable;
@group(0) @binding(6) var<storage,read> thermal_lambda:array<f32>;
@group(0) @binding(7) var<storage,read_write> temperature_next:array<f32>;
@group(0) @binding(8) var<storage,read_write> air_energy_next:array<f32>;
@group(0) @binding(9) var<storage,read> chunk_state:array<u32>;
const EMPTY:u32=0u;const TABLE_LEN:u32=16u;const SEALED:u32=0u;const REF:f32=20.0;const ZERO:f32=273.15;const STD:f32=293.15;const AIR_K:f32=0.025;const INTERFACE:f32=0.05;const BASE:f32=0.12;const DEAD:f32=0.01;
fn in_domain(x:i32,y:i32)->bool{return x>=0&&y>=0&&x<i32(params.width)&&y<i32(params.height);} fn idx(x:i32,y:i32)->u32{return u32(y)*params.width+u32(x);}
fn props(m:u32)->vec2<f32>{let packed=thermal_table.values[m/2u];return select(packed.xy,packed.zw,(m&1u)!=0u);}fn chunk_of(i:u32)->u32{return((i/params.width)/params.chunk_size)*params.chunks_x+((i%params.width)/params.chunk_size);}fn enabled(a:u32,b:u32)->bool{return params.sleep_enabled==0u||chunk_state[chunk_of(a)]==0u||chunk_state[chunk_of(b)]==0u;}
fn has_node(i:u32)->bool{let m=material_current[i];return(m!=EMPTY&&m<TABLE_LEN&&props(m).y>0.0)||(m==EMPTY&&air_mass_current[i]>0.0);}
fn temp(i:u32)->f32{return select(air_energy_current[i]/air_mass_current[i]-ZERO,temperature_current[i],material_current[i]!=EMPTY);} fn cap(i:u32)->f32{return select(air_mass_current[i],props(material_current[i]).y,material_current[i]!=EMPTY);}
fn g(i:u32,j:u32)->f32{let a=material_current[i];let b=material_current[j];if(!has_node(i)||!has_node(j)){return 0.0;}if(a==EMPTY&&b==EMPTY){return AIR_K;}if(a==EMPTY){return min(props(b).x,INTERFACE);}if(b==EMPTY){return min(props(a).x,INTERFACE);}return min(props(a).x,props(b).x);}
fn effective(d:f32)->f32{return select(0.0,d,abs(d)>DEAD);}
fn face(i:u32,nx:i32,ny:i32)->f32{let ti=temp(i);if(!in_domain(nx,ny)){if(params.boundary_mode==SEALED||material_current[i]!=EMPTY){return 0.0;}return BASE*thermal_lambda[i]*AIR_K*effective(20.0-ti);}let n=idx(nx,ny);if(!has_node(n)||!enabled(i,n)){return 0.0;}return BASE*min(thermal_lambda[i],thermal_lambda[n])*g(i,n)*effective(temp(n)-ti);}
fn directed_next(value:f32,direction:f32)->f32{if(direction==0.0){return value;}if(value==0.0){return bitcast<f32>(select(0x80000001u,1u,direction>0.0));}let bits=bitcast<u32>(value);if(direction>0.0){return bitcast<f32>(select(bits-1u,bits+1u,value>0.0));}return bitcast<f32>(select(bits+1u,bits-1u,value>0.0));}
fn ensure_stored_progress(current:f32,proposed:f32,work:f32)->f32{if(work!=0.0&&proposed==current){return directed_next(current,work);}return proposed;}
@compute @workgroup_size(64,1,1) fn unified_thermal_commit_main(@builtin(global_invocation_id) gid:vec3<u32>){let i=gid.y*params.threads_x+gid.x;if(i>=params.cell_count){return;}if(!has_node(i)){temperature_next[i]=REF;if(material_current[i]!=EMPTY){air_energy_next[i]=0.0;}return;}let x=i32(i%params.width);let y=i32(i/params.width);let q=face(i,x-1,y)+face(i,x+1,y)+face(i,x,y-1)+face(i,x,y+1);if(material_current[i]==EMPTY){temperature_next[i]=REF;let proposed=air_energy_current[i]+q;air_energy_next[i]=ensure_stored_progress(air_energy_current[i],proposed,q);}else{let change=q/cap(i);let proposed=temperature_current[i]+change;temperature_next[i]=ensure_stored_progress(temperature_current[i],proposed,change);air_energy_next[i]=0.0;}}
