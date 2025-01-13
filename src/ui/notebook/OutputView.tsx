import { encode } from "html-entities";
import { memo } from "react";

import { MultilineString, OutputDisplayData } from "@/bindings";
import { CellResult } from "@/stores/notebook";

type Props = {
  value: CellResult | undefined;
};

export default function OutputView({ value }: Props) {
  if (!value) {
    return null;
  }
  const outputs = value.outputs ?? [];
  return (
    <div className="select-text whitespace-pre-wrap break-words px-8 pb-6 pt-4 text-sm after:contents">
      {outputs.map((output, index) => (
        <div key={index}>
          {output.output_type === "stream" ? (
            <pre>{multiline(output.text)}</pre>
          ) : output.output_type === "display_data" ? (
            <OutputViewDisplayData output={output} />
          ) : output.output_type === "execute_result" ? (
            <OutputViewDisplayData output={output} />
          ) : output.output_type === "error" ? (
            // TODO: Display error tracebacks.
            <pre className="text-red-500">
              {output.ename}: {output.evalue}
            </pre>
          ) : null}
        </div>
      ))}
    </div>
  );
}

function multiline(source: MultilineString): string {
  if (typeof source === "string") {
    return source;
  }
  return source.join("");
}

const OutputViewDisplayData = memo(
  ({ output }: { output: OutputDisplayData }) => {
    const html = displayDataToHtml(output.data, output.metadata);

    if (html) {
      return <div dangerouslySetInnerHTML={{ __html: html }}></div>;
    } else {
      return null;
    }
  },
);

/**
 * Returns the HTML form of a display data message.
 *
 * https://jupyter-client.readthedocs.io/en/stable/messaging.html#display-data
 */
function displayDataToHtml(
  data: Record<string, any>,
  metadata: Record<string, any>,
): string | null {
  for (const imageType of [
    "image/png",
    "image/jpeg",
    "image/svg+xml",
    "image/bmp",
    "image/gif",
  ]) {
    if (Object.hasOwn(data, imageType)) {
      const value = data[imageType];
      const alt = String(data["text/plain"] ?? "");
      const meta = metadata[imageType];
      if (typeof value === "string") {
        let image = `<img src="data:${imageType};base64,${encode(value)}" alt="${encode(alt)}"`;
        if (meta) {
          if (typeof meta.height === "number" && meta.height > 0) {
            image += ` height="${meta.height}"`;
          }
          if (typeof meta.width === "number" && meta.width > 0) {
            image += ` width="${meta.width}"`;
          }
        }
        image += " />";
        return image;
      }
    }
  }

  const value = data["text/plain"];
  if (typeof value === "string") {
    return `<pre>${encode(value)}</pre>`;
  } else if (Array.isArray(value)) {
    return `<pre>${value.join("")}</pre>`;
  }

  return null;
}
