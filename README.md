# viiru - terminal editor for Scratch

**Bringing the beloved graphical programming language for novices to an inaccessible text-based medium!**


*(Terribly sorry for the flickering in advance!)*

https://github.com/user-attachments/assets/c810f8cf-9e06-4060-a462-efefd5440122

## How to run

`viiru` currently requires both `npm` and `cargo` to be installed, because it is written in Rust and TypeScript.

First, install JavaScript dependencies: `npm install`

Then, you can launch the editor: `npm run main`

## Keyboard shortcuts

* o: open a project file from a given path
* w: write a project file to a given path
* q: quit, warning on unsaved changes
* Q: quit without saving changes
* hjkl: move the cursor
* HJKL: move the cursor and the scroll view
* space: interact with blocks; pick up and move them, edit inline values
* t: toggle toolbox view; the toolbox allows you to spawn new blocks
* s: stamp; creates a clone of the hovered block
* D: delete a held or hovered block

## What's next

* Optimize screen drawing routines to minimize flashing
* Clean up the toolbox, both in terms of implementation and usability. It is currently somewhat hacky 
  and inconvenient.
* More placement options, e.g. snapping blocks above or in the middle a stack
* Implement variable and list handling
* Implement extensions (which includes pen blocks), as well as hidden blocks for compatibility
* General code quality fixes all around (this has been a bit rushed)
* Implement custom blocks
* Implement target switching, i.e. editing scripts of more than a single sprite (or the stage)
* Implement static and dynamic dropdowns. This requires having access to multiple targets.
* Replace the `Spec`-string DSL with a common configuration format, such as TOML using `serde`. It was 
  initially designed to hold only the text of each block + its input slots, but slowly the scope grew...
* Add command-line flags to open projects directly without using the `o` command.
* Allow for custom keybindings to be provided through a configuration file (looking in `.config/` or a 
  command-line flag).
* Get rid of the `scratch-vm` and `scratch-blocks` libraries entirely, and perform my own serialization.
  I initially used them since I didn't want to reimplement all of the editor logic. However, the TUI still
  ended up holding a large amount of its own state and performing its own editor logic, in part because it 
  was easier compared to the task of digging it out of the libraries somewhere through the Node.js bridge.
  This would have the benefit of completely dropping the requirement on Node.js, as well as all of the
  dependencies from it. (`npm audit` is very unhappy.)
* Binary releases? Tests? The world's your oyster

## Acknowledgements

Thank you to taswell and Dolphy for being my #1 supporters, even though you had no idea what I was cooking.
