import { useLocation } from "wouter";

export default function NotFoundPage() {
  const [location] = useLocation();

  return (
    <div className="flex h-screen flex-col items-center justify-center text-center">
      <h1 className="mb-4 text-3xl">Invalid path</h1>
      <p className="mb-4 max-w-md">
        It looks we've made a mistake! Please{" "}
        <a
          target="_blank"
          href="https://github.com/ekzhang/jute/issues"
          className="underline"
        >
          file an issue
        </a>{" "}
        and include this URL in your report:
      </p>
      <pre className="mb-4 min-w-40 max-w-lg select-text overflow-y-auto rounded-md bg-gray-200 p-4 font-medium">
        <code>{location}</code>
      </pre>
    </div>
  );
}
