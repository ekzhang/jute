import { ArrowLeft } from "lucide-react";
import { Link } from "wouter";

type Props = {
  error?: string;
};

export function Error(props: Props) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-2">
      <h3 className="text-3xl">An error occured.</h3>

      <p className="text-lg">{props.error || "An unknown error occurred."}</p>

      <Link to="/">
        <button className="mt-4 flex items-center gap-2">
          <ArrowLeft size="1em" />
          Go back to home
        </button>
      </Link>
    </div>
  );
}
