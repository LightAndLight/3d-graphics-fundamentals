# 2023-07-08 Realtime 3D Lighting Fundamentals

## Goals

* Relearn GPU programming
* Render simple 3D objects in perspective
  * [x] Basic render pass
  * [x] 2D rendering
  * [x] Camera and perspective
  * [x] `.obj` model rendering
  * [ ] Indexed rendering
  * [ ] Per-object transformation matrix
* Light objects using physically-based techniques
* Add shadows via shadow mapping
* Use GPU-driven techniques as much as possible

## Stretch Goals

* Ambient occlusion
* High dynamic range
* Bloom

## Resources

* GPU programming fundamentals
  * <https://zdgeier.com/wgpuintro.html> - `wgpu` / `WGSL`
  * <https://sotrh.github.io/learn-wgpu/> - `wgpu` / `WGSL`
  * <https://vkguide.dev/> - `vulkan` / `GLSL`
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