# Jute

Jute is a native notebook for interactive computing.

Double-click to open any Jupyter file in a beautiful, streamlined desktop app.
Run code in 40 languages on powerful remote kernels, and collaborate with
real-time multiplayer.

> [!IMPORTANT]
>
> Nothing here is actually implemented yet; I just feel the need to write down
> and continuously evolve an aspiration of where I'm going.

## Why?

Notebooks are critical to modern data science, education, and research. They
should be a first-class document type that _feels effortless_.

How effortless? No fiddling around with `pip install`, slow load times, insecure
browser contexts, port-forwarding, setting up kernels, `jupyter lab build`,
unnecessary context menus, or forgetting to hit "Save".

I just want to **write code interactively**, and to **share interactive
documents**.

> Jupyter notebooks remain the best option for exploratory data analysis,
> reproducible documents, sharing of results, tutorials, etc.
>
> – [Jake VanderPlas](https://twitter.com/jakevdp/status/1046757277133230080)

> The Notebook system is designed around two central ideas: (a) an openly
> specified protocol to control an interactive computational engine, and (b) an
> equally open format to record these interactions between the user and the
> computational engine, including the results produced by the computations.
>
> – [K. Jarrod Millman and Fernando Pérez](https://osf.io/h9gsd)

### Related work

The Jupyter project is in widespread use and has a vibrant open-source
ecosystem. Jute does not aim to reproduce _all_ features of Jupyter, only the
most frequently used ones. The goal of Jute is to reimagine notebook design, so
some elements may be simplified to emphasize more important user flows.

These existing projects take different approaches to interface design, but still
may be of interest to you:

- [JupyterLab Desktop](https://github.com/jupyterlab/jupyterlab-desktop) —
  Official Jupyter Lab desktop application, based on Electron.
- [VS Code Jupyter extension](https://github.com/Microsoft/vscode-jupyter) —
  Notebook editor inside VS Code.
- [nbterm](https://github.com/davidbrochart/nbterm) — Terminal user interface
  for Jupyter.

In most cases Jute is simpler, more streamlined, and faster than alternatives,
but it is less compatible with the existing Jupyter ecosystem.

## Technical

Tauri, React, Rust.

Making an alternate frontend is only possible due to the excellent engineering
effort of the Jupyter Project to build its software in composable,
well-specified parts.

## Author

- [Eric Zhang](https://www.ekzhang.com/)
  ([@ekzhang1](https://twitter.com/ekzhang1))
