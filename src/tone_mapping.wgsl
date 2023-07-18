@group(0) @binding(0)
var hdr_render_target: texture_2d<f32>;

@group(0) @binding(1)
var hdr_render_target_sampler: sampler;

@group(0) @binding(2)
var<uniform> tone_mapping_enabled: u32;

@group(0) @binding(3)
var<storage, read> saturating_luminance: f32;

fn reinhard(in: vec3<f32>) -> vec3<f32> {
  return in / (vec3<f32>(1.0) + in);
}

// Source: http://filmicworlds.com/blog/why-a-filmic-curve-saturates-your-blacks/
fn duiker_approx(in: vec3<f32>) -> vec3<f32> {
  let x = max(vec3<f32>(0.0), in - vec3<f32>(0.004));
  return
    pow(
      x * (6.2 * x + vec3<f32>(0.5))
      /
      (x * (6.2 * x + vec3<f32>(1.7)) + vec3<f32>(0.06)),
      vec3<f32>(2.2)
    );
}

@vertex
fn vertex_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
  return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fragment_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let hdr_render_target_dimensions = textureDimensions(hdr_render_target);
  
  let hdr_texel: vec4<f32> = textureSampleLevel(
    hdr_render_target,
    hdr_render_target_sampler,
    vec2<f32>(
      position.x / f32(hdr_render_target_dimensions.x),
      position.y / f32(hdr_render_target_dimensions.y)
    ),
    0.0
  );

  var tonemapped_rgb: vec3<f32>;
  if tone_mapping_enabled == 1u {
    let normalised_rgb = hdr_texel.rgb / saturating_luminance;
    
    tonemapped_rgb = duiker_approx(normalised_rgb);
    // tonemapped_rgb = reinhard(normalised_rgb);
  } else {
    tonemapped_rgb = hdr_texel.rgb;
  }

  return vec4<f32>(tonemapped_rgb, hdr_texel.a);
}