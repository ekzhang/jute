import { open } from "@tauri-apps/plugin-dialog";
import { ArrowRight } from "lucide-react";
import { Link, useLocation } from "wouter";

import { NotebookRoot } from "@/bindings";
import Header from "@/ui/shared/Header";

const simpleNotebook = {
  cells: [
    {
      cell_type: "markdown",
      metadata: {},
      source: [
        "## A simple notebook\n",
        "\n",
        "This is a simple notebook to get you started! Run cells with `Shift+Enter`.",
      ],
    },
    {
      cell_type: "code",
      execution_count: null,
      metadata: {},
      outputs: [],
      source: ['print("Hello, world!")'],
    },
    {
      cell_type: "code",
      execution_count: null,
      metadata: {},
      outputs: [],
      source: [
        "for i in range(100):\n",
        "    if i % 15 == 0:\n",
        '        print("FizzBuzz")\n',
        "    elif i % 3 == 0:\n",
        '        print("Fizz")\n',
        "    elif i % 5 == 0:\n",
        '        print("Buzz")\n',
        "    else:\n",
        "        print(i)",
      ],
    },
    {
      cell_type: "code",
      execution_count: null,
      metadata: {},
      outputs: [],
      source: [
        "import matplotlib.pyplot as plt\n",
        "import numpy as np\n",
        "\n",
        "plt.plot(np.random.randn(200))",
      ],
    },
  ],
  metadata: {
    kernelspec: {
      display_name: "Python 3",
      language: "python",
      name: "python3",
    },
    language_info: {
      name: "python",
      version: "3.13.0",
    },
  },
  nbformat: 4,
  nbformat_minor: 2,
} as NotebookRoot;

export default function HomePage() {
  const [, navigate] = useLocation();
  return (
    <div className="h-screen overflow-y-auto">
      <Header />
      <div className="px-8 py-20">
        <h1 className="mb-2.5 text-4xl">Welcome to Jute</h1>

        <h2 className="text-lg text-gray-400">
          A native notebook for interactive computing.
        </h2>

        <div className="my-8 flex gap-4">
          <Link
            to={
              "/notebook?" +
              new URLSearchParams({ inline: JSON.stringify(simpleNotebook) })
            }
            className="flex h-60 w-48 flex-col justify-end rounded border border-gray-300 p-4 transition-colors hover:border-black"
          >
            Getting started
          </Link>
        </div>

        <button
          className="flex items-center gap-2 hover:underline"
          onClick={async () => {
            const file = await open({
              multiple: false,
              directory: false,
              filters: [{ name: "Jupyter Notebook", extensions: ["ipynb"] }],
            });
            if (file)
              navigate("/notebook?" + new URLSearchParams({ path: file }));
          }}
        >
          Open a notebook <ArrowRight size="1em" />
        </button>
      </div>
    </div>
  );
}
