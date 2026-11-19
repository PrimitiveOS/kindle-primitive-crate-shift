# Install Sokoban

1. Open **Store** from Primitive Launcher.
2. Search for **Sokoban** and tap **Get**.
3. Open **Sokoban** from the Launcher home screen.

Development builds can be installed by USB on an ARMHF Kindle running firmware 5.16.3 or newer. Check the release ZIP
against its SHA-256 file, then extract it. Copy the `crateshift/` payload folder to the Kindle drive root and
`documents/CrateShift.sh` into the existing `documents/` folder. Eject the Kindle before opening the game.

The game uses Launcher’s shared fonts and display settings. It saves progress in `crateshift/crateshift.json` on the Kindle.
