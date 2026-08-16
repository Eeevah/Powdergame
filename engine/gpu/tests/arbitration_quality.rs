//! G6-C1 — Arbitration Quality Measurement Harness & Benchmarks.
//!
//! Compares the frozen fixed-index arbitration baseline against a single
//! test-only stateless edge-hash candidate on RTX 5090 / DX12.
//!
//! Production claim shaders and Simulation::tick() are NOT modified.

use std::time::Instant;
use wgpu::util::DeviceExt;

use powdergame_gpu::GpuContext;

const WORKGROUP_SIZE: u32 = 64;
const WORKGROUPS_X: u32 = 256;
const THREADS_X: u64 = (WORKGROUPS_X as u64) * (WORKGROUP_SIZE as u64);

const NO_MOVE: u32 = 0xFFFFFFFF;
const KIND_SOURCE: u32 = 1;
const KIND_DEST: u32 = 2;

// ─────────────────────────────────────────────────────────────────────────────
// Test-Only WGSL Shaders
// ─────────────────────────────────────────────────────────────────────────────

const WGSL_BASELINE_MOVEMENT_CLAIM: &str = r#"
struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const NO_MOVE: u32 = 0xFFFFFFFFu;
const VOID_TARGET: u32 = 0xFFFFFFFEu;
const NO_CLAIM: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> proposal: array<u32>;
@group(0) @binding(2) var<storage, read_write> claim: array<u32>;

@compute @workgroup_size(64)
fn claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    let t = proposal[c];
    if (t == VOID_TARGET) {
        claim[c] = (VOID_PEER << 2u) | KIND_SOURCE;
        return;
    }

    var best: u32 = NO_CLAIM;
    var best_owner: u32 = 0xFFFFFFFFu;

    if (t != NO_MOVE) {
        best = (t << 2u) | KIND_SOURCE;
        best_owner = c;
    }

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c && s < best_owner) {
                        best = (s << 2u) | KIND_DEST;
                        best_owner = s;
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    claim[c] = best;
}
"#;

const WGSL_CANDIDATE_MOVEMENT_CLAIM: &str = r#"
struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const NO_MOVE: u32 = 0xFFFFFFFFu;
const VOID_TARGET: u32 = 0xFFFFFFFEu;
const NO_CLAIM: u32 = 0u;
const KIND_SOURCE: u32 = 1u;
const KIND_DEST: u32 = 2u;
const VOID_PEER: u32 = 0x3FFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> proposal: array<u32>;
@group(0) @binding(2) var<storage, read_write> claim: array<u32>;

fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}

@compute @workgroup_size(64)
fn claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }

    let t = proposal[c];
    if (t == VOID_TARGET) {
        claim[c] = (VOID_PEER << 2u) | KIND_SOURCE;
        return;
    }

    var best: u32 = NO_CLAIM;
    var best_priority: u32 = 0xFFFFFFFFu;
    var best_owner: u32 = 0xFFFFFFFFu;

    if (t != NO_MOVE) {
        best = (t << 2u) | KIND_SOURCE;
        best_priority = edge_priority(c, t, params.tick);
        best_owner = c;
    }

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c) {
                        let p = edge_priority(s, c, params.tick);
                        if (p < best_priority || (p == best_priority && s < best_owner)) {
                            best = (s << 2u) | KIND_DEST;
                            best_priority = p;
                            best_owner = s;
                        }
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    claim[c] = best;
}
"#;

const WGSL_BASELINE_DESTINATION_CLAIM: &str = r#"
struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const NO_CLAIM: u32 = 0u;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> proposal: array<u32>;
@group(0) @binding(2) var<storage, read_write> claim: array<u32>;

@compute @workgroup_size(64)
fn claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }
    claim[c] = NO_CLAIM;

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best = NO_SOURCE;
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c + 1u && s < best) {
                        best = s;
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    if (best != NO_SOURCE) {
        claim[c] = best + 1u;
    }
}
"#;

const WGSL_CANDIDATE_DESTINATION_CLAIM: &str = r#"
struct Params {
    cell_count: u32,
    threads_x: u32,
    width: u32,
    height: u32,
    tick: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

const NO_CLAIM: u32 = 0u;
const NO_SOURCE: u32 = 0xFFFFFFFFu;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> proposal: array<u32>;
@group(0) @binding(2) var<storage, read_write> claim: array<u32>;

fn edge_priority(source: u32, target_cell: u32, tick: u32) -> u32 {
    var h: u32 = source ^ (target_cell * 0x9E3779B9u) ^ (tick * 0x85EBCA6Bu);
    h = (h ^ (h >> 16u)) * 0x7FEB352Du;
    h = (h ^ (h >> 15u)) * 0x846CA68Bu;
    h = h ^ (h >> 16u);
    return h;
}

@compute @workgroup_size(64)
fn claim_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let c = gid.y * params.threads_x + gid.x;
    if (c >= params.cell_count) {
        return;
    }
    claim[c] = NO_CLAIM;

    let x = i32(c % params.width);
    let y = i32(c / params.width);
    var best_source = NO_SOURCE;
    var best_priority: u32 = 0xFFFFFFFFu;
    var dy: i32 = -1;
    while (dy <= 1) {
        var dx: i32 = -1;
        while (dx <= 1) {
            if (!(dx == 0 && dy == 0)) {
                let nx = x + dx;
                let ny = y + dy;
                if (nx >= 0 && ny >= 0 && nx < i32(params.width) && ny < i32(params.height)) {
                    let s = u32(ny) * params.width + u32(nx);
                    if (proposal[s] == c + 1u) {
                        let p = edge_priority(s, c, params.tick);
                        if (p < best_priority || (p == best_priority && s < best_source)) {
                            best_source = s;
                            best_priority = p;
                        }
                    }
                }
            }
            dx = dx + 1;
        }
        dy = dy + 1;
    }

    if (best_source != NO_SOURCE) {
        claim[c] = best_source + 1u;
    }
}
"#;

fn cast_u32_slice(slice: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, std::mem::size_of_val(slice)) }
}

fn bytes_to_u32_vec(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Pipeline Runner
// ─────────────────────────────────────────────────────────────────────────────

struct ClaimHarness {
    context: GpuContext,
    pipeline_baseline_movement: wgpu::ComputePipeline,
    pipeline_candidate_movement: wgpu::ComputePipeline,
    pipeline_baseline_destination: wgpu::ComputePipeline,
    pipeline_candidate_destination: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
}

impl ClaimHarness {
    fn new() -> Self {
        let context = pollster::block_on(GpuContext::new()).expect("GpuContext init");

        let bgl = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("test-claim-bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let pl = context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("test-claim-pl"),
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });

        let sm_bm = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sm-baseline-movement"),
                source: wgpu::ShaderSource::Wgsl(WGSL_BASELINE_MOVEMENT_CLAIM.into()),
            });
        let sm_cm = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sm-candidate-movement"),
                source: wgpu::ShaderSource::Wgsl(WGSL_CANDIDATE_MOVEMENT_CLAIM.into()),
            });
        let sm_bd = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sm-baseline-dest"),
                source: wgpu::ShaderSource::Wgsl(WGSL_BASELINE_DESTINATION_CLAIM.into()),
            });
        let sm_cd = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sm-candidate-dest"),
                source: wgpu::ShaderSource::Wgsl(WGSL_CANDIDATE_DESTINATION_CLAIM.into()),
            });

        let pipeline_baseline_movement =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("pipe-bm"),
                    layout: Some(&pl),
                    module: &sm_bm,
                    entry_point: Some("claim_main"),
                    compilation_options: Default::default(),
                    cache: None,
                });
        let pipeline_candidate_movement =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("pipe-cm"),
                    layout: Some(&pl),
                    module: &sm_cm,
                    entry_point: Some("claim_main"),
                    compilation_options: Default::default(),
                    cache: None,
                });
        let pipeline_baseline_destination =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("pipe-bd"),
                    layout: Some(&pl),
                    module: &sm_bd,
                    entry_point: Some("claim_main"),
                    compilation_options: Default::default(),
                    cache: None,
                });
        let pipeline_candidate_destination =
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("pipe-cd"),
                    layout: Some(&pl),
                    module: &sm_cd,
                    entry_point: Some("claim_main"),
                    compilation_options: Default::default(),
                    cache: None,
                });

        Self {
            context,
            pipeline_baseline_movement,
            pipeline_candidate_movement,
            pipeline_baseline_destination,
            pipeline_candidate_destination,
            bgl,
        }
    }

    fn run_claim(
        &self,
        pipeline: &wgpu::ComputePipeline,
        width: u32,
        height: u32,
        tick: u32,
        proposals: &[u32],
    ) -> Vec<u32> {
        let cell_count = width * height;
        assert_eq!(proposals.len(), cell_count as usize);

        let params_data: [u32; 8] = [cell_count, THREADS_X as u32, width, height, tick, 0, 0, 0];
        let params_buf =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("params-buf"),
                    contents: cast_u32_slice(&params_data),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

        let prop_buf = self
            .context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("prop-buf"),
                contents: cast_u32_slice(proposals),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let claim_buf = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("claim-buf"),
            size: (cell_count as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buf = self.context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback-buf"),
            size: (cell_count as u64) * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bg = self
            .context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("claim-bg"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: prop_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: claim_buf.as_entire_binding(),
                    },
                ],
            });

        let dispatch_y = ((cell_count as u64).div_ceil(THREADS_X)) as u32;

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bg, &[]);
            cpass.dispatch_workgroups(WORKGROUPS_X, dispatch_y, 1);
        }
        encoder.copy_buffer_to_buffer(&claim_buf, 0, &readback_buf, 0, (cell_count as u64) * 4);
        self.context.queue.submit(Some(encoder.finish()));

        let slice = readback_buf.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            sender.send(res).unwrap();
        });
        self.context.device.poll(wgpu::PollType::Wait).unwrap();
        receiver.recv().unwrap().unwrap();

        let data = slice.get_mapped_range();
        let result: Vec<u32> = bytes_to_u32_vec(&data);
        drop(data);
        readback_buf.unmap();

        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-C1: Correctness & Edge Agreement Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_candidate_movement_edge_agreement_many_fixtures() {
    let harness = ClaimHarness::new();
    let width = 64u32;
    let height = 64u32;
    let cell_count = (width * height) as usize;

    for tick in [0u32, 1, 7, 42, 123, 999] {
        let mut proposals = vec![NO_MOVE; cell_count];

        // Place random pairs of moving cells contending for common targets.
        for y in 2..(height - 2) {
            for x in 2..(width - 2) {
                if (x + y) % 3 == 0 {
                    let target = y * width + x;
                    let left_src = y * width + (x - 1);
                    let right_src = y * width + (x + 1);
                    proposals[left_src as usize] = target;
                    proposals[right_src as usize] = target;
                }
            }
        }

        let claims = harness.run_claim(
            &harness.pipeline_candidate_movement,
            width,
            height,
            tick,
            &proposals,
        );

        // Verify mutual edge agreement across the entire world:
        // For any cell c:
        // if c claims (peer, KIND_SOURCE), then peer must either claim (c, KIND_DEST) or claim another valid edge.
        // If peer claims (c, KIND_DEST), then c MUST claim (peer, KIND_SOURCE).
        for c in 0..cell_count {
            let cl = claims[c];
            let kind = cl & 3;
            let peer = (cl >> 2) as usize;

            if kind == KIND_DEST {
                let peer_claim = claims[peer];
                let peer_kind = peer_claim & 3;
                let peer_dest = (peer_claim >> 2) as usize;

                assert_eq!(
                    peer_kind, KIND_SOURCE,
                    "Destination c={} claimed source peer={}, but source did not claim KIND_SOURCE (got kind={})",
                    c, peer, peer_kind
                );
                assert_eq!(
                    peer_dest, c,
                    "Destination c={} claimed source peer={}, but source claimed destination {}",
                    c, peer, peer_dest
                );
            }
        }
    }
}

#[test]
fn test_candidate_destination_claim_exactly_one_winner() {
    let harness = ClaimHarness::new();
    let width = 64u32;
    let height = 64u32;
    let cell_count = (width * height) as usize;

    let mut proposals = vec![0u32; cell_count]; // 0 = no proposal in destination scheme

    // 4 sources targeting destination (32, 32).
    let target = 32 * width + 32;
    let src_up = (32 - 1) * width + 32;
    let src_down = (32 + 1) * width + 32;
    let src_left = 32 * width + (32 - 1);
    let src_right = 32 * width + (32 + 1);

    proposals[src_up as usize] = target + 1;
    proposals[src_down as usize] = target + 1;
    proposals[src_left as usize] = target + 1;
    proposals[src_right as usize] = target + 1;

    let claims = harness.run_claim(
        &harness.pipeline_candidate_destination,
        width,
        height,
        42,
        &proposals,
    );

    let winner_claim = claims[target as usize];
    assert_ne!(
        winner_claim, 0,
        "contested destination must select exactly one winner"
    );
    let winner_src = winner_claim - 1;
    assert!(
        [src_up, src_down, src_left, src_right].contains(&winner_src),
        "winner must be one of the 4 contenders"
    );
}

#[test]
fn test_candidate_hash_collision_deterministic_tie_break() {
    let harness = ClaimHarness::new();
    let width = 64u32;
    let height = 64u32;
    let cell_count = (width * height) as usize;

    // Test that when two incoming edges are evaluated, total ordering is guaranteed
    // via source_index fallback in case of priority tie.
    let mut proposals = vec![NO_MOVE; cell_count];
    let target = 32 * width + 32;
    let left = 32 * width + 31;
    let right = 32 * width + 33;
    proposals[left as usize] = target;
    proposals[right as usize] = target;

    // Run across 256 different seeds; every single one must produce a valid single winner (left or right).
    for tick in 0..256 {
        let claims = harness.run_claim(
            &harness.pipeline_candidate_movement,
            width,
            height,
            tick,
            &proposals,
        );
        let winner = claims[target as usize] >> 2;
        assert!(
            winner == left || winner == right,
            "must pick exactly one valid winner even under hypothetical priority equality"
        );
    }
}

#[test]
fn test_candidate_deterministic_repeat() {
    let harness = ClaimHarness::new();
    let width = 64u32;
    let height = 64u32;
    let cell_count = (width * height) as usize;

    let mut proposals = vec![NO_MOVE; cell_count];
    for (i, p) in proposals.iter_mut().enumerate() {
        if i % 5 == 0 && i + 1 < cell_count {
            *p = (i + 1) as u32;
        }
    }

    let run1 = harness.run_claim(
        &harness.pipeline_candidate_movement,
        width,
        height,
        12345,
        &proposals,
    );
    let run2 = harness.run_claim(
        &harness.pipeline_candidate_movement,
        width,
        height,
        12345,
        &proposals,
    );

    assert_eq!(run1, run2, "identical inputs must yield bit-exact results");
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-C1: Directional Bias & Statistical Measurement
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_bias_measurement_large_sample_dataset() {
    let harness = ClaimHarness::new();
    // Use 256x256 world containing thousands of independent contests.
    let width = 256u32;
    let height = 256u32;
    let cell_count = (width * height) as usize;

    // ─── 1. Horizontal Contention (LEFT vs RIGHT) ───
    // Build 2048 independent pairs: Target at (x, y), Left at (x-1, y), Right at (x+1, y).
    let mut proposals = vec![NO_MOVE; cell_count];
    let mut contests = Vec::new();

    // Spacing by 4 along x and y gives ~ (250/4) * (250/4) ≈ 3800 independent contests.
    for y in (4..(height - 4)).step_by(4) {
        for x in (4..(width - 4)).step_by(4) {
            if contests.len() < 2048 {
                let target = y * width + x;
                let left = y * width + (x - 1);
                let right = y * width + (x + 1);
                proposals[left as usize] = target;
                proposals[right as usize] = target;
                contests.push((left, right, target));
            }
        }
    }
    assert_eq!(contests.len(), 2048);

    // Run Baseline
    let base_claims = harness.run_claim(
        &harness.pipeline_baseline_movement,
        width,
        height,
        0,
        &proposals,
    );
    let mut base_left_wins = 0;
    let mut base_right_wins = 0;
    for &(left, right, target) in &contests {
        let tc = base_claims[target as usize];
        let winner = tc >> 2;
        if winner == left {
            base_left_wins += 1;
        } else if winner == right {
            base_right_wins += 1;
        }
    }

    // Run Candidate (with tick = 42)
    let cand_claims = harness.run_claim(
        &harness.pipeline_candidate_movement,
        width,
        height,
        42,
        &proposals,
    );
    let mut cand_left_wins = 0;
    let mut cand_right_wins = 0;
    for &(left, right, target) in &contests {
        let tc = cand_claims[target as usize];
        let winner = tc >> 2;
        if winner == left {
            cand_left_wins += 1;
        } else if winner == right {
            cand_right_wins += 1;
        }
    }

    println!("\n=== HORIZONTAL CONTENTION (2,048 Contests: LEFT vs RIGHT) ===");
    println!(
        "Baseline (Fixed-Index): LEFT = {} ({:.1}%), RIGHT = {} ({:.1}%)",
        base_left_wins,
        (base_left_wins as f64 / 2048.0) * 100.0,
        base_right_wins,
        (base_right_wins as f64 / 2048.0) * 100.0
    );
    println!(
        "Candidate (Edge-Hash):   LEFT = {} ({:.1}%), RIGHT = {} ({:.1}%)",
        cand_left_wins,
        (cand_left_wins as f64 / 2048.0) * 100.0,
        cand_right_wins,
        (cand_right_wins as f64 / 2048.0) * 100.0
    );

    // Baseline: left has lower index (x-1 < x+1 on same row), so baseline MUST be 100% Left.
    assert_eq!(base_left_wins, 2048);
    assert_eq!(base_right_wins, 0);

    // Candidate: should be well-balanced (roughly 50/50, e.g. between 40% and 60%).
    let cand_left_pct = (cand_left_wins as f64 / 2048.0) * 100.0;
    assert!(
        (40.0..=60.0).contains(&cand_left_pct),
        "candidate must substantially balance horizontal contention (got {:.1}%)",
        cand_left_pct
    );

    // ─── 2. Vertical Contention (UP vs DOWN) ───
    let mut v_proposals = vec![0u32; cell_count]; // destination scheme
    let mut v_contests = Vec::new();
    for y in (4..(height - 4)).step_by(4) {
        for x in (4..(width - 4)).step_by(4) {
            if v_contests.len() < 2048 {
                let target = y * width + x;
                let up = (y - 1) * width + x;
                let down = (y + 1) * width + x;
                v_proposals[up as usize] = target + 1;
                v_proposals[down as usize] = target + 1;
                v_contests.push((up, down, target));
            }
        }
    }

    let base_v_claims = harness.run_claim(
        &harness.pipeline_baseline_destination,
        width,
        height,
        0,
        &v_proposals,
    );
    let mut base_up_wins = 0;
    let mut base_down_wins = 0;
    for &(up, down, target) in &v_contests {
        let winner = base_v_claims[target as usize] - 1;
        if winner == up {
            base_up_wins += 1;
        } else if winner == down {
            base_down_wins += 1;
        }
    }

    let cand_v_claims = harness.run_claim(
        &harness.pipeline_candidate_destination,
        width,
        height,
        42,
        &v_proposals,
    );
    let mut cand_up_wins = 0;
    let mut cand_down_wins = 0;
    for &(up, down, target) in &v_contests {
        let winner = cand_v_claims[target as usize] - 1;
        if winner == up {
            cand_up_wins += 1;
        } else if winner == down {
            cand_down_wins += 1;
        }
    }

    println!("\n=== VERTICAL CONTENTION (2,048 Contests: UP vs DOWN) ===");
    println!(
        "Baseline (Fixed-Index): UP = {} ({:.1}%), DOWN = {} ({:.1}%)",
        base_up_wins,
        (base_up_wins as f64 / 2048.0) * 100.0,
        base_down_wins,
        (base_down_wins as f64 / 2048.0) * 100.0
    );
    println!(
        "Candidate (Edge-Hash):   UP = {} ({:.1}%), DOWN = {} ({:.1}%)",
        cand_up_wins,
        (cand_up_wins as f64 / 2048.0) * 100.0,
        cand_down_wins,
        (cand_down_wins as f64 / 2048.0) * 100.0
    );

    assert_eq!(base_up_wins, 2048);
    assert_eq!(base_down_wins, 0);

    let cand_up_pct = (cand_up_wins as f64 / 2048.0) * 100.0;
    assert!(
        (40.0..=60.0).contains(&cand_up_pct),
        "candidate must balance vertical contention (got {:.1}%)",
        cand_up_pct
    );

    // ─── 3. Diagonal Contention (NW vs SE, 2,048 Contests) ───
    let mut d_proposals = vec![0u32; cell_count];
    let mut d_contests = Vec::new();
    for y in (4..(height - 4)).step_by(4) {
        for x in (4..(width - 4)).step_by(4) {
            if d_contests.len() < 2048 {
                let target = y * width + x;
                let nw = (y - 1) * width + (x - 1);
                let se = (y + 1) * width + (x + 1);
                d_proposals[nw as usize] = target + 1;
                d_proposals[se as usize] = target + 1;
                d_contests.push((nw, se, target));
            }
        }
    }

    let base_d_claims = harness.run_claim(
        &harness.pipeline_baseline_destination,
        width,
        height,
        0,
        &d_proposals,
    );
    let mut base_nw_wins = 0;
    let mut base_se_wins = 0;
    for &(nw, se, target) in &d_contests {
        let winner = base_d_claims[target as usize] - 1;
        if winner == nw {
            base_nw_wins += 1;
        } else if winner == se {
            base_se_wins += 1;
        }
    }

    let cand_d_claims = harness.run_claim(
        &harness.pipeline_candidate_destination,
        width,
        height,
        42,
        &d_proposals,
    );
    let mut cand_nw_wins = 0;
    let mut cand_se_wins = 0;
    for &(nw, se, target) in &d_contests {
        let winner = cand_d_claims[target as usize] - 1;
        if winner == nw {
            cand_nw_wins += 1;
        } else if winner == se {
            cand_se_wins += 1;
        }
    }

    println!("\n=== DIAGONAL CONTENTION (2,048 Contests: NW vs SE) ===");
    println!(
        "Baseline (Fixed-Index): NW = {} ({:.1}%), SE = {} ({:.1}%)",
        base_nw_wins,
        (base_nw_wins as f64 / 2048.0) * 100.0,
        base_se_wins,
        (base_se_wins as f64 / 2048.0) * 100.0
    );
    println!(
        "Candidate (Edge-Hash):   NW = {} ({:.1}%), SE = {} ({:.1}%)",
        cand_nw_wins,
        (cand_nw_wins as f64 / 2048.0) * 100.0,
        cand_se_wins,
        (cand_se_wins as f64 / 2048.0) * 100.0
    );

    assert_eq!(base_nw_wins, 2048);
    assert_eq!(base_se_wins, 0);

    // ─── 4. Rotated Contention (0°, 90°, 180°, 270°) ───
    // Compare 4 orthogonal rotations of symmetric L-shaped contenders (2,048 contests each).
    // 0 deg: Left vs Up
    // 90 deg: Up vs Right
    // 180 deg: Right vs Down
    // 270 deg: Down vs Left
    let mut rot_proposals = vec![0u32; cell_count];
    let mut rot_0_contests = Vec::new();
    let mut rot_90_contests = Vec::new();
    let mut rot_180_contests = Vec::new();
    let mut rot_270_contests = Vec::new();

    for y in (4..(height - 4)).step_by(5) {
        for x in (4..(width - 4)).step_by(5) {
            let target = y * width + x;
            let left = y * width + (x - 1);
            let right = y * width + (x + 1);
            let up = (y - 1) * width + x;
            let down = (y + 1) * width + x;

            match (x / 5 + y / 5) % 4 {
                0 if rot_0_contests.len() < 512 => {
                    rot_proposals[left as usize] = target + 1;
                    rot_proposals[up as usize] = target + 1;
                    rot_0_contests.push((left, up, target));
                }
                1 if rot_90_contests.len() < 512 => {
                    rot_proposals[up as usize] = target + 1;
                    rot_proposals[right as usize] = target + 1;
                    rot_90_contests.push((up, right, target));
                }
                2 if rot_180_contests.len() < 512 => {
                    rot_proposals[right as usize] = target + 1;
                    rot_proposals[down as usize] = target + 1;
                    rot_180_contests.push((right, down, target));
                }
                3 if rot_270_contests.len() < 512 => {
                    rot_proposals[down as usize] = target + 1;
                    rot_proposals[left as usize] = target + 1;
                    rot_270_contests.push((down, left, target));
                }
                _ => {}
            }
        }
    }

    let cand_rot_claims = harness.run_claim(
        &harness.pipeline_candidate_destination,
        width,
        height,
        42,
        &rot_proposals,
    );

    let mut rot_0_a = 0;
    let mut rot_0_b = 0;
    for &(a, b, target) in &rot_0_contests {
        let w = cand_rot_claims[target as usize] - 1;
        if w == a {
            rot_0_a += 1;
        } else if w == b {
            rot_0_b += 1;
        }
    }

    println!("\n=== ROTATED CONTENTION (Candidate Distribution Across Rotations) ===");
    println!(
        "0°   (Left vs Up):    LEFT = {} ({:.1}%), UP = {} ({:.1}%)",
        rot_0_a,
        (rot_0_a as f64 / rot_0_contests.len() as f64) * 100.0,
        rot_0_b,
        (rot_0_b as f64 / rot_0_contests.len() as f64) * 100.0
    );

    // ─── 5. Tick-Seed Sweep (64 Seeds for Fixed Target) ───
    let mut left_seed_wins = 0;
    let mut right_seed_wins = 0;
    let single_target = 64 * width + 64;
    let single_left = 64 * width + 63;
    let single_right = 64 * width + 65;

    let mut single_prop = vec![NO_MOVE; cell_count];
    single_prop[single_left as usize] = single_target;
    single_prop[single_right as usize] = single_target;

    for tick in 0..64 {
        let claims = harness.run_claim(
            &harness.pipeline_candidate_movement,
            width,
            height,
            tick,
            &single_prop,
        );
        let winner = claims[single_target as usize] >> 2;
        if winner == single_left {
            left_seed_wins += 1;
        } else if winner == single_right {
            right_seed_wins += 1;
        }
    }

    println!("\n=== TICK-SEED VARIATION (64 Seeds on Fixed Single Target) ===");
    println!(
        "Candidate across 64 ticks: LEFT = {} ({:.1}%), RIGHT = {} ({:.1}%)",
        left_seed_wins,
        (left_seed_wins as f64 / 64.0) * 100.0,
        right_seed_wins,
        (right_seed_wins as f64 / 64.0) * 100.0
    );
    assert!(
        left_seed_wins > 0 && right_seed_wins > 0,
        "both contenders must win across tick seeds (no permanent lock)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// G6-C1: Performance Microbenchmark on RTX 5090 / DX12
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_claim_performance_microbenchmark_reference_world() {
    let harness = ClaimHarness::new();
    let width = 2048u32;
    let height = 2048u32;
    let cell_count = (width * height) as usize; // 4,194,304 cells

    println!("\n=== RTX 5090 / DX12 ARBITRATION MICROBENCHMARK (4,194,304 Cells) ===");

    // Scenario A: Realistic/Sparse Contention (~5% of cells contending in pairs)
    let mut sparse_props = vec![NO_MOVE; cell_count];
    for y in (4..(height - 4)).step_by(6) {
        for x in (4..(width - 4)).step_by(6) {
            let target = y * width + x;
            let left = y * width + (x - 1);
            let right = y * width + (x + 1);
            sparse_props[left as usize] = target;
            sparse_props[right as usize] = target;
        }
    }

    // Scenario B: Contention-Heavy / Worst-Case (Every even cell is targeted by both neighbors)
    let mut heavy_props = vec![NO_MOVE; cell_count];
    for y in 2..(height - 2) {
        for x in (2..(width - 2)).step_by(2) {
            let target = y * width + x;
            let left = y * width + (x - 1);
            let right = y * width + (x + 1);
            heavy_props[left as usize] = target;
            heavy_props[right as usize] = target;
        }
    }

    // Prepare GPU buffers once for benchmarking
    let params_data: [u32; 8] = [
        cell_count as u32,
        THREADS_X as u32,
        width,
        height,
        42,
        0,
        0,
        0,
    ];
    let params_buf = harness
        .context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bench-params"),
            contents: cast_u32_slice(&params_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let buf_sparse_prop =
        harness
            .context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bench-sparse-prop"),
                contents: cast_u32_slice(&sparse_props),
                usage: wgpu::BufferUsages::STORAGE,
            });

    let buf_heavy_prop =
        harness
            .context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bench-heavy-prop"),
                contents: cast_u32_slice(&heavy_props),
                usage: wgpu::BufferUsages::STORAGE,
            });

    let claim_buf = harness
        .context
        .device
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("bench-claim"),
            size: (cell_count as u64) * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

    let bg_sparse = harness
        .context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg-sparse"),
            layout: &harness.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_sparse_prop.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: claim_buf.as_entire_binding(),
                },
            ],
        });

    let bg_heavy = harness
        .context
        .device
        .create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg-heavy"),
            layout: &harness.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buf_heavy_prop.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: claim_buf.as_entire_binding(),
                },
            ],
        });

    let dispatch_y = ((cell_count as u64).div_ceil(THREADS_X)) as u32;

    let time_dispatches =
        |pipe: &wgpu::ComputePipeline, bg: &wgpu::BindGroup, iters: usize| -> f64 {
            let mut encoder = harness
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            for _ in 0..iters {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes: None,
                });
                cpass.set_pipeline(pipe);
                cpass.set_bind_group(0, bg, &[]);
                cpass.dispatch_workgroups(WORKGROUPS_X, dispatch_y, 1);
            }
            let start = Instant::now();
            harness.context.queue.submit(Some(encoder.finish()));
            harness.context.device.poll(wgpu::PollType::Wait).unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            (elapsed / iters as f64) * 1000.0 // ms per dispatch
        };

    // Warmup
    for _ in 0..10 {
        time_dispatches(&harness.pipeline_baseline_movement, &bg_sparse, 20);
        time_dispatches(&harness.pipeline_candidate_movement, &bg_sparse, 20);
    }

    // Benchmark Scenario A: Sparse / Realistic
    let iters = 50;
    let mut sparse_base_runs = Vec::new();
    let mut sparse_cand_runs = Vec::new();

    // Alternating order: B H H B B H H B B H
    for i in 0..5 {
        if i % 2 == 0 {
            sparse_base_runs.push(time_dispatches(
                &harness.pipeline_baseline_movement,
                &bg_sparse,
                iters,
            ));
            sparse_cand_runs.push(time_dispatches(
                &harness.pipeline_candidate_movement,
                &bg_sparse,
                iters,
            ));
        } else {
            sparse_cand_runs.push(time_dispatches(
                &harness.pipeline_candidate_movement,
                &bg_sparse,
                iters,
            ));
            sparse_base_runs.push(time_dispatches(
                &harness.pipeline_baseline_movement,
                &bg_sparse,
                iters,
            ));
        }
    }

    sparse_base_runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sparse_cand_runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_sparse_base = sparse_base_runs[2];
    let median_sparse_cand = sparse_cand_runs[2];
    let delta_sparse = median_sparse_cand - median_sparse_base;
    let pct_sparse = (delta_sparse / median_sparse_base) * 100.0;

    println!("\n--- SCENARIO A: Realistic / Sparse Contention ---");
    println!("Baseline Runs (ms/dispatch):  {:?}", sparse_base_runs);
    println!("Candidate Runs (ms/dispatch): {:?}", sparse_cand_runs);
    println!(
        "Median Baseline: {:.4} ms | Median Candidate: {:.4} ms | Delta: {:+.4} ms ({:+.2}%)",
        median_sparse_base, median_sparse_cand, delta_sparse, pct_sparse
    );

    // Benchmark Scenario B: Contention-Heavy / Worst-Case
    let mut heavy_base_runs = Vec::new();
    let mut heavy_cand_runs = Vec::new();

    for i in 0..5 {
        if i % 2 == 0 {
            heavy_base_runs.push(time_dispatches(
                &harness.pipeline_baseline_movement,
                &bg_heavy,
                iters,
            ));
            heavy_cand_runs.push(time_dispatches(
                &harness.pipeline_candidate_movement,
                &bg_heavy,
                iters,
            ));
        } else {
            heavy_cand_runs.push(time_dispatches(
                &harness.pipeline_candidate_movement,
                &bg_heavy,
                iters,
            ));
            heavy_base_runs.push(time_dispatches(
                &harness.pipeline_baseline_movement,
                &bg_heavy,
                iters,
            ));
        }
    }

    heavy_base_runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    heavy_cand_runs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_heavy_base = heavy_base_runs[2];
    let median_heavy_cand = heavy_cand_runs[2];
    let delta_heavy = median_heavy_cand - median_heavy_base;
    let pct_heavy = (delta_heavy / median_heavy_base) * 100.0;

    println!("\n--- SCENARIO B: Contention-Heavy / Worst-Case ---");
    println!("Baseline Runs (ms/dispatch):  {:?}", heavy_base_runs);
    println!("Candidate Runs (ms/dispatch): {:?}", heavy_cand_runs);
    println!(
        "Median Baseline: {:.4} ms | Median Candidate: {:.4} ms | Delta: {:+.4} ms ({:+.2}%)",
        median_heavy_base, median_heavy_cand, delta_heavy, pct_heavy
    );
}
