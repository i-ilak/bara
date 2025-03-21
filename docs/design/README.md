# Design doc

`bara` is the tool I use to build my website. The key features that I need:

- convert `markdown` to `html`,
- process the header of the markdown files:
  - title
  - date
  - description
  - tags
  - map information
- enable '`time-machine`', i.e. a way to see all past versions of the website
  - this also includes `archiving` the current state of the website
- Create a config file to provide some flexibility in naming locations
- Provide `scss` support

## Work context

To work, we will need 3 things:

1. Options passed when `bara` was started. We will use [`clap`](https://lib.rs/crates/clap) for this, since it features everything we need. For now, a rather simplistic setup with three command line options should be enough.

   - `build (default; optional)`: one run that will build the website and output all relevant files into a target location, as specified in the config file, so that the website could afterwards be served.
   - `archive`: should archive the current state of the site into the `time_machine` so that for subsequent builds
     it appears as a selection option in the dropdown menu.
   - `watch`: hot-reload mode, or something similar, i.e. should be used for development. When a change is detected
     in a subset of files, e.g. all the markdown files, css files, etc. re-run build.

1. Abstraction of the config file.

1. A working directory in which we can perform most of the work.
