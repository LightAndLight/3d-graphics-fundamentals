/*
struct ObjectId{value: u32}

I can't have `VertexInput.object_id : ObjectId` because `wgpu` reports a type mismatch.
The `object_id` field needs to have type `u32` because that's the only type I can name
in the vertex buffer layout.

I'd also like to write this but apparently I'm not allowed to pass the array as an argument.

fn get_object(objects: array<ObjectData>, object_id: ObjectId) -> ObjectData {
  return objects[object_id.value];
}
*/

struct VertexInput{
  @location(0) position: vec3<f32>,
  @location(1) color: vec4<f32>,
  @location(2) object_id: u32
}

struct VertexOutput{
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>
}

@group(0) @binding(0)
var<uniform> camera_to_clip: mat4x4<f32>;

struct ObjectData{
  transform: mat4x4<f32>
}

@group(0) @binding(1)
var<storage, read> objects: array<ObjectData>;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  
  output.position = camera_to_clip * objects[input.object_id].transform * vec4<f32>(input.position, 1.0);
  output.color = input.color;

  return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  return input.color;
}