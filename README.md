# Pixel Sorter

This program implements several variations of the original pixel sorting algorithm in Rust. It also provides an easy way to chain various pixel sorts together and execute them sequentially.

### What is pixel sorting?

From [this website](http://satyarth.me/articles/pixel-sorting/):
>Pixel sorting is an interesting, glitchy effect which selectively orders the pixels in the rows/columns of an image. It was popularized (possibly invented) by artist Kim Asendorf.

Here are some examples of what pixel sorting can produce (all created with this program):

<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/classic/output.jpg" width="600" height="600" />
<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/classic/output2.jpg" width="600" height="600" />
<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/classic/output3.jpg" width="600" height="600" />
<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/kernel/output.jpg" width="600" height="600" />
<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/kernel/output2.jpg" width="600" height="600" />
<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/vector/output.jpg" width="600" height="600" />

### Installation

Download `pixelsorter.exe` from the latest release, which should appear in the sidebar on the right. Pixelsorter is a command line application.

### Usage

Open Powershell, navigate to the executable and run `./pixelsorter.exe shell`. This starts an interactive shell where pixelsorter commands can be run.

### Commands

The `open` and `save` commands are used to open and save images. This pixelsorter has 1 buffer which all commands operate on. 

Load the buffer:
> `open input.jpg`

Save the buffer:
> `save output.jpg`

This makes it easier to efficiently chain sorts together, since saving the image is quite an expensive operation. Commands can also be written in a file, and then run with `./pixelsorter.exe -f commands.txt`. See the `examples/` folder for more.

##### Sorting

This pixelsorter has 3 sorts available: classic, kernel, and vector. Each mode requires a certain amount of arguments, all of which must be supplied. For example, the `classic` sort takes 4 arguments: vertical, reverse, reverse_threshold and threshold. The first three are booleans while the last is an int. Therefore, a classic sort command could look like this:
`classic false false false 120`. This is the command which generaearthted the first image in the examples above from this image of Earth:

<img src="https://github.com/rishabh-bector/pixelsorter/blob/master/examples/classic/input.jpg" width="600" height="600" />

A description of all the commands and their arguments:

##### Classic
The classic pixel sort. Creates intervals in either the vertical or horizontal direction (`vertical`) based on pixel brightness being over or under (`reverse_threshold`) a certain threshold (`threshold`). Sorts those intervals based on pixel brightness, and then places them back in decreasing or increasing order (`reverse`). See [this website](http://satyarth.me/articles/pixel-sorting/) for a better explanation of the algorithm.

`classic <vertical: bool> <reverse: bool> <reverse_threshold: bool> <threshold: uint>`

Example:
> `classic false false true 150` 

##### Kernel

Creates a grid of rectangular intervals based on the size (`numx`, `numy`), sorts the pixels in those intervals based on brightness, and then places them back in decreasing or increasing order (`reverse`).

`kernel <reverse: bool> <reverse_threshold: bool> <threshold: uint> <numx: uint> <numy: uint>`

Example:
> `kernel false false 120 50 50`

##### Vector

Creates intervals based on a certain vector field described by `expression`. By default the  n intervals (`amount`) will all start along the left y edge. The `spacing` defines the spacing between each group of intervals of size h, where h is the height of the image in pixels. This allows one to create a grid of starting intervals--see the example below, where size = spacing. The rest of the interval is then defined by steps taken based on the value of the vector field at that point. This behavior can get quite glitchy.

`vector <reverse: bool> <size: uint> <amount: uint> <spacing: uint> <expression: unspaced string>`

Example:
> `vector false 30 15000 30 sin((x*y)/100)`