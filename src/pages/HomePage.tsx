import { open } from "@tauri-apps/plugin-dialog";
import { ArrowRight } from "lucide-react";
import { Link, useLocation } from "wouter";

const sampleNotebookNames = [
  "~/Numpy_starter.ipynb",
  "~/Data_analysis.ipynb",
  "~/Machine_learning.ipynb",
];

export default function HomePage() {
  const [, navigate] = useLocation();
  return (
    <div className="mt-20 flex flex-col gap-4 px-8 font-light">
      <h1 className="mt-2 text-5xl">Welcome to Jute</h1>

      <h2 className="text-xl text-gray-500">
        A notebook for interactive computing.
      </h2>

      <div className="flex gap-4">
        {sampleNotebookNames.map((name, i) => (
          <Link key={i} to={"/notebook?" + new URLSearchParams({ path: name })}>
            <div
              key={name}
              className="flex h-60 w-48 items-end bg-gray-300 p-2 transition-all hover:bg-gray-400"
            >
              {name}
            </div>
          </Link>
        ))}
      </div>

      <button
        className="flex w-fit items-center gap-2"
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
  );
}
