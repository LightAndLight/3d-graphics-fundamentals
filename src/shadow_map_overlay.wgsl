struct Camera{
  eye: vec3<f32>,
  view_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct ObjectData{
  transform: mat4x4<f32>
}

@group(0) @binding(1)
var<storage, read> objects: array<ObjectData>;

struct DirectionalLight{
  color: vec4<f32>,
  direction: vec3<f32>,
  illuminance: f32
}

@group(0) @binding(2)
var<storage, read> directional_lights: array<DirectionalLight>;

struct Material{
  color: vec4<f32>,
  roughness: f32,
  metallic: f32,
}

@group(0) @binding(3)
var shadow_map_atlas: texture_depth_2d;

@group(0) @binding(4)
var shadow_map_atlas_sampler: sampler_comparison;

struct ShadowingDirectionalLight{
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  shadow_map_atlas_position: vec2<f32>,
  shadow_map_atlas_size: vec2<f32>
}

@group(0) @binding(5)
var<storage, read> shadowing_directional_lights: array<ShadowingDirectionalLight>;

struct VertexInput{
  @location(0) position: vec3<f32>,
  @location(1) object_id: u32,
  @location(2) normal: vec3<f32>,
  @location(3) material_id: u32
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;

  let world_position = objects[input.object_id].transform * vec4<f32>(input.position, 1.0);
  output.world_position = world_position.xyz / world_position.w;
  output.position = camera.view_proj * world_position;

  // Nothing told me that I was forgetting to attach normals!
  // The normal can only get passed through for translations.
  output.normal = input.normal;

  return output;
}

struct VertexOutput{
  @builtin(position) position: vec4<f32>,
  @location(0) world_position: vec3<f32>,
  @location(1) normal: vec3<f32>,
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  let view_direction = normalize(camera.eye - input.world_position);

  // the interpolated vertex normals won't be normalised.
  let surface_normal = normalize(input.normal);

  var color: vec4<f32>;  
  for (var i: u32 = 0u; i < arrayLength(&directional_lights); i++) {
    let directional_light = directional_lights[i];
    let shadowing_directional_light = shadowing_directional_lights[i];

    let light_direction: vec3<f32> = -directional_light.direction; 
    
    let fragment_light_space = 
      shadowing_directional_light.projection *
      shadowing_directional_light.view *
      vec4<f32>(input.world_position, 1.0);
    let fragment_depth = fragment_light_space.z / fragment_light_space.w;

    let shadow_map_atlas_dimensions = vec2<f32>(textureDimensions(shadow_map_atlas));
    
    let shadow_map_entry_start_uv =
      shadowing_directional_light.shadow_map_atlas_position / shadow_map_atlas_dimensions;
    
    let shadow_map_entry_size_uv =
      shadowing_directional_light.shadow_map_atlas_size / shadow_map_atlas_dimensions;

    // The fragment's light space position mapped to (0..1, 0..1)
    let fragment_shadow_map_atlas_entry_space =
      (fragment_light_space.xy * vec2<f32>(1.0, -1.0) + vec2<f32>(1.0))
      /
      vec2<f32>(2.0);
    
    let shadow_map_offset_uv =
      shadow_map_entry_size_uv * fragment_shadow_map_atlas_entry_space;

    let shadow_map_atlas_entry_texel_border =
      trunc(fragment_shadow_map_atlas_entry_space * shadowing_directional_light.shadow_map_atlas_size)
      /
      shadowing_directional_light.shadow_map_atlas_size;

    if abs(fragment_shadow_map_atlas_entry_space.x - shadow_map_atlas_entry_texel_border.x) < 0.00005 || abs(fragment_shadow_map_atlas_entry_space.y - shadow_map_atlas_entry_texel_border.y) < 0.00005 {
      color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    }
  }
  
  return color;
}