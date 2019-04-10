# datboi
This started out as an experiment, and to learn more about how emulators actually do what they do.  
I ended up not having the time to finish it, but I might come back to it and get it to a point where it can play a few simple games.

For now, it does the following things:

* Parse and correctly execute all opcodes (including multi byte opcodes),
* Load the bios and play the Logo animation,
* Write the logo animation to the graphics output,
* Boot Tetris (without graphical output).

Things that would be required to actually play a game:

* Add all memory banks,
* Add sound output,
* Probably fix a bunch of opcode bugs.

If you have any question, feel free to shoot me a message or open an issue.

## Resources
The following things were used as resources

* [Gameboy opcodes](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
* [Gameboy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
* [GB Emulation in Javascript by Imran Nazar](http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-The-CPU)
