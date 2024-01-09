# Jute

Jute is a native notebook for interactive computing.

Double-click to open any Jupyter file in a beautiful, streamlined desktop app.
Run code in 40 languages on powerful remote kernels, and collaborate with
real-time multiplayer.

> [!IMPORTANT]
>
> Nothing here is actually implemented yet, I just feel the need to write down
> an aspirational view of where I'm going.

## Why?

Notebooks are tremendously important to modern data science, education, and
research. They should be a first-class document type with excellent editors that
_feel effortless_.

How effortless? No fiddling around with `pip install`, slow load times, insecure
browser contexts, port-forwarding, setting up kernels, `jupyter lab build`,
unnecessary context menus, or forgetting to hit "Save".

I just want to write code interactively, and to share interactive documents.

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

## Technical

Tauri, React, Rust.

Making an alternate frontend is only possible due to the excellent engineering
work of the Jupyter Project to build its software in composable, well-specified
parts.

## Author

- [Eric Zhang](https://www.ekzhang.com/)
  ([@ekzhang1](https://twitter.com/ekzhang1))
