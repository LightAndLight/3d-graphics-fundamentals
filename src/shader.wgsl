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

  // Nothing told me that I was forgetting to attach normals!
  // The normal can only get passed through for translations.
  output.normal = input.normal;

  if display_normals == 1u {
    output.color = vec4<f32>(output.normal, 1.0);
  } else {
    output.color = input.color;
  }

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

fn diffuse_brdf(albedo: vec3<f32>, _light_direction: vec3<f32>, _view_direction: vec3<f32>) -> vec3<f32> {
  return albedo / PI;
}

fn schlick(light_direction: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  let f0 = vec3<f32>(0.04);
  return f0 + (1.0 - f0) * pow(1.0 - max(dot(normal, light_direction), 0.0), 5.0);
}

fn is_positive(x: f32) -> f32 {
  if x <= 0.0 {
    return 0.0;
  } else {
    return 1.0;
  }
}

fn distribution(alpha: f32, normal: vec3<f32>, half_vector: vec3<f32>) -> f32 {
  let alpha_squared = alpha * alpha;
  let n_dot_h = max(dot(normal, half_vector), 0.00001);
  let n_dot_h_2 = n_dot_h * n_dot_h;
  return 
    alpha_squared * is_positive(n_dot_h)
    /
    (PI * pow(n_dot_h, 4.0) * pow(alpha_squared + (1.0 / n_dot_h_2 - 1.0), 2.0));
}

fn g1(alpha: f32, normal: vec3<f32>, v: vec3<f32>, h: vec3<f32>) -> f32 {
  let n_dot_v = max(dot(normal, v), 0.00001);
  let n_dot_v_2 = n_dot_v * n_dot_v;
  let alpha_squared = alpha * alpha;
  return 
    is_positive(dot(v, h) / n_dot_v) * 2.0
    /
    (1.0 + sqrt(1.0 + alpha_squared * (1.0 / n_dot_v_2 - 1.0)));
}

fn geometry(alpha: f32, normal: vec3<f32>, light_direction: vec3<f32>, view_direction: vec3<f32>, h: vec3<f32>) -> f32 {
  return g1(alpha, normal, light_direction, h) * g1(alpha, normal, view_direction, h);
}

/* `distribution` and `geometry` are the GGX microfacet normal distribution and geometric attenuation functions
straight out of [1]. The original GGX functions are written in terms of `cos` and `tan^2`, which can be calculated
using the dot product of unit vectors, and `1 / dot(...)^2 - 1` (`tan^2(theta) = 1 / cos^2(theta) - 1`).

Versions that appear in the wild, such as in <http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html>,
are optimised and written to avoid unnecessary divides-by-zero, so they take a slightly different form.

[1]: Walter, B., Marschner, S. R., Li, H., & Torrance, K. E. (2007, June).
    Microfacet models for refraction through rough surfaces.
    In Proceedings of the 18th Eurographics conference on Rendering Techniques (pp. 195-206).
*/

fn brdf(normal: vec3<f32>, albedo: vec3<f32>, roughness: f32, light_direction: vec3<f32>, view_direction: vec3<f32>) -> vec3<f32> {
  let half_vector = normalize(light_direction + view_direction);
  
  let alpha = pow(roughness, 2.0);

  let f = schlick(light_direction, half_vector);
  let g = geometry(alpha, normal, light_direction, view_direction, half_vector);
  let d = distribution(alpha, normal, half_vector);
  let specular = 
    f * g * d
    / 
    (4.0 * max(dot(normal, light_direction), 0.00001) * max(dot(normal, view_direction), 0.00001));

  let diffuse = (1.0 - f) * diffuse_brdf(albedo, light_direction, view_direction);

  return diffuse + specular;
}

fn srgb_to_linear_scalar(srgb: f32) -> f32 {
  if srgb <= 0.04045 {
    return srgb / 12.92;
  } else {
    return pow((srgb + 0.055) / 1.055, 2.4);
  }
}

fn srgb_to_linear(srgb: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(
    srgb_to_linear_scalar(srgb.r),
    srgb_to_linear_scalar(srgb.g),
    srgb_to_linear_scalar(srgb.b),
    srgb.a
  );
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  if display_normals == 1u {
    return input.color;
  } else {
    var radiance: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    let roughness = 0.4;
 
    let view_direction = normalize(camera.eye - input.world_position);

    let albedo = srgb_to_linear(input.color);

    // the interpolated vertex normals won't be normalised.
    let surface_normal = normalize(input.normal);
      
    for (var i: u32 = 0u; i < arrayLength(&point_lights); i++) {
      let point_light = point_lights[i];

      // TODO: don't recalculate this for every fragment.
      let point_light_position: vec4<f32> = objects[point_light.object_id].transform * vec4<f32>(0.0, 0.0, 0.0, 1.0);

      let light_direction: vec3<f32> = normalize((point_light_position.xyz / point_light_position.w) - input.world_position); 
      let distance_to_light = length(light_direction);

      let light_color = srgb_to_linear(point_light.color);

      radiance +=
        PI *
        brdf(
          surface_normal,
          albedo.rgb,
          roughness,
          // normalising these directions is important
          light_direction,
          view_direction
        ) *
        light_color.rgb * falloff(point_light.intensity, distance_to_light) *
        max(dot(surface_normal, light_direction), 0.0);
    }
    
    for (var i: u32 = 0u; i < arrayLength(&directional_lights); i++) {
      let directional_light = directional_lights[i];

      let light_direction: vec3<f32> = -directional_light.direction; 
      
      let light_color = srgb_to_linear(directional_light.color);

      radiance +=
        PI *
        brdf(surface_normal, albedo.rgb, roughness, light_direction, view_direction) *
        light_color.rgb *
        max(dot(surface_normal, light_direction), 0.0);
    }
    
    return vec4<f32>(radiance, input.color.a);
  }
}