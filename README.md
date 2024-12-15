# Fortune's Folio
Write notes minimalistically. Built with Sveltekit + Tauri

![Preview of the App](static/example-app.png)

## Features
- Markdown supported
- Add Tags on notes
- View all tags on the "View Tags" page, along with all notes associated with a given tag. Click on a filtered note to navigate to it
- Filter notes by tag search
- Keybinds for all operations (such as creating a note)
- Single .exe download (6 mb size) - no installation process

### Features To-Do
- On the "View Tags" page, be able to click a red X on a tag to bring open a deletion dialogue. 
  It will show you all the notes that have that tag, and if you click "Proceed" it will remove 
  that tag entirely from the database, and delete the tag from any notes that may have it
- Add timestamps to each note. It will be set by default, but you can change it to whatever you prefer.
- Create downloads for MacOS and Linux

### Bugs To-Do
- Bug where if you look up a tag on the notes page, it somehow splits the tag you searched for on 
  the note into its component letters individually into separate tags. Just a visual bug,
  but it strangely only happens in release builds