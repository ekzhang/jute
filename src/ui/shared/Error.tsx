import { ArrowLeft } from "lucide-react";
import { Link } from "wouter";

type Props = {
  error?: string;
}

export function Error(props: Props) {
  return (
    <div className="flex flex-col gap-2 items-center justify-center w-full h-full">
      <h3 className="text-3xl">
        An error occured.
      </h3>

      <div className="text-l">
        {props.error || "An unknown error occurred."}
      </div>

      <Link to="/">
        <button className="mt-4 flex items-center gap-2">
          <ArrowLeft size="1em" />
          Go back to home
        </button>
      </Link>
    </div>
  );
}
