# Fortune's Folio
Write notes minimalistically. Built with Sveltekit + Tauri

![Preview of the App](static/example-app.png)

## Installation
Click on the "Releases" tab on the right hand side of GitHub, download the .exe and run! That's it.

## Features
- Markdown supported
- Add Tags on notes
- View all tags on the "View Tags" page, along with all notes associated with a given tag. Click on a filtered note to navigate to it
- Filter notes by tag search
- Keybinds for all operations (such as creating a note)
- Single .exe download (6 mb size) - no installation process
- Undo the deletion of the last created note (until you close the software)

### Keybinds
- CTRL + N
    - Create a new note
- CTRL + D
    - Delete current note
- CTRL + S
    - Save current note
- CTRL + E
    - Edit current note
- CTRL + F
    - Toggle fullscreen
- CTRL + M
    - Minimize the app
- CTRL + Escape
    - Close the app
- CTRL + U
    - Undo the last note's deletion

### Features To-Do
- On the "View Tags" page, be able to click a red X on a tag to bring open a deletion dialogue. 
  It will show you all the notes that have that tag, and if you click "Proceed" it will remove 
  that tag entirely from the database, and delete the tag from any notes that may have it
- Add timestamps to each note. It will be set by default, but you can change it to whatever you prefer.
- Create downloads for MacOS and Linux
