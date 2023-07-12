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

@group(0) @binding(2)
var<uniform> display_normals: u32; // Apparently booleans aren't host-mappable?

struct PointLight{
  object_id: u32,
  color: vec4<f32>,
  intensity: f32
}

@group(0) @binding(3)
var<storage, read> point_lights: array<PointLight>;

struct DirectionalLight{
  color: vec4<f32>,
  direction: vec3<f32>
}

@group(0) @binding(4)
var<storage, read> directional_lights: array<DirectionalLight>;

struct VertexInput{
  @location(0) position: vec3<f32>,
  @location(1) color: vec4<f32>,
  @location(2) object_id: u32,
  @location(3) normal: vec3<f32>
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;

  let world_position = objects[input.object_id].transform * vec4<f32>(input.position, 1.0);
  output.world_position = world_position.xyz / world_position.w;
  output.position = camera.view_proj * world_position;

  if display_normals == 1u {
    output.color = vec4<f32>(input.normal, 1.0);
  } else {
    output.color = input.color;
  }

  // Nothing told me that I was forgetting to attach normals!
  output.normal = input.normal;

  return output;
}

struct VertexOutput{
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) world_position: vec3<f32>
}

const PI: f32 = 3.14159;

fn falloff(intensity: f32, distance: f32) -> f32 {
  return pow(intensity / distance, 2.0);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  if display_normals == 1u {
    return input.color;
  } else {
    var radiance: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
 
    let view_direction = camera.eye - input.world_position;
      
    for (var i: u32 = 0u; i < arrayLength(&point_lights); i++) {
      let point_light = point_lights[i];

      // TODO: don't recalculate this for every fragment.
      let point_light_position: vec4<f32> = objects[point_light.object_id].transform * vec4<f32>(0.0, 0.0, 0.0, 1.0);

      let light_direction: vec3<f32> = (point_light_position.xyz / point_light_position.w) - input.world_position; 
      let distance_to_light = length(light_direction);

      /*
      radiance +=
        PI *
        (albedo / PI) * // Lambertian BRDF
        point_light.color * falloff(distance_to_light) *
        max(dot(input.normal, light_direction), 0.0);
      
      Simplifies to:
      */
      radiance +=
        input.color.rgb *
        point_light.color.rgb * falloff(point_light.intensity, distance_to_light) *
        max(dot(input.normal, light_direction), 0.0);
    }
    
    for (var i: u32 = 0u; i < arrayLength(&directional_lights); i++) {
      let directional_light = directional_lights[i];

      let light_direction: vec3<f32> = -directional_light.direction; 

      radiance +=
        input.color.rgb *
        directional_light.color.rgb *
        max(dot(input.normal, light_direction), 0.0);
    }
    
    return vec4<f32>(radiance, input.color.a);
  }
}