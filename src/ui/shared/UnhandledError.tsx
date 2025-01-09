import { ArrowLeft } from "lucide-react";
import { Link } from "wouter";

type Props = {
  error?: string;
  onReturn?: () => void;
};

export function UnhandledError({ error, onReturn }: Props) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-2 p-6 text-center">
      <h3 className="mb-4 text-3xl">An error occured</h3>

      <p className="mb-4 max-w-md">
        It looks like there was an unhandled issue. Please{" "}
        <a
          target="_blank"
          href="https://github.com/ekzhang/jute/issues"
          className="underline"
        >
          file an issue
        </a>{" "}
        and include this message in your report:
      </p>

      <pre className="mb-4 max-w-lg select-text">
        {error || "An unknown error occurred."}
      </pre>

      <Link
        to="/"
        className="flex items-center gap-2 rounded-md border px-3 py-2 transition-colors hover:border-black"
        onClick={onReturn}
      >
        <ArrowLeft size="1em" />
        Take me back home
      </Link>
    </div>
  );
}
