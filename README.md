# FFT Visualization

A live Web Demo might be available [here](https://fft.maltesparenb.org/).

## Guide
On the left sidebar in the top you can add sin/cos functions and change their amplitude, frequency and y-shift.
On the bottom you can change the number of samples and the input signal range used for the FFT.

On the right you can see three plots. The top one contains all the separate functions. The middle one displays the combined wave of the given functions (the sum of the amplitude at that point). The bottom plot displays the output of the FFT.

> The egui_plot's I've used are not really configurable. Therefore depending on the plot you are navigating you can see the y-scaling on the other plots change as well. This causes some other plots to look like a single flat line. If you switch the plot you can double left click with your mouse to reset to the correct scaling.

## Algorithm
I used the recursive version of the Cooley-Tuckey FFT. In my implementation its not optimized at all, but it works.

## Development
If you want to compile the project yourself you can either use the native version via `cargo run` or the wasm version with `trunk serve`. If any problems occur, have a look into the eframe template repo from emilk.