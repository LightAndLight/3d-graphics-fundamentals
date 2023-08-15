// Originally defined in `render_hrs.wgsl:Camera`.
struct Camera{
  eye: vec3<f32>,
  zfar: f32,
  view_proj: mat4x4<f32>,
  view_proj_inv: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(0) @binding(1)
var sky_texture: texture_2d<f32>;

@group(0) @binding(2)
var sky_texture_sampler: sampler;

@group(0) @binding(3)
var<uniform> sky_intensity: f32;

@vertex
fn vertex_main(@location(0) position: vec2<f32>) -> VertexOutput {
  let view_from = camera.view_proj_inv * vec4<f32>(0.0, 0.0, 0.0, 1.0);
  let view_to = camera.view_proj_inv * vec4<f32>(position, 1.0, 1.0);

  var output: VertexOutput;
  output.position = vec4<f32>(position, 0.0, 1.0);
  output.view_direction = (view_to.xyz / view_to.w) - (view_from.xyz / view_from.w);
  return output;
}

struct VertexOutput{
  @builtin(position) position: vec4<f32>,
  @location(0) view_direction: vec3<f32>
}

const TAU: f32 = 6.2831;
const PI: f32 = 3.1415;

// Given a direction vector, sample from a texture containing an equirectangular projection of a sphere.
fn direction_to_uv_equirectangular(direction: vec3<f32>) -> vec2<f32> {
  /* Based on this conversion: <https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates>

  +Y axis is up, +Z is forward, and rotating toward +X gives a positive azimuth.

  Azimuth (`phi` in the link) ranges from `-pi` to `pi`. Polar angle from `0` to `pi`.
  */
  let azimuth = sign(direction.x) * acos(direction.z / length(direction.zx));
  let polar_angle = acos(direction.y);
  
  let spherical_coords = vec2<f32>(azimuth, polar_angle);

  /* Given the above conversion, the top-left of the texture corresponds to `(azimuth, polar_angle) = (2pi, 0)`.
  Center of texture: `(pi, pi/2)`. Bottom-right: `(0, pi)`. Convert this space into UV coordinates.
  */ 
  return (spherical_coords * vec2<f32>(-1.0, 1.0) + vec2<f32>(PI, 0.0)) / vec2<f32>(TAU, PI);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  /* Assume the surface of the skybox is infinitely far away. I think then I can consider
  the camera as living in at the center of the unit sphere, looking outward. Then the patch
  of sky to render for this fragment is determined by the direction of a ray going from the eye position,
  through the center of the fragment.
  */

  return 
    vec4<f32>(vec3<f32>(sky_intensity), 1.0) *
    textureSample(
      sky_texture,
      sky_texture_sampler,
      direction_to_uv_equirectangular(normalize(input.view_direction))
    );
}