# Fortune's Folio
An app to take your notes, minimalistically, focused around the idea of "tagging" your notes. Good for making thought bubbles. Built with Sveltekit + Tauri

## Features To-Do
- On the "View Tags" page, be able to click a red X on a tag to bring open a deletion dialogue. 
  It will show you all the notes that have that tag, and if you click "Proceed" it will remove 
  that tag entirely from the database, and delete the tag from any notes that may have it
- Add timestamps to each note. It will be set by default, but you can change it to whatever you prefer.

## Bugs To-Do
- Bug where if you look up a tag on the notes page, it somehow splits the tag you searched for on 
  the note into its component letters individually into separate tags. Just a visual bug,
  but it strangely only happens in release builds