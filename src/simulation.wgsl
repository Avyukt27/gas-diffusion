struct Uniforms {
    delta: f32,
    diffusion_coefficient: f32,
    width: u32,
    height: u32,
    cell_size: f32,
}

struct Cell {
    concentration: f32,
    advection_x: f32,
    advection_y: f32,
    wall: u32,
}

@group(0) @binding(0) var<storage, read> input_grid: array<Cell>;
@group(0) @binding(1) var<storage, read_write> output_grid: array<Cell>;
@group(0) @binding(2) var<storage, read> sources: array<f32>;
@group(0) @binding(3) var<storage, read_write> pressures: array<f32>;
@group(0) @binding(4) var<storage, read_write> divergences: array<f32>;
@group(0) @binding(5) var<uniform> uniforms: Uniforms;

fn get_idx(x: u32, y: u32) -> u32 {
    return y * uniforms.width + x;
}

fn sample_field(x: i32, y: i32, current_idx: u32, field_type: u32) -> f32 {
    if x < 0 || x > i32(uniforms.width) || y < 0 || y >= i32(uniforms.height) {
        return 0.0;
    }

    let target_idx = get_idx(u32(x), u32(y));
    if input_grid[target_idx].wall == 1u {
        if field_type == 0u {
            return input_grid[current_idx].concentration;
        } else {
            return pressures[current_idx];
        }
    }

    if field_type == 0u {
        return input_grid[target_idx].concentration;
    } else {
        return pressures[target_idx];
    }
}

@compute @workgroup_size(16, 16)
fn compute_divergence(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if x <= 0u || x >= uniforms.width - 1u || y <= 0u || y >= uniforms.height - 1u { return; }

    let idx = get_idx(x, y);
    let u = input_grid[idx].advection_x;
    let v = input_grid[idx].advection_y;

    var u_left = 0.0;
    if input_grid[get_idx(x - 1u, y)].wall == 0u { u_left = input_grid[get_idx(x - 1u, y)].advection_x; }

    var v_up = 0.0;
    if input_grid[get_idx(x, y - 1u)].wall == 0u { v_up = input_grid[get_idx(x, y - 1u)].advection_y; }

    divergences[idx] = (u - u_left) / uniforms.cell_size + (v - v_up) / uniforms.cell_size;
}

@compute @workgroup_size(16, 16)
fn solve_pressures(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if x == 0u || x >= uniforms.width - 1u || y == 0u || y >= uniforms.height - 1u { return; }

    let idx = get_idx(x, y);
    if input_grid[idx].wall == 1u { return; }

    let p_left = sample_field(i32(x) - 1, i32(y), idx, 1u);
    let p_right = sample_field(i32(x) + 1, i32(y), idx, 1u);
    let p_up = sample_field(i32(x), i32(y) - 1, idx, 1u);
    let p_down = sample_field(i32(x), i32(y) + 1, idx, 1u);

    var neighbor_sum = 0.0;
    var fluid_count = 0.0;

    if input_grid[get_idx(x - 1u, y)].wall == 0u { neighbor_sum += p_left;  fluid_count += 1.0; }
    if input_grid[get_idx(x + 1u, y)].wall == 0u { neighbor_sum += p_right; fluid_count += 1.0; }
    if input_grid[get_idx(x, y - 1u)].wall == 0u { neighbor_sum += p_up;    fluid_count += 1.0; }
    if input_grid[get_idx(x, y + 1u)].wall == 0u { neighbor_sum += p_down;  fluid_count += 1.0; }

    if fluid_count > 0.0 {
        pressures[idx] = (neighbor_sum - uniforms.cell_size * uniforms.cell_size * divergences[idx]) / fluid_count;
    }
}

@compute @workgroup_size(16, 16)
fn subtract_gradient(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if x >= uniforms.width || y >= uniforms.height { return; }

    let idx = get_idx(x, y);

    output_grid[idx].wall = input_grid[idx].wall;
    output_grid[idx].concentration = input_grid[idx].concentration;
    output_grid[idx].advection_x = input_grid[idx].advection_x;
    output_grid[idx].advection_y = input_grid[idx].advection_y;

    if x < uniforms.width - 1u {
        if input_grid[get_idx(x + 1u, y)].wall == 0u {
            output_grid[idx].advection_x -= (pressures[get_idx(x + 1u, y)] - pressures[idx]) / uniforms.cell_size;
        } else {
            output_grid[idx].advection_x = 0.0;
        }
    }

    if y < uniforms.height - 1u {
        if input_grid[get_idx(x, y + 1u)].wall == 0u {
            output_grid[idx].advection_y -= (pressures[get_idx(x, y + 1u)] - pressures[idx]) / uniforms.cell_size;
        } else {
            output_grid[idx].advection_y = 0.0;
        }
    }
}

@compute @workgroup_size(16, 16)
fn advect_diffusion(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let y = id.y;
    if x >= uniforms.width || y >= uniforms.height { return; }

    let idx = get_idx(x, y);
    if input_grid[idx].wall == 1u {
        output_grid[idx].concentration = 0.0;
        return;
    }

    let adv_x = input_grid[idx].advection_x;
    let adv_y = input_grid[idx].advection_y;

    let trace_x = f32(x) - uniforms.delta * adv_x;
    let trace_y = f32(y) - uniforms.delta * adv_y;

    let x0 = u32(clamp(floor(trace_x), 0.0, f32(uniforms.width - 1u)));
    let x1 = u32(clamp(ceil(trace_x), 0.0, f32(uniforms.width - 1u)));
    let y0 = u32(clamp(floor(trace_y), 0.0, f32(uniforms.height - 1u)));
    let y1 = u32(clamp(ceil(trace_y), 0.0, f32(uniforms.height - 1u)));

    let s1 = trace_x - floor(trace_x);
    let t1 = trace_y - floor(trace_y);

    let c00 = input_grid[get_idx(x0, y0)].concentration;
    let c10 = input_grid[get_idx(x1, y0)].concentration;
    let c01 = input_grid[get_idx(x0, y1)].concentration;
    let c11 = input_grid[get_idx(x1, y1)].concentration;

    let advected_concentration = mix(mix(c00, c10, s1), mix(c01, c11, s1), t1);

    let c_left = sample_field(i32(x) - 1, i32(y), idx, 0u);
    let c_right = sample_field(i32(x) + 1, i32(y), idx, 0u);
    let c_up = sample_field(i32(x), i32(y) - 1, idx, 0u);
    let c_down = sample_field(i32(x), i32(y) + 1, idx, 0u);

    var neighbor_sum = 0.0;
    var fluid_count = 0.0;

    if input_grid[get_idx(x - 1u, y)].wall == 0u { neighbor_sum += c_left; } else { neighbor_sum += advected_concentration; }
    if input_grid[get_idx(x + 1u, y)].wall == 0u { neighbor_sum += c_right; } else { neighbor_sum += advected_concentration; }
    if input_grid[get_idx(x, y - 1u)].wall == 0u { neighbor_sum += c_up; } else { neighbor_sum += advected_concentration; }
    if input_grid[get_idx(x, y + 1u)].wall == 0u { neighbor_sum += c_down; } else { neighbor_sum += advected_concentration; }

    let diffusion = uniforms.diffusion_coefficient * uniforms.delta * (neighbor_sum - 4.0 * advected_concentration) / (uniforms.cell_size * uniforms.cell_size);

    let final_concentration = advected_concentration + diffusion + sources[idx];
    output_grid[idx].concentration = clamp(final_concentration, 0.0, 1.0);
}
