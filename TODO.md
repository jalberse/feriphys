TODO:
* Add to forms.rs code to get a cube with the face' normals inward (i.e. fix the vertex positions to get that result).
   Note we only gotta change the vertex declaration or indices ordering - our normal calculation will get the correct normals from that.
* Draw the cube and the ball, with the cube inside the ball.
* Get the ball moving without any physics, move it around in a circle or something.
   We'd want to follow this loosely, I think, for the model transformation matrix: https://www.youtube.com/watch?v=9SsjhrxH08o
   Also useful: https://www.braynzarsoft.net/viewtutorial/q16390-33-instancing-with-indexed-primitives
   We need a new transform.rs with a way to get a model transformation matrix given some position, rotation, etc.
   From there, we can either:
   1. Create a uniform buffer for our model's transformation. I kind of dislike this because it doesn't take advantage of our instancing capabilities.
   2. Use queue::write_buffer to update our instance buffer with the model's transformation matrix.
      https://github.com/gfx-rs/wgpu/discussions/1438 here, kvark describes the pseudo-code behind write_buffer. It uses a staging buffer and copies it in.
      We'd want to build out a system such that we know the offset of the index buffer to ensure we e.g. update the ball's instance data, not the cube's.
      That can be simple for now, but as we get into particle systems, we want that to be more robust to handle arbitrary instances (up to some max allowable,
      since the index buffer is of a constant size we can't overflow. Though I wonder how large that can be - enough for 10k, 500k particles?)
      This also is less efficient than staging_belt, but I don't want to do that more complex implementation if not necessary.
* Add physics in for the bouncing ball
* (Optional) since I'm standing up this instancing framework, it might be possible to render *many* bouncing balls in their own cubes around the scene,
  and we could fly around to them. Only if it's trivial with how I set up instancing and such. Otherwise, get extra points in other ways.