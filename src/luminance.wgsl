@group(0) @binding(0)
var hdr_render_target: texture_2d<f32>;

@group(0) @binding(1)
var hdr_render_target_sampler: sampler;

@group(0) @binding(2)
var<uniform> total_luminance_pixels_per_thread: u32;

@group(0) @binding(3)
var<storage, read_write> total_luminance_intermediate: array<f32>;

@group(0) @binding(4)
var<storage, read_write> average_luminance: f32;

@group(0) @binding(5)
var<storage, read_write> auto_EV100: f32;

@group(0) @binding(6)
var<storage, read_write> saturating_luminance: f32;

// @group(1) @binding(0)
// var ldr_render_target: texture_storage_2d<rgba8unorm, write>;

const LUMINANCE_COEFFICIENTS: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

@compute @workgroup_size(256)
fn calculate_total_luminance_intermediate(
  @builtin(local_invocation_id) local_id: vec3<u32>
) {
  let hdr_render_target_dimensions = vec2<u32>(1918u, 2088u); // textureDimensions(hdr_render_target);
  let hdr_texels = hdr_render_target_dimensions.x * hdr_render_target_dimensions.y;

  let starting_index: u32 = local_id.x * total_luminance_pixels_per_thread;
  
  var total = 0.0;
  for (var i: u32 = starting_index; i < starting_index + total_luminance_pixels_per_thread; i++) {
    if i < hdr_texels {
      let hdr_texel = textureSampleLevel(
        hdr_render_target,
        hdr_render_target_sampler,
        vec2<f32>(
          f32(i % hdr_render_target_dimensions.x) / f32(hdr_render_target_dimensions.x),
          f32(i / hdr_render_target_dimensions.x) / f32(hdr_render_target_dimensions.y)
        ),
        0.0
      );
      total += dot(LUMINANCE_COEFFICIENTS, hdr_texel.rgb);
    }
  }
  total_luminance_intermediate[local_id.x] = total;
}

/* Recommended EV for a scene's average luminance.

`EV = log_2(L_avg * (S / K))`[^1]

```
EV
= log_2(L_avg * (S / K))
= log_2(L_avg) + log_2(S / K))
= log_2(L_avg) + log_2(S) - log_2(K)
```

For `S = 100` (ISO100 speed) and `K = 12.5` (commonly-used calibration constant[^2]):

```
log_2(L_avg) + log_2(100) - log_2(12.5)
= log_2(L_avg) + 3
```

[^1]: <https://en.wikipedia.org/wiki/Exposure_value#Relationship_of_EV_to_lighting_conditions>
[^2]: <https://en.wikipedia.org/wiki/Light_meter#Calibration_constants>
*/
fn average_luminance_to_EV100(l_avg: f32) -> f32 {
  return log2(max(l_avg, 0.00001)) + 3.0;
}

/* The amount of luminance required to saturate our ISO100 "sensor".

Saturation-based speed: `S_sat = 78/H_sat`[^1]. `H_sat` is the luminance exposure that will saturate
the sensor. Rearrange: `H_sat = 78/S_sat`. `S_sat = 100` for ISO100 film speed. `H_sat = 0.78`.

Luminous exposure: `H = qL * (t / N^2)`[^2] (`q` is a lens transmittance factor, `L` is incoming luminance, `t` is exposure time, `N` is the aperture f-number, )
Exposure value: `EV = log_2(N^2 / t)`[^3]

Luminous exposure in terms of incoming luminance and EV:

```
H
= qL * (t / N^2)
= qL / (N^2 / t)
= qL / 2^EV
```

Incoming luminance that will cause ISO100 sensor to saturate, in terms of EV.

```
q * L_sat / 2^EV = 0.78
L_sat = (0.78 / q) * 2^EV
```

[^1]: <https://en.wikipedia.org/wiki/Film_speed#Saturation-based_speed>
[^2]: <https://en.wikipedia.org/wiki/Film_speed#Measurements_and_calculations>
[^3]: <https://en.wikipedia.org/wiki/Exposure_value#Formal_definition>

*/
fn saturating_luminance_EV100(ev: f32) -> f32 {
  return 1.2 * pow(2.0, ev);
}

@compute @workgroup_size(1)
fn calculate_average_luminance() {
  let hdr_render_target_dimensions = textureDimensions(hdr_render_target);
  let hdr_texels = hdr_render_target_dimensions.x * hdr_render_target_dimensions.y;
  
  var total_luminance = 0.0;
  for (var i: u32 = 0u; i < arrayLength(&total_luminance_intermediate); i++) {
    total_luminance += total_luminance_intermediate[i];
  }
  
  average_luminance = total_luminance / f32(hdr_texels);
  // auto_EV100 = 14.6; // "sunny 16" EV100
  auto_EV100 = average_luminance_to_EV100(average_luminance);
  saturating_luminance = saturating_luminance_EV100(auto_EV100);
}