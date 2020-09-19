* Camera UBO to fix the aspect ratio (Ortho camera)
* Animation uniform
* Add line rendering capability to the renderer. Are we going to use strokefiller?
    * Yes! But we're going to use the Ctor's &mut in order to count our distance along the line.
* Shaders shouldn't have an inColor attribute, they should have an inValue attribute.
    * They will use this to animate paths using the animation uniform
* Animation ideas:
    * For the lines, in the vertex shader we'll use the Value attribute and take the cosine of it before we pass it into the vertex shader. This way, it's interpolated across the lines. Cosine is to make areas of the lines that will come together. When we threshold the value in the fragment shader, we will add a bias for X + Y (start from top left and go to the right). So information passed to the fragment shader is [x, y, value]
