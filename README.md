# Flugs »

Comparing spectra with ease.

Flugs (German _quickly_) is a viewer application for spectral data (or other x-y
data in CSV format). It's main goal is to allow plotting and comparing spectra
as fast as possible.

## Features:

- Fuzzy search spectra to quickly add them to the current session
- Preview spectra in search menu
- Scale and shift spectra
- Sort spectra into groups
- Show/hide groups
- Integrate signals and scale on integrals
- (Quick) save and load session
- Load CSV and Bruker OPUS files
- Export plot to SVG

## Usage:

A list of keyboard shortcuts is available by pressing F1.

CTRL + Space brings up search menu. Start typing to recursivly search through
the selected root folder. Separate search patterns by spaces to match them
anywhere in the file name.

Use the mouse or up/down keys to highlight a file. If the highlighted file can
be loaded as a spectrum, you will see a preview. Press a number 0 to 9 to select
the spectrum for a certain group. Press enter to accept your selection.

Double click the spectrum to zoom out to show all. Click and hold drags
the view. The mouse wheel zooms.

In the default "Display Plots" mode (press F4 or click "Mode" menu to switch
modes), click a spectrum to select it. While holding SHIFT, click and drag the
mouse to scale the spectrum along the y-axis. Clicking and dragging the mouse
while holding CTRL will shift the spectrum along the y-axis, while holding ALT
will shift along the x-axis. When right clicking, the scale/offsets can be set
precisely in the context menu.

In the "Integration" mode, click and drag the mouse to define the integration
window. Right click to open the context window to read off integrals (hover the
file names) or scaling on a single spectrum (click filename) or all currently
visible spectra (click "All").

# Configuration File

#TODO




