@vertex
fn vertex_main(@location(0) vertex_position: vec3<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(
    vertex_position,
    1.0
  );
}

@fragment
fn fragment_main(@builtin(position) fragment_position: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(0.1, 0.2, 0.3, 1.0);
}