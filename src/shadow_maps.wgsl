struct Light{
  shadow_view: mat4x4<f32>,
  shadow_projection: mat4x4<f32>,
  shadow_map_atlas_position: vec2<f32>,
  shadow_map_atlas_size: vec2<f32>
}

@group(0) @binding(0)
var<uniform> light: Light;

struct ObjectData{
  transform: mat4x4<f32>
}

@group(0) @binding(1)
var<storage, read> objects: array<ObjectData>;

struct VertexInput{
  @location(0) position: vec3<f32>,
  @location(1) object_id: u32,
  @location(2) normal: vec3<f32>,
  @location(3) material_id: u32
}

@vertex
fn vertex_main(input: VertexInput) -> @builtin(position) vec4<f32> {
  let object = objects[input.object_id];
  return
    light.shadow_projection *
    light.shadow_view *
    object.transform *
    vec4<f32>(input.position, 1.0);
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @builtin(frag_depth) f32 {
  return position.z;
}