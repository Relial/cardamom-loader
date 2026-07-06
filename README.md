## Overview

Loads plugins from a folder placed next to the executable. Folder name can be set in cardamom-loader.toml and defaults to "cardamom".

Optionally opens a console window.

If you also want to use a different dinput8.dll, rename it to dinput8_c.dll and place it next to this one.

Made for 32bit executables.

## Linux

Run with WINEDLLOVERRIDES="dinput8=n,b"

If running from Steam, use WINEDLLOVERRIDES="dinput8=n,b" %command%

On some proton versions this override is already set by default.
