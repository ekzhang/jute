import { Command } from "cmdk";
import {
  ArrowDownIcon,
  ArrowUpIcon,
  FileTypeIcon,
  ListRestartIcon,
  ListVideoIcon,
  PaletteIcon,
  PauseIcon,
  PlayIcon,
  RotateCcw,
  SettingsIcon,
} from "lucide-react";
import { useEffect, useState } from "react";

export default function NotebookCommandMenu() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };

    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

  return (
    <Command.Dialog open={open} onOpenChange={setOpen}>
      <Command.Input autoFocus placeholder="Search for an action…" />

      <Command.List>
        {/* {loading && <Command.Loading>Hang on…</Command.Loading>} */}

        <Command.Empty>No results found.</Command.Empty>

        <Command.Group heading="Execution">
          <Command.Item>
            <PlayIcon /> Run cell
          </Command.Item>
          <Command.Item>
            <ListVideoIcon /> Run all cells
          </Command.Item>
          <Command.Item>
            <PauseIcon />
            Interrupt kernel
          </Command.Item>
          <Command.Item>
            <RotateCcw />
            Restart kernel
          </Command.Item>
          <Command.Item>
            <ListRestartIcon />
            Restart kernel and run all cells
          </Command.Item>
        </Command.Group>

        <Command.Group heading="Formatting">
          <Command.Item>
            <PaletteIcon />
            Format code with Black
          </Command.Item>
        </Command.Group>

        <Command.Group heading="Notebook">
          <Command.Item>
            <ArrowUpIcon />
            Move cell up
          </Command.Item>
          <Command.Item>
            <ArrowDownIcon />
            Move cell down
          </Command.Item>
          <Command.Item>
            <FileTypeIcon />
            Change cell type
          </Command.Item>
        </Command.Group>

        <Command.Group heading="Settings">
          <Command.Item>
            <SettingsIcon />
            Open settings
          </Command.Item>
        </Command.Group>
      </Command.List>
    </Command.Dialog>
  );
}
