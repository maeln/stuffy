# stuffy

## it's stuff

Stuffy is my little tool to quickly do experiment with shader.

Just run `cargo run` and run.

Since this is a pathtracer, the longer you let it run, the more the picture will converge (be less noisy).

## Control

You can move the camera around by clicking and dragging the mouse and with `W A S D` (sorry azerty user, no time to handle this properly).
The mouse wheel control the aperture (bigger aperture = more bokeh blur / depth of field), and clicking with the right button will set the focus
to whatever is on the cursor when clicked (note: since a bigger aperture will distort the image, focus the right object with a big aperture can be tricky).

To get the parameter of the camera, press `I` and it will show on the terminal. `R` will reset the image, holding `P` pause the rendering, `O` display the current number of sample rendered, and `V` zero-out the aperture as long as it is pressed.
