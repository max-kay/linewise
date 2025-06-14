import datetime as dt
import inspect
import importlib
import os
import sys
import time
import traceback

import drawsvg as draw
from termcolor import colored, cprint
from watchdog.events import FileSystemEventHandler
from watchdog.observers import Observer

import figs

REPORT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
FIG_DIR = os.path.join(REPORT_DIR, "generated")


def clear_screen():
    sys.stdout.write("\033[2J\033[H")
    sys.stdout.flush()


def make_imgs() -> str:
    start = dt.datetime.now()
    fnames = []
    errors = []
    no_error = True
    for name in dir(figs):
        func = getattr(figs, name)
        if not callable(func):
            continue
        sig = inspect.signature(func)
        return_type = sig.return_annotation
        if return_type is not draw.Drawing:
            continue

        file = f"{name}.svg"
        fnames.append(file)
        try:
            img: draw.Drawing = func()
            if not isinstance(img, draw.Drawing):
                raise TypeError(f"expected drawsvg.Drawing found {type(img)}")
            img.save_svg(os.path.join(FIG_DIR, file))
            errors.append("")
        except Exception:
            no_error = False
            errors.append(traceback.format_exc())

    if no_error:
        msg = colored("successfully generated figures:\n", "green")
        msg += "\n".join(fnames)
        msg += (
            f"\n\n    took {(dt.datetime.now() - start).total_seconds() * 1000:.3f}ms"
        )
        return msg
    else:
        msg = colored("could not generate figures:\n", "red")
        for name, err in zip(fnames, errors):
            if len(err) != 0:
                msg += f"{name}\n{err}\n"
        return msg


def compile() -> None:
    print(make_imgs())


class FigsEventHandler(FileSystemEventHandler):
    def on_modified(self, event) -> None:
        if event.is_directory:
            return
        *_, fname = event.src_path.split("/")
        if fname.endswith(".py"):
            clear_screen()
            cprint("Watching figures", "yellow", attrs=["bold"])
            try:
                importlib.reload(figs)
                msg = make_imgs()
                print(f"last compilation: {dt.datetime.now().strftime('%H:%M:%S')}")
                print(msg)
            except Exception:
                cprint("Error while generating images:", "red")
                traceback.print_exc()


def watch() -> None:
    event_handler = FigsEventHandler()
    observer = Observer()
    figs_path = os.path.join(REPORT_DIR, "scripts")
    observer.schedule(event_handler, figs_path)
    observer.start()

    clear_screen()
    cprint("Watching figures", "yellow", attrs=["bold"])
    print(f"Last compilation: {dt.datetime.now().strftime('%H:%M:%S')}")
    print(make_imgs())

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
        observer.join()
        print()


def main(argv: list[str]) -> None:
    try:
        os.makedirs(FIG_DIR, exist_ok=True)
        if len(argv) == 2:
            match argv[1]:
                case "watch" | "w":
                    watch()
                case "compile" | "c":
                    compile()
                case cmd:
                    print(f"unknown command `{cmd}`")
        elif len(argv) == 1:
            compile()
        else:
            print("invalid arguments")
    except KeyboardInterrupt:
        print()


if __name__ == "__main__":
    main(sys.argv)
