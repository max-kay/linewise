import datetime as dt
import importlib
import inspect
import os
import sys
import traceback

import drawsvg as draw
from termcolor import cprint, colored

import figs

REPORT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
FIG_DIR = os.path.join(REPORT_DIR, "generated")


def clear_screen():
    sys.stdout.write("\033[2J\033[H")
    sys.stdout.flush()


def make_imgs() -> list[tuple[str, str]]:
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
        try:
            img: draw.Drawing = func()
            img.save_svg(os.path.join(FIG_DIR, file))
            fnames.append(file)
            errors.append("")
        except Exception:
            no_error = False
            errors.append(traceback.format_exc())

    if no_error:
        return colored("successfully generated figures:\n", "green") + "\n".join(fnames)
    else:
        return colored("could not generate figures:", "red") + "\n".join(
            map(
                filter(zip(fnames, errors), lambda val: len(val[1]) != 0),
                lambda val: f"{val[0]}\n{val[1]}",
            )
        )


def compile() -> None:
    print(make_imgs())

def watch() -> None:
    file = os.path.join(REPORT_DIR, "scripts", "figs.py")
    last_mod: float = os.path.getmtime(file)
    msg = make_imgs()
    clear_screen()
    cprint("Watching Figures", "yellow", attrs=["bold"])
    print(f"last compilation: {dt.datetime.now().strftime('%H:%m:%S')}")
    print(msg)
    while True:
        mtime = os.path.getmtime(file)
        if last_mod < mtime:
            try:
                importlib.reload(figs)
                msg = make_imgs()
                last_mod = mtime
                clear_screen()
                cprint("Watching Figures", "yellow", attrs=["bold"])
                print(f"last compilation: {dt.datetime.now().strftime('%H:%m:%s')}")
                print(msg)
            except Exception:
                clear_screen()
                cprint("Watching Figures", "yellow", attrs=["bold"])
                print("could not generate images due to:")
                traceback.print_exc()
                last_mod = mtime
        time.sleep(1)


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
