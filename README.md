# 2023-07-08 Realtime 3D Lighting Fundamentals

## Goals

* Remember GPU programming
  * [x] Basic render pass
  * [x] 2D rendering
* Render simple 3D objects in perspective
  * [x] Camera and perspective
  * [x] `.obj` model rendering (vertex positions)
  * [x] Per-object transformation matrix
  * [x] Depth testing
  * [x] Compute vertex normals when missing from `.obj` files
* Shade objects using physically-based techniques
  * [x] Include vertex normals
  * [x] Punctual and directional lights with diffuse reflectance
  * [x] Specular reflectance for dielectric materials
  * [x] Per-object material parameters (color, roughness)
  * [x] Metallic materials
  * [x] Physical light units, manual exposure
  * [x] Tone mapping
  * [x] Automatic exposure using average scene luminance
* Shadow mapping
  * [x] Basic shadow map for directional lights
  * [x] Omnidirectional shadow mapping (point lights)
  * [ ] Light frustum fitting
* Environment mapping / image-based lighting
  * [x] HDRI skybox
* Use GPU-driven techniques as much as possible
  * [x] Per-object transformation matrices stored in a single GPU buffer
  * [ ] `draw_indirect`

## Stretch goals

* [ ] Anti-aliasing
* [ ] Ambient occlusion
* [ ] Bloom
* [ ] Area lights
* [ ] Histogram-based auto-exposure
* [ ] Parallax occlusion mapping
* [ ] Virtual/adaptive shadow maps

## For fun

* [ ] Parse-less on-disk model format with a program that converts to / from `.obj`
* [ ] Render the depth buffer on screen
* Performance improvements
  * [ ] Indexed draws

  To load meshes straight from `mmap`ped files.

## Resources

* Graphics programming on GPUs
  * <https://zdgeier.com/wgpuintro.html> - `wgpu` / `WGSL`
  * <https://sotrh.github.io/learn-wgpu/> - `wgpu` / `WGSL`
  * <https://vkguide.dev/> - `vulkan` / `GLSL`
  * <https://learnopengl.com/> `opengl` / `GLSL`
* Normal calculation
  * <https://computergraphics.stackexchange.com/questions/4031/programmatically-generating-vertex-normals>
  * <https://iquilezles.org/articles/normals/>
* Physically based shading
  * <https://interplayoflight.wordpress.com/2013/12/30/readings-on-physically-based-rendering/> -
    PBS literature master list 
  * <https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-24-importance-being-linear>
  * <https://renderwonk.com/blog/index.php/archive/adventures-with-gamma-correct-rendering/https://renderwonk.com/blog/index.php/archive/adventures-with-gamma-correct-rendering/>
  * <https://blog.selfshadow.com/publications/> - SIGGRAPH's "Physically Based Shading in Theory and
    Practise" series
    * <https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf> -
      Very clear introduction to physically based shading fundamentals
  * <https://www.realtimerendering.com/>
    * Chapter 9 - Physically Based Shading
  * <https://google.github.io/filament/Filament.html> - explanations of physically based rendering
    in the context of Google's [Filament](https://google.github.io/filament/) engine
  * Production implementations
    * Blender - <https://github.com/blender/blender/blob/main/source/blender/draw/engines/eevee/shaders/bsdf_common_lib.glsl>
    * Disney - <https://github.com/wdas/brdf/blob/main/src/brdfs/disney.brdf>
* HDR rendering
  * Exposure
    * <https://en.wikipedia.org/wiki/Film_speed>
    * <https://en.wikipedia.org/wiki/Exposure_(photography)>
    * <https://en.wikipedia.org/wiki/Exposure_value>
    * <https://en.wikipedia.org/wiki/Sunny_16_rule>
    * <https://seblagarde.wordpress.com/2015/07/14/siggraph-2014-moving-frostbite-to-physically-based-rendering/>
    * <https://placeholderart.wordpress.com/2014/11/21/implementing-a-physically-based-camera-manual-exposure/>
  * Tone mapping
    * <https://seenaburns.com/dynamic-range/>
    * <https://64.github.io/tonemapping/>
    * <http://filmicworlds.com/blog/filmic-tonemapping-operators/>
    * <http://filmicworlds.com/blog/why-a-filmic-curve-saturates-your-blacks/>
    * <http://filmicworlds.com/blog/filmic-tonemapping-with-piecewise-power-curves/>
    * <http://duikerresearch.com/2015/09/filmic-tonemapping-for-real-time-rendering/>
* Shadow mapping
  * <https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping>
  * <https://developer.nvidia.com/gpugems/gpugems/part-ii-lighting-and-shadows/chapter-12-omnidirectional-shadow-mapping>
    * Cube map face selection
      * <https://stackoverflow.com/questions/6980530/selecting-the-face-of-a-cubemap-in-glsl>
      * <https://www.gamedev.net/forums/topic/687535-implementing-a-cube-map-lookup-function/5337472/>
  * Projection fitting
    * <https://learn.microsoft.com/en-us/windows/win32/dxtecharts/common-techniques-to-improve-shadow-depth-maps#techniques-to-improve-shadow-maps>
    * <https://gamedev.stackexchange.com/questions/73851/how-do-i-fit-the-camera-frustum-inside-directional-light-space>
* Depth buffer precision / logarithmic depth buffers
  * <https://outerra.blogspot.com/2009/08/logarithmic-z-buffer.html>
  * <https://www.gamedev.net/blog/73/entry-2006307-tip-of-the-day-logarithmic-zbuffer-artifacts-fix/>
  * <https://outerra.blogspot.com/2012/11/maximizing-depth-buffer-range-and.html>
  * <https://outerra.blogspot.com/2013/07/logarithmic-depth-buffer-optimizations.html>
  * <http://web.archive.org/web/20201113123351/https://thxforthefish.com/posts/reverse_z/>
* Test models
  * <https://github.com/alecjacobson/common-3d-test-models>
  * <http://graphics.stanford.edu/data/3Dscanrep/>
  * <https://www.cc.gatech.edu/projects/large_models/index.html>

    (change `www-static` to `www` on downloads)
  * [Utah teapot](https://en.wikipedia.org/wiki/Utah_teapot)
    * <https://graphics.stanford.edu/courses/cs148-10-summer/as3/code/as3/teapot.obj>
    * <https://graphics.cs.utah.edu/courses/cs6620/fall2013/prj05/teapot.obj>
  * [Stanford bunny](https://en.wikipedia.org/wiki/Stanford_bunny)
    * <https://graphics.stanford.edu/~mdfisher/Data/Meshes/bunny.obj>
* `.obj` parser: <https://crates.io/crates/tobj>