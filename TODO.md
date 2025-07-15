UI/UX:
- [ ] Reload the last quicksave on startup
- [ ] Warn when scaling on integral yields inf
- [ ] It should not be possible to load the same spectrum twice (into the same group!)
- [x] ~~Keyboard controls in search menu: arrow down should not go below number of search hits~~
- [x] ~~Implement plotting/hiding of groups accessible with switch
      buttons on plotting UI~~
- [x] Include legend
  - [x] ~~Showing filename~~
  - [x] ~~Showing alias~~
- [x] ~~Select good colors for plots based on FileID~~
- [.] Implement file settings form
  - [x] ~~Use collapsing header for file settings form~~
  - [x] ~~Allow alias for file~~
  - [ ] Allow selection of plot color
  - [x] ~~Implement scaling/shifting of plots (form based and mouse control)~~
  - [ ] Implement plotting of columns different than first two
    - [ ] Allow plotting more than one dataset from CSV file
- [x] Create Global Settings View
- [ ] Include error log window 
- [ ] Files in file settings view should have a fixed order
- [.] Allow to attach meta data to files
  - [x] Add arbitrary comments
- [x] ~~Allow saving/loading to/from arbitrary path~~

- [x] ~~Allow moving files between groups~~

Search:
- [x] ~~Do not block the UI when opening new search path via file dialog~~
- [x] Allow selection using keyboard up/down in search modal

Plotter:
- [ ] Have a flag to disable/enable the legend
- [ ] Allow custom x/y tick positions when exporting
- [ ] Allow custom color for each plot

Internals:
- [x] ~~Use array of length 10 for groups instead of hashmap~~
- [x] ~~Handle UI interactions using an event system?~~
- [x] Remove hard coded paths
  - [.] Work with configuration file
- [x] CSV with repeated delimiter (whitespace) should parse correctly
- [.] Parse Bruker files
  - Fails for some files, needs debugging

BUGS:
- [x] ~~Removing one file from group may may delete all files~~
- [x] ~~CSV parsing can panic~~
- [x] ~~spectral integration currently not working with OPUS files, probably because of the inversed x-axis~~
- [ ] CSV module ignores last data row in CSV files?
