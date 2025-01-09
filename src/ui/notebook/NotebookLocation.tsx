import { FolderOpenIcon } from "lucide-react";

type Props = {
  directory: string;
  filename: string | null;
};

export default function NotebookLocation({ directory, filename }: Props) {
  let shortDirectory: string;
  // If there are at least three slashes in the directory, shorten it to .../a/b/c
  if (directory.split("/").length > 3) {
    const parts = directory.split("/");
    shortDirectory = ".../" + parts.slice(-3).join("/");
  } else {
    shortDirectory = directory;
  }

  let basename: string;
  let extension: string;
  if (!filename) {
    basename = "Untitled notebook";
    extension = "";
  } else if (filename.includes(".")) {
    const parts = filename.split(".");
    extension = parts.pop()!;
    basename = parts.join(".");
  } else {
    basename = filename;
    extension = "";
  }

  return (
    <div className="ml-14">
      <div className="flex items-center text-gray-500">
        <FolderOpenIcon size={16} className="mr-2" />
        <code className="pt-0.5 text-xs">{shortDirectory}</code>
      </div>
      {/* TODO: Alternate shortened display if the notebook already has an <h1> heading. */}
      <h1 className="mt-2 text-3xl">
        {basename}
        <span className="text-gray-300">.{extension}</span>
      </h1>
    </div>
  );
}
