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
  * [ ] Compute vertex normals when missing from `.obj` files
* Light objects using physically-based techniques
  * [x] Include vertex normals
  * [x] Punctual and directional lights with diffuse reflectance
  * [ ] Specular reflectance for punctual and directional lights (not quite working)
* [ ] Self-shadowing
* [ ] Add shadows via shadow mapping
* Use GPU-driven techniques as much as possible
  * [x] Per-object transformation matrices stored in a single GPU buffer
  * [ ] `draw_indirect`

## Stretch goals

* [ ] Ambient occlusion
* [ ] High dynamic range
* [ ] Bloom
* [ ] Area lights

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
* Physically based shading
  * <https://blog.selfshadow.com/publications/>
    * <https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf>
  * <https://www.realtimerendering.com/>
    * Chapter 9
  * Realtime area lights: <https://eheitzresearch.wordpress.com/415-2/>
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