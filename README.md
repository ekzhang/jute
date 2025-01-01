# Jute

Jute is a native notebook for interactive computing.

Double-click to open any Jupyter notebook in a beautiful, streamlined desktop
app. Run code in 40 languages both locally, and on powerful cloud kernels (with
GPUs). Collaborate with real-time multiplayer.

Jute is also designed to integrate modern features: code completion, semantic
highlighting, simple standalone kernels, automatic formatting, and AI.

(This is a complete rewrite of the Jupyter frontend for speed, simplicity, and
usability.)

> [!IMPORTANT]
>
> Jute is not usable or fully implemented yet; I just feel the need to write
> down and continuously evolve an aspiration of where I'm going.

## Why?

Notebooks are critical to modern data science, education, and research. They
should be a first-class document type that _feels effortless_.

How effortless? No fiddling around with `pip install`, slow load times, insecure
browser contexts, port-forwarding, setting up kernels, extensions that never
seem to work, `jupyter lab build`, obscure menus, or notebook checkpoints that
clog up Git.

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

Jute is specifically designed to work as a native app. That means file
management is left to the operating system; we're not going to embed a
half-functional windowing system, folder viewer, or file editor.

In exchange, you'll get an application that starts up instantly, with a heavy
focus on the developer experience. You and the computer — as a thinking tool.

### Design principles

1. **The kernel as a window.** Every running kernel gets its own notebook
   window. When you close a notebook, the kernel is terminated. No wasted
   resources.
2. **What you need to see.** If a UI element is unnecessary, we're removing it.
   (Why does Jupyter Lab have those tabs to the left? What's "Command" mode? The
   eternal notification icon at the bottom right? The "Activate Next Tab Bar"
   button?) Meanwhile, we make it easier to access important elements like
   restarting kernels, CPU and RAM usage, and Markdown.
3. **Intelligent tools.** Features like autocompletion / go-to-definition (LSP),
   hover to see docstrings, and real-time collaboration should "just work" by
   default. It's a pain to configure these for Jupyter (so many errors!), and it
   should really be easier.
4. **Aesthetic minimalism.** Jute should be beautiful. But it should also be
   minimal, so you can focus on getting things done without distractions. Think
   of a new file in a code editor — a blank slate for creativity.

### Related work

The Jupyter project is in widespread use and has a vibrant open-source
ecosystem. Jute does not aim to reproduce _all_ features of Jupyter, only the
most frequently used ones. The goal of Jute is to reimagine notebook design, so
some elements may be simplified to emphasize more important user flows.

These existing projects take different approaches, but still may be of interest
to you:

- [JupyterLab Desktop](https://github.com/jupyterlab/jupyterlab-desktop) —
  Official Jupyter Lab desktop application, based on Electron.
- [VS Code Jupyter extension](https://github.com/Microsoft/vscode-jupyter) —
  Notebook editor inside VS Code.
- [nbterm](https://github.com/davidbrochart/nbterm) — Terminal user interface
  for Jupyter.
- [Juno](https://apps.apple.com/us/app/juno-jupyter-python-ide/id1462586500?platform=ipad)
  — Python notebook editor for iPhone and iPad.

In most cases Jute is simpler, more streamlined, and faster than alternatives,
but it may be less compatible with the existing Jupyter ecosystem.

## Technical

Tauri, React, Rust.

Making an alternate frontend is only possible due to the moumental engineering
effort of the Jupyter Project.

## Author

- [Eric Zhang](https://www.ekzhang.com/)
  ([@ekzhang1](https://twitter.com/ekzhang1))
