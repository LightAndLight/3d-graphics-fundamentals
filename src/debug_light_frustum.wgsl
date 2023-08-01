@group(0) @binding(0)
var<uniform> shadow_view_inverse: mat4x4<f32>;

// Originally defined at `render_hdr.wgsl:Camera`.
struct Camera{
  eye: vec3<f32>,
  zfar: f32,
  view_proj: mat4x4<f32>,
  view_proj_inv: mat4x4<f32>
}

@group(0) @binding(1)
var<uniform> camera: Camera;

struct VertexInput{
  @location(0) position: vec3<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> @builtin(position) vec4<f32> {
  return
    camera.view_proj *
    shadow_view_inverse *
    vec4<f32>(input.position, 1.0);
}

struct FragmentOutput{
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> FragmentOutput {
  var output: FragmentOutput; 
  output.depth = log2(max(1e-6, 1.0 / position.w)) * (1.0 / log2(camera.zfar + 1.0));
  output.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
  return output;
}