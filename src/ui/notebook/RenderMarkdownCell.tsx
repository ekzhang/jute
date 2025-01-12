import { Edit3Icon } from "lucide-react";
import Markdown from "react-markdown";

import styles from "./RenderMarkdownCell.module.css";

type Props = {
  source: string;
  onStartEdit?: () => void;
};

/** Display rendered Markdown in a cell. */
export default function RenderMarkdownCell({ source, onStartEdit }: Props) {
  return (
    <div
      className="group relative ml-14 max-w-full overflow-hidden py-2 pr-4 text-sm"
      onDoubleClick={onStartEdit}
    >
      <Markdown
        className={styles.markdown}
        components={{
          a: ({ ...props }) => (
            <a {...props} target="_blank" rel="noreferrer" />
          ),
        }}
      >
        {source}
      </Markdown>

      <button
        className="absolute right-2 top-2 flex items-center rounded text-gray-500 opacity-0 transition-all hover:text-black group-hover:opacity-100"
        onClick={onStartEdit}
      >
        <Edit3Icon size={16} className="mr-1" />
        Edit
      </button>
    </div>
  );
}
