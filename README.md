Feriphyis is a work in progress.

# Feriphys

Feriphys is a collection of physics simulations, and visualizations of those simulations,
written in Rust.

The visualizations are written using wgpu.

# TODO

Set up the GUI to actually control the bouncing ball simulation properly, to familiarize myself
with egui.
See gui_window.rs for that. Then we'd use self.gui_window.ui(&self.platform.context());
instead of where I make the window currently.
Then, yea, some input handling shenanigans.
To share, we might have some Trait called GUI and each of our simulations will implement the GUI trait.
It would have render() and handle_events(), both of which would have default implementations I guess, since I don't think it would change
    (besdies a different GuiWindow field, but that could be passed in or use different type that implement a GuiWindow trait or something)
Then they could each implement their own get_simulation_state_mut() and each simulation would implement a sync_state_from_gui().

Clean up lib.rs - cluttered. Move some stuff into their own modules or existing modules as appropriate.
Consider what can be re-used.

Then, squirel away the current lib.rs into some new BouncingBallSim.rs file (top level) which holds its own event
loop and stuff.
Change the current lib.rs to open that based on some command line argument.
Add a new ParticlesSim.rs (or something) to start our particle sim.
Begin development on ParticlesSim.rs. Anything that *can* be abstracted out of BouncingBallSim.rs
to be shared, should be, if it's easy.