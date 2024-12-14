# Fortune's Folio
An app to take your notes, minimalistically. Built with Sveltekit + Tauri

## Features To-Do
- On the "View Tags" page, be able to click a red X on a tag to bring open a deletion dialogue. 
  It will show you all the notes that have that tag, and if you click "Proceed" it will remove 
  that tag entirely from the database, and delete the tag from any notes that may have it
- Add timestamps to each note. It will be set by default, but you can change it to whatever you prefer.

## Bugs To-Do
- Bug where if you look up a tag on the notes page, it somehow splits the tag you searched for on 
  the note into its component letters individually into separate tags. Just a visual bug
- The sidebar adjustment javascript is broken
- When typing a note when its not fullscreen, the textarea takes up a huge area offscreen 
  before it goes to the next line. It needs to fit the viewport-width so I can actually see my 
  entire note as I type