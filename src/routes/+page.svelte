<script lang="ts">
    import { goto } from '$app/navigation';
    import { marked, Renderer } from 'marked';
    import { type Tokens } from 'marked';
    import { onMount, tick } from "svelte";
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';
    import { writable, get } from 'svelte/store';
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import type { Folder, Note, FolderNode } from '../lib/types';
    import { buildFolderTree } from '../lib/buildFolderTree';
    import TreeFolder from '../components/TreeFolder.svelte';
    import { selectedNoteID } from '../stores';

    const appWindow = getCurrentWindow();

    marked.setOptions({
        breaks: true, // Enable line breaks
        gfm: true,    // Enable GitHub Flavored Markdown
    });

    marked.use({
        renderer: {
            link(this: Renderer, token: Tokens.Link) {
                const { href, title, text } = token;

                // If no href, default to '#'
                const safeHref = href || '#';
                // If there's a title, we include it. Otherwise, omit the attribute.
                const safeTitle = title ? ` title="${title}"` : '';

                return `<a href="${safeHref}"${safeTitle} target="_blank" rel="noopener noreferrer">${text}</a>`;
            }
        }
    });

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

    function startResizing(event: MouseEvent): void {
        isResizing = true;
        startX = event.clientX;

        const sidebar = document.querySelector("aside") as HTMLElement | null;
        if (sidebar) {
            startWidth = sidebar.offsetWidth;

            // Add listeners for continuous resizing
            document.addEventListener("mousemove", resize);
            document.addEventListener("mouseup", stopResizing);
        }
    }

    let isAnimating = false;
    function resize(event: MouseEvent): void {
        if (isResizing && !isAnimating) {
            isAnimating = true;

            requestAnimationFrame(() => {
                const dx = event.clientX - startX;

                const sidebar = document.querySelector("aside") as HTMLElement | null;
                if (sidebar) {
                    let newWidth = startWidth + dx;

                    // Explicitly handle resizing from 0px to a positive width
                    if (newWidth < 1) {
                        newWidth = 0;
                    } else {
                        newWidth = Math.min(newWidth, window.innerWidth * 0.5); // Cap at 50vw
                    }

                    // Disable transitions if resizing from 0px
                    if (parseFloat(getComputedStyle(sidebar).width) === 0) {
                        sidebar.style.transition = "none"; // Temporarily disable transitions
                    } else {
                        sidebar.style.transition = ""; // Restore transitions
                    }

                    // Update CSS variable
                    document.documentElement.style.setProperty("--sidebar-width", `${newWidth}px`);
                }
                isAnimating = false;
            });
        }
    }

    function stopResizing(): void {
        isResizing = false;

        // Remove listeners when resizing stops
        document.removeEventListener("mousemove", resize);
        document.removeEventListener("mouseup", stopResizing);
    }

    let tags: string[] = []; // List of all existing tags
    let filteredTags  = []; // Filtered tags for autocomplete
    let isInputFocused: boolean = false; // Track input focus state

    let notes: Note[] = [];
    let selectedNote: Note | null = null;
    let tagSearch = "";
    let newTag = "";
    let filteredTagSearch: string[] = []; // Filtered tags for autocomplete
    let isSearchFocused: boolean = false; // Tracks focus on the search input

    // Filter tags for the autocomplete dropdown
    $: filteredTagSearch = [...new Set(tags.filter(tag =>
        tag.toLowerCase().includes(tagSearch.toLowerCase()) &&
        tagSearch.trim() !== ''
    ))];

    // Handle tag selection
    const selectTag = (tag: string) => {
        tagSearch = tag; // Set the input value
        isSearchFocused = false; // Hide the dropdown
        searchNotesByTag(); // Trigger search with the selected tag
    };

    async function addTag(): Promise<void> {
        if (selectedNote) {
            const trimmedTag = newTag.trim();
            if (trimmedTag && !selectedNote.tags.includes(trimmedTag)) {
                // Update tags locally
                selectedNote.tags = [...selectedNote.tags, trimmedTag];

                // Reset input
                newTag = "";

                // Ensure the change is reflected in the notes list
                notes = notes.map(note =>
                    note.id === selectedNote?.id
                        ? { ...note, tags: selectedNote?.tags }
                        : note
                );

                // Optionally, save the change to the backend
                try {
                    await invoke("update_note_tags", { id: selectedNote.id, tags: selectedNote.tags });
                } catch (error) {
                    console.error("Failed to update tags:", error);
                }
            }
        }
    }

    function selectNote(note: Note): void {
        selectedNote = {
            ...note,
            tags: Array.isArray(note.tags)
                ? note.tags.filter(tag => tag.trim() !== "") // Remove empty tags
                : note.tags.split(",").map((tag) => tag.trim()).filter(tag => tag !== ""),
            markdown: true, // Default to Preview Markdown mode
        };
    }

    async function addNewNote() {
        const newNote = {
            folderid: Number(1), // pass a folder or null
            title: `New Note ${notes.length + 1}`,
            content: "",
            markdown: false,
        };

        console.log(newNote)
        console.log("selectedFolderId is", selectedFolderId);

        try {
            // Add the new note to the backend
            await invoke("add_note", {
                folderid: Number(selectedFolderId), // pass a folder or null
                title: `New Note ${notes.length + 1}`,
                content: "",
                markdown: false,
            });

            // Fetch the updated notes list
            await fetchNotes();

            // Identify the newly added note (assuming it's the last one added)
            selectedNote = {
                ...notes[notes.length - 1],
                markdown: false,
                tags: [], // Ensure no empty tags are present
            };
        } catch (error) {
            console.error("Failed to add note:", error);
        }
    }

    async function addNewFolder() {
        const name = prompt("Enter folder name:");
        if (!name) return;

        // If you want subfolders, pass the currently selected folder as `parent_id`.
        // If you only want top-level, pass `null`.
        let parentId = selectedFolderId;

        // If you *never* want subfolders, always do `parentId = null`.

        try {
            await invoke("add_folder", { name, parentId });
            await fetchFolders();
            buildTree();
        } catch (err) {
            console.error("Failed to create folder:", err);
        }
    }

    async function deleteFolder(folderId: number) {
        if (!folderId) return;

        const confirmDelete = confirm("Are you sure you want to delete this folder?");
        if (!confirmDelete) return;

        try {
            await invoke("delete_folder", { folder_id: folderId });
            await fetchFolders(); // Refresh folder data
            buildTree(); // Rebuild the tree structure
        } catch (err) {
            console.error("Failed to delete folder:", err);
        }
    }


    async function saveChanges() {
        if (selectedNote) {
            try {
                // Save the selected note's ID
                const currentNoteId = selectedNote.id;

                const validTags = Array.isArray(selectedNote.tags)
                    ? selectedNote.tags.filter((tag) => tag.trim() !== "") // Filter out empty tags
                    : selectedNote.tags
                    ? selectedNote.tags.split(",").map((tag) => tag.trim()).filter((tag) => tag !== "")
                    : [];

                await invoke("update_note", {
                    id: selectedNote.id,
                    title: selectedNote.title,
                    content: selectedNote.content,
                    markdown: selectedNote.markdown,
                    tags: validTags,
                    folderid: selectedNote.folderid ?? null,
                });

                selectedNote.markdown = true; // Switch to markdown mode after saving
                await fetchNotes(); // Refresh the notes list

                // Re-select the current note based on its ID
                const updatedNote = notes.find((note) => note.id === currentNoteId);
                if (updatedNote) {
                    selectNote(updatedNote);
                }
            } catch (error) {
                console.error("Failed to save changes:", error);
            }
        }
    }

    // Saves changes without exiting the edit move
    async function saveChangesInline() {
        if (selectedNote) {
            try {
                // Save the selected note's ID
                const currentNoteId = selectedNote.id;

                const validTags = Array.isArray(selectedNote.tags)
                    ? selectedNote.tags.filter((tag) => tag.trim() !== "") // Filter out empty tags
                    : selectedNote.tags
                    ? selectedNote.tags.split(",").map((tag) => tag.trim()).filter((tag) => tag !== "")
                    : [];

                await invoke("update_note", {
                    id: selectedNote.id,
                    title: selectedNote.title,
                    content: selectedNote.content,
                    markdown: selectedNote.markdown,
                    tags: validTags,
                    folderid: selectedNote.folderid ?? null
                });

                // selectedNote.markdown = true; // Switch to markdown mode after saving
                await fetchNotes(false); // Refresh the notes list

                // // Re-select the current note based on its ID
                // const updatedNote = notes.find((note) => note.id === currentNoteId);
                // if (updatedNote) {
                //     selectNote(updatedNote);
                // }
            } catch (error) {
                console.error("Failed to save changes:", error);
            }
        }
    }

    let lastDeletedNote: Note | null = null;
    async function deleteNote() {
        if (!selectedNote) return;

        try {
            // Save the current note as the last deleted note
            lastDeletedNote = { ...selectedNote };

            // Send delete request to the backend
            await invoke("delete_note", { id: selectedNote.id });

            // Refresh the notes list
            await fetchNotes();

            // Clear the selected note if no notes remain
            if (notes.length === 0) {
                selectedNote = null;
            } else {
                // Select another note (e.g., the first one)
                selectedNote = notes[0];
            }
        } catch (error) {
            console.error("Failed to delete note:", error);
        }
    }

    async function undoDeleteNote() {
        if (!lastDeletedNote) return;

        try {
            // Add the last deleted note back to the backend
            await invoke("add_note", {
                title: lastDeletedNote.title,
                content: lastDeletedNote.content,
                markdown: lastDeletedNote.markdown,
                tags: lastDeletedNote.tags,
            });

            // Fetch the updated notes list
            await fetchNotes();

            // Select the restored note
            selectedNote = notes.find(
                note => note.title === lastDeletedNote!.title && note.content === lastDeletedNote!.content
            ) || null;

            // Clear the deleted note reference after restoration
            lastDeletedNote = null;
        } catch (error) {
            console.error("Failed to undo delete note:", error);
        }
    }

    let textareaElement: HTMLTextAreaElement | null = null;

    function autoResizeTextarea() {
        // Also automatically save the text area
        saveChangesInline();
        
        if (textareaElement) {
            // Save the current scroll position of the window
            const windowScrollY = window.scrollY;

            // Reset the textarea height and adjust it to fit content
            textareaElement.style.height = 'auto'; // Reset height
            textareaElement.style.height = `${textareaElement.scrollHeight + 5}px`; // Set height to fit content. Add +5 to ensure no overflow

            // Get the caret (cursor) position
            const selectionStart = textareaElement.selectionStart;

            // Create a temporary div to measure caret position
            const tempDiv = document.createElement('div');
            const textBeforeCaret = textareaElement.value.substring(0, selectionStart);
            const lineHeight = parseInt(window.getComputedStyle(textareaElement).lineHeight, 10) || 16;

            // Mirror textarea styles onto the temporary div
            const style = window.getComputedStyle(textareaElement);
            tempDiv.style.cssText = `
                position: absolute;
                visibility: hidden;
                white-space: pre-wrap;
                word-wrap: break-word;
                width: ${textareaElement.offsetWidth}px;
                font-size: ${style.fontSize};
                font-family: ${style.fontFamily};
                line-height: ${style.lineHeight};
                padding: ${style.paddingTop} ${style.paddingRight} ${style.paddingBottom} ${style.paddingLeft};
            `;
            tempDiv.textContent = textBeforeCaret.replace(/\n$/, '\n\n'); // Add newline for accurate line spacing

            // Append the temp div to the body
            document.body.appendChild(tempDiv);

            // Get the position of the caret in the textarea
            const caretRect = tempDiv.getBoundingClientRect();
            const caretTop = caretRect.top;

            // Clean up the temporary div
            document.body.removeChild(tempDiv);

            // Calculate the center position of the viewport
            const viewportCenter = window.innerHeight / 2;

            // Scroll the window to keep the caret in the center of the viewport
            if (caretTop > viewportCenter || caretTop < viewportCenter) {
                window.scrollTo(0, windowScrollY + (caretTop - viewportCenter));
            }
        }
    }


    async function toggleMarkdown(): Promise<void> {
        if (selectedNote) {
            selectedNote.markdown = !selectedNote.markdown;
            await tick();
            autoResizeTextarea();
        }
    }

    function getLineRange(textarea: HTMLTextAreaElement): [number, number] {
        const text = textarea.value;
        const pos = textarea.selectionStart;

        // Find the start of the line
        let lineStart = text.lastIndexOf('\n', pos - 1);
        if (lineStart === -1) {
            lineStart = 0;
        } else {
            lineStart += 1; // move from the newline char to the next character
        }

        // Find the end of the line
        let lineEnd = text.indexOf('\n', pos);
        if (lineEnd === -1) {
            lineEnd = text.length;
        }

        return [lineStart, lineEnd];
    }

    function wrapSelection(symbolLeft: string, symbolRight: string = symbolLeft) {
        if (!selectedNote || selectedNote.markdown || !textareaElement) return;

        let start = textareaElement.selectionStart;
        let end = textareaElement.selectionEnd;

        // If no selection, select the entire line
        if (start === end) {
            const [lineStart, lineEnd] = getLineRange(textareaElement);
            start = lineStart;
            end = lineEnd;
        }

        const text = textareaElement.value;
        const before = text.substring(0, start);
        const selectedText = text.substring(start, end);
        const after = text.substring(end);

        const newText = before + symbolLeft + selectedText + symbolRight + after;

        // Update <textarea>
        textareaElement.value = newText;

        // Place the cursor after the newly added symbols
        const newCursor = end + symbolLeft.length + symbolRight.length;
        textareaElement.setSelectionRange(newCursor, newCursor);

        // Sync with your note
        selectedNote.content = newText;
        autoResizeTextarea();
    }

    function makeLink() {
        if (!selectedNote || selectedNote.markdown || !textareaElement) return;

        let start = textareaElement.selectionStart;
        let end = textareaElement.selectionEnd;
        const text = textareaElement.value;

        // If no selection, select the entire line
        if (start === end) {
            const [lineStart, lineEnd] = getLineRange(textareaElement);
            start = lineStart;
            end = lineEnd;
        }

        // Now we have a start/end range (either user-selected or the entire line)
        const before = text.substring(0, start);
        let selectedText = text.substring(start, end).trim(); // e.g. "test"

        // If there's literally nothing on that line, default to "title"
        if (!selectedText) {
            selectedText = "title";
        }

        const after = text.substring(end);

        // Replace selection with [selectedText](https://www.)
        const newText = before + `[${selectedText}](https://www.)` + after;
        textareaElement.value = newText;

        // Place the cursor inside the parentheses "()" so user can type the URL
        const newCursorPos = before.length + selectedText.length + 3; // [ + selectedText + ]( 
        textareaElement.setSelectionRange(newCursorPos, newCursorPos);

        // Sync with your note and auto-resize
        selectedNote.content = newText;
        autoResizeTextarea();
    }

    function addHeaderLevel() {
        if (!selectedNote || selectedNote.markdown || !textareaElement) return;

        let start = textareaElement.selectionStart;
        let end = textareaElement.selectionEnd;
        const text = textareaElement.value;

        // If no selection, select the entire line
        if (start === end) {
            const [lineStart, lineEnd] = getLineRange(textareaElement);
            start = lineStart;
            end = lineEnd;
        }

        const before = text.substring(0, start);
        const selection = text.substring(start, end);
        const after = text.substring(end);

        // Convert selection into lines
        const lines = selection.split("\n").map((line) => {
            // Keep original leading spaces
            const leadingSpaces = line.length - line.trimStart().length;
            const leftIndent = line.slice(0, leadingSpaces);
            const trimmed = line.trimStart();

            // Count how many # are already at the start
            const headingPrefixMatch = trimmed.match(/^#+/);
            const existingCount = headingPrefixMatch ? headingPrefixMatch[0].length : 0;

            // Increase heading level by 1
            const newCount = existingCount + 1;

            // Remove old #, then ensure a single space
            const textAfterPrefix = trimmed.replace(/^#+/, "").trimStart();

            return leftIndent + "#".repeat(newCount) + " " + textAfterPrefix;
        });

        const newSelection = lines.join("\n");
        const newText = before + newSelection + after;

        textareaElement.value = newText;
        // Keep selection range around newly transformed lines
        textareaElement.setSelectionRange(start, start + newSelection.length);

        selectedNote.content = newText;
        autoResizeTextarea();
    }

    // Add the keybind listener
    export const maximizedWindow = writable(false);
    function handleKeydown(event: KeyboardEvent): void {
        if ((event.ctrlKey || event.metaKey) && event.key === "e") {
            event.preventDefault();
            if (selectedNote) { 
                toggleMarkdown();
            }
        }
        if ((event.ctrlKey || event.metaKey) && event.key === "s") {
            event.preventDefault();
            saveChanges();
        }
        if ((event.ctrlKey || event.metaKey) && event.key === "d") {
            event.preventDefault();
            deleteNote();
        }
        if ((event.ctrlKey || event.metaKey) && event.key === "n") {
            event.preventDefault();
            addNewNote();
        }
        if ((event.ctrlKey || event.metaKey) && event.key === "f") {
            event.preventDefault();
            appWindow.toggleMaximize();
        }
        if((event.ctrlKey || event.metaKey) && event.key == "m") {
            event.preventDefault();
            appWindow.minimize();
        }
        if((event.ctrlKey || event.metaKey) && event.key == "Escape") {
            event.preventDefault();
            appWindow.close()
        }
        if((event.ctrlKey || event.metaKey) && event.key == "t") {
            event.preventDefault();
            goto('/tags');
        }
        if((event.ctrlKey || event.metaKey) && event.key == "u") {
            event.preventDefault();
            undoDeleteNote();
        }
        // Bold
        if ((event.ctrlKey || event.metaKey) && event.key === "b") {
            event.preventDefault();
            wrapSelection("**");
        }

        // Italic
        if ((event.ctrlKey || event.metaKey) && event.key === "i") {
            event.preventDefault();
            wrapSelection("*");
        }

        // Heading
        // Each press adds another '#'
        if ((event.ctrlKey || event.metaKey) && event.key === "h") {
            event.preventDefault();
            addHeaderLevel();
        }

        // Link => Ctrl+Shift+L
        if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "l") {
            event.preventDefault();
            makeLink();
        }
    }

    // For when user clicks a folder in the tree
    let selectedFolderId: number | null = null;
  function handleSelectFolder(folderId: number) {
    if(selectedFolderId == folderId) {
        selectedFolderId = null;
    }
    else selectedFolderId = folderId;
    // you could show folder details or do nothing
  }

  // For when user clicks a note in the tree
  function handleSelectNote(note: Note) {
    selectNote(note);
    selectedNoteId = note.id;
  }

    async function searchNotesByTag(): Promise<void> {
        if (!tagSearch.trim()) {
            // Fetch all notes if search input is empty
            await fetchNotes();
            return;
        }

        try {
            const filtered = (await invoke("search_notes_by_tag", { tag: tagSearch.trim() })) as Note[];
            notes = filtered;

            // Reset selected note if none matches the search
            selectedNote = notes.length > 0 ? { ...notes[0], markdown: true } : null;
        } catch (error) {
            console.error("Failed to search notes by tag:", error);
        }
    }

  // Fetch notes from the backend
  let folders: Folder[] = [];

  async function fetchFolders() {
        try {
            folders = await invoke<Folder[]>('get_folders');
        } catch (error) {
            console.error('Failed to fetch folders:', error);
        }
  }

    async function fetchNotes(selectFirst: boolean = true) {
        try {
            const rawNotes = (await invoke("get_notes")) as Note[];
            notes = rawNotes.map((note) => ({
                ...note,
                tags: typeof note.tags === "string"
                    ? note.tags.split(",").map((tag) => tag.trim()) // Convert tags from string to array
                    : note.tags,
                markdown: true, // Set markdown mode to true by default after saving
                folderid: note.folderid,
            }));
            if (selectFirst && notes.length > 0) {
                selectedNote = notes[0]; // Select the first note by default
                selectedNoteId = selectedNote.id;
            }
        } catch (error) {
            console.error("Failed to fetch notes:", error);
        }
    }

    // Store for dark mode
    export const darkMode = writable(true);

    // Toggle dark mode
    async function loadDarkMode() {
        const enabled = await invoke<boolean>('get_dark_mode');
        darkMode.set(enabled);

        // Save the dark mode state in localStorage
        localStorage.setItem('darkMode', enabled ? 'true' : 'false');

        if (enabled) {
            document.documentElement.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
        }

        // Force a tick to ensure the DOM re-renders
        await tick();
    }

    async function toggleDarkMode() {
        const currentMode = get(darkMode); // Get the current mode value
        const newMode = !currentMode;

        darkMode.set(newMode); // Update the store immediately for UI responsiveness

        localStorage.setItem('darkMode', newMode ? 'true' : 'false');

        // Persist the new mode to the backend
        try {
            await invoke('set_dark_mode', { enabled: newMode });
            if (newMode) {
                document.documentElement.classList.add('dark');
            } else {
                document.documentElement.classList.remove('dark');
            }
        } catch (error) {
            console.error('Failed to update dark mode:', error);
            // Rollback if saving fails
            darkMode.set(currentMode);
        }
    }


    // Fetch all tags from the backend
    const fetchTags = async () => {
        try {
            tags = await invoke<string[]>('get_tags');
        } catch (error) {
            console.error('Failed to fetch tags:', error);
        }
    };

    // Filter existing tags based on input
    $: filteredTags = tags
        .filter(tag =>
            tag.toLowerCase().includes(newTag.toLowerCase()) &&
            newTag.trim() !== '' &&
            (!selectedNote?.tags || !selectedNote.tags.includes(tag))
        );

    function calculateMaxWidth(): void {
        const header = document.querySelector<HTMLElement>("div.big-parent-container");
        if (!header) return; // Exit if the header is not found

        // Get the total width of the header
        const headerWidth = header.getBoundingClientRect().width;

        // Calculate the combined width of all children except the 'main' element
        let otherChildrenWidth = 0;

        Array.from(header.children).forEach((child) => {
            const element = child as HTMLElement; // Ensure type safety for child elements
            if (!element.matches("main")) { // Skip the 'main' child
                const childWidth = element.getBoundingClientRect().width;
                otherChildrenWidth += childWidth;
            }
        });

        // Calculate the available width for 'main'
        const mainWidth = Math.max(0, headerWidth - otherChildrenWidth); // Ensure non-negative width

        // Apply the calculated width to 'main'
        const main = header.querySelector<HTMLElement>("main");
        if (main) {
            main.style.width = `${mainWidth}px`;
        }
    }

    let folderTree: FolderNode[] = []; // Our final tree structure

  // After we fetch `folders` and `notes`, build the tree:
  function buildTree() {
    folderTree = buildFolderTree(folders, notes, null);
  }

    let containerElement: HTMLElement | null = null;

    let selectedNoteId: number | null = null;
    onMount(() => {
        // Check system preferences for dark mode
        loadDarkMode();
        fetchTags()
        // Adjust the textarea height on mount
        autoResizeTextarea(); 

        // Observe resizing for responsive updates
        const resizeObserver = new ResizeObserver(() => {
            calculateMaxWidth();
        });

        if(containerElement) resizeObserver.observe(containerElement);

        // Initial calculation
        calculateMaxWidth();

        // Wrap the async logic in a self-invoking function
        (async () => {
            // await fetchNotes(); // Ensure notes are fetched before proceeding
            // await fetchFolders();
            await Promise.all([fetchFolders(), fetchNotes()]).then(() => {
                buildTree(); 
            });

            const params = new URLSearchParams(window.location.search);
            const noteIdParam = params.get('noteId');
            selectedNoteId = noteIdParam ? parseInt(noteIdParam) : null;

            if (selectedNoteId) {
                selectedNote = notes.find((note) => note.id === selectedNoteId) || null;
            }
        })();

        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
            resizeObserver.disconnect(); // Clean up observer on component destroy
        };
    });
    $: console.log(selectedNoteId)
    $: if(selectedNote) {
        selectedNoteId = selectedNote.id;
    }
    $: if(selectedNote) {
        selectedNoteID.set(selectedNote.id);
    }
</script>

<div bind:this={containerElement} class="big-parent-container flex flex-grow bg-gray-100 dark:bg-gray-900 w-full">
    <!-- Sidebar -->
    <aside
        class="w-1/4 bg-gray-800 text-white p-4 flex-none flex flex-col h-screen"
        style="max-height: calc(100vh - 32px);"
    >
        <div class="flex items-center justify-between mb-6">
            <!-- Notes Header -->
            <h2 class="text-2xl font-bold">Notes</h2>

            <!-- Dark Mode Button -->
            <button
                class="flex items-center space-x-2 p-2 rounded-lg transition-colors
                    bg-gray-200 text-gray-600 hover:!bg-gray-300 dark:!bg-gray-600 dark:!text-gray-200"
                on:click={toggleDarkMode}
            >
                <!-- Moon Icon -->
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="currentColor"
                    viewBox="0 0 24 24"
                    class="w-5 h-5 transition-colors text-gray-600 dark:text-gray-200"
                >
                    <path
                        d="M21.753 15.904a9.005 9.005 0 01-10.796-10.797c.084-.336-.206-.63-.54-.547A10.001 10.001 0 1019.94 19.94c.084-.336-.211-.63-.547-.54z"
                    />
                </svg>
            </button>
        </div>

        <!-- Search by Tag -->
        <div class="relative mb-4">
            <input
                type="text"
                class="w-full p-2 rounded-lg text-gray-900"
                bind:value={tagSearch}
                placeholder="Search by tag..."
                on:focus={() => (isSearchFocused = true)}
                on:blur={() => setTimeout(() => (isSearchFocused = false), 150)}
                on:input={searchNotesByTag}
            />
            <!-- Autocomplete Dropdown -->
            {#if filteredTagSearch.length > 0 && isSearchFocused}
            <ul
                class="autocomplete absolute top-full mt-1 left-0 w-full bg-white border rounded-lg z-10 shadow-md"
            >
                {#each filteredTagSearch as tag (tag)}
                <li>
                    <button
                        class="px-2 py-1 w-full text-left cursor-pointer hover:bg-blue-100 focus:outline-none"
                        on:click={() => selectTag(tag)}
                        on:keydown={(e) => {
                            if (e.key === 'Enter') {
                                e.preventDefault();
                                selectTag(tag);
                            }
                        }}
                    >
                        {tag}
                    </button>
                </li>
                {/each}
            </ul>
            {/if}
        </div>

        <!-- Notes List -->
        <!-- Folder Explorer -->
         <div class="flex-1 overflow-y-auto max-h-full">
        <TreeFolder
            folderTree={folderTree}
            onSelectFolder={handleSelectFolder}
            onSelectNote={handleSelectNote}
            selectedFolderId={selectedFolderId}
        />
         </div>

         <button
         class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
         on:click={addNewFolder}
       >
         + New Folder
       </button>
        <button
            class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
            on:click={() => addNewNote()}
        >
            + New Note
        </button>
        <button
            class="mt-4 p-2 bg-blue-500 text-white font-semibold rounded-lg hover:bg-blue-600"
            on:click={() => goto('/tags')}
        >
            View Tags
        </button>
    </aside>

    <!-- Resizable handle -->
    <button
        aria-label="Resize sidebar"
        class="w-1 bg-gray-600 cursor-col-resize focus:outline-none"
        on:mousedown={startResizing}
    ></button>


    <!-- Main Content -->
  <main style="max-width: 100%;" class="flex-1 flex flex-col relative whitespace-nowrap">
    <!-- Header -->
    <header style="max-width: 100%;" class="shadow-md w-full p-4 flex items-center justify-between bg-white dark:bg-gray-800 dark:text-white">
      <div class="flex-1 overflow-auto whitespace-nowrap">
        {#if selectedNote}
          {#if selectedNote.markdown}
            <h1 class="text-xl font-bold truncate">{selectedNote.title}</h1>
          {:else}
            <input
              type="text"
              class="text-xl font-bold border-b-2 focus:outline-none focus:border-blue-500 p-1 w-full max-w-lg truncate"
              bind:value={selectedNote.title}
              on:input={saveChangesInline}
              placeholder="Enter title here"
            />
          {/if}
        {/if}
      </div>
      {#if selectedNote}
        <div class="flex-none ml-4 flex space-x-4">
            {#if selectedNote.markdown}
          <button
            class="font-semibold p-2 bg-green-600 text-white rounded-lg hover:bg-yellow-600"
            on:click={toggleMarkdown}
            on:click={autoResizeTextarea}
          >
          Edit
          </button>
          {/if}
          {#if !selectedNote.markdown}
            <button
                class="font-semibold p-2 bg-green-500 text-white rounded-lg hover:bg-green-600"
                on:click={saveChanges}
            >
                Preview
            </button>
          {/if}
          <button
            class="font-semibold p-2 bg-red-500 text-white rounded-lg hover:bg-red-600"
            on:click={deleteNote}
          >
            Delete
          </button>
          {#if lastDeletedNote}
            <button
                class="p-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
                on:click={undoDeleteNote}
                >
                Undo Delete
            </button>
          {/if}
        </div>
      {/if}
      
    </header>
  
    <!-- Content -->
    <div class="flex-1 flex flex-col min-h-100 bg-gray-50 overflow-y-auto dark:bg-gray-900">
      {#if selectedNote}
        {#if selectedNote.markdown}
          <div style="flex: 1; min-height: auto; white-space: normal; word-wrap: break-word;" class="p-4 prose max-w-none mb-8">
            {@html marked(selectedNote.content)}
          </div>
        {:else}
        <textarea
            style="flex: 1; min-height: auto;" 
            class="flex-1 w-full h-auto border rounded-lg p-4 focus:outline-none resize-none bg-white dark:bg-gray-200 dark:text-gray-800"
            bind:value={selectedNote.content}
            placeholder="Start writing your note here..."
            on:input={autoResizeTextarea}
            bind:this={textareaElement}
        ></textarea>
        {/if}
      {/if}
      <!-- Sticky Tags Section -->
      {#if selectedNote}
    <div style="padding: 20px 0.5rem;" class={`h-auto bg-gray-100 border-t flex flex-col ${!selectedNote?.markdown ? 'h-28 sticky bottom-0' : 'h-16'} px-2 pt-2`}>
        <div class="flex items-center">
            <h2 class="text-lg font-bold pr-2">Tags</h2>
            <div style="max-height: 100px; overflow-y: auto;" class="flex items-center space-x-2 flex-wrap">
                {#if selectedNote}
                    {#each selectedNote?.tags as tag (tag)}
                        {#if tag.trim() !== ""}
                            <div
                                style="max-width: 250px; overflow-x: auto; background-color: rgb(229 231 235 / var(--tw-bg-opacity, 1));"
                                class="tag flex items-center px-2 py-1 my-2 rounded"
                                class:bg-blue-500={!selectedNote.markdown && tag.includes(newTag.trim())}
                                class:bg-gray-200={selectedNote.markdown && !tag.includes(newTag.trim())}
                                class:text-white={!selectedNote.markdown && tag.includes(newTag.trim())}
                                class:text-gray-800={selectedNote.markdown && !tag.includes(newTag.trim())}
                            >
                                {tag}
                                {#if !selectedNote.markdown}
                                    <button
                                        class="ml-2 text-red-500 hover:text-red-700"
                                        on:click={() => {
                                            if (Array.isArray(selectedNote!.tags)) {
                                                // If tags is already an array, filter it
                                                selectedNote!.tags = selectedNote!.tags.filter(t => t !== tag);
                                            } else if (typeof selectedNote!.tags === 'string') {
                                                // If tags is a string, split it into an array, filter, and rejoin
                                                selectedNote!.tags = selectedNote!.tags
                                                    .split(',')
                                                    .map(t => t.trim())
                                                    .filter(t => t !== tag)
                                                    .join(', ');
                                            }
                                        }}
                                    >
                                        &times;
                                    </button>
                                {/if}
                            </div>
                        {/if}
                    {/each}
                {:else if tagSearch.trim()}
                    <!-- Show placeholder if no note is selected -->
                    <p class="text-gray-500 italic">No matching notes found for tag "{tagSearch.trim()}".</p>
                {/if}
            </div>
        </div>
    
        {#if !selectedNote?.markdown}
        <div class="relative mt-4">
            <div class="flex space-x-2 items-center">
                <input
                    type="text"
                    class="flex-grow border rounded-lg px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
                    bind:value={newTag}
                    placeholder="Add a tag..."
                    on:focus={() => (isInputFocused = true)}
                    on:blur={() => setTimeout(() => (isInputFocused = false), 150)} 
                />
                <button
                    class="flex-shrink-0 px-4 py-1 bg-blue-500 text-white rounded-lg hover:bg-blue-600 whitespace-nowrap"
                    on:click={() => { addTag(); saveChangesInline(); }}
                >
                    Add Tag
                </button>
            </div>
    
            <!-- Autocomplete Dropdown -->
            {#if filteredTags.length > 0 && isInputFocused}
                <ul
                    class="absolute bottom-full mb-1 left-0 w-full bg-white border rounded-lg z-10 shadow-md"
                >
                    {#each filteredTags as tag (tag)}
                    <li>
                        <button
                            class="px-2 py-1 w-full text-left cursor-pointer hover:bg-blue-100 focus:outline-none"
                            on:click={() => {
                                newTag = tag; // Set the input value to the selected suggestion
                                isInputFocused = true; // Keep the dropdown visible for further input
                            }}
                            on:keydown={(e) => {
                                if (e.key === 'Enter') {
                                    e.preventDefault(); // Prevent form submission or default button behavior
                                    newTag = tag; // Set the input value to the selected suggestion
                                    isInputFocused = true; // Keep the dropdown visible
                                }
                            }}
                        >
                            {tag}
                        </button>
                    </li>
                    {/each}
                </ul>            
            {/if}
        </div>
        {/if}
    </div>
    {/if}
  </main>  
</div>

  
<style>
.tag {
    display: inline-flex;
    align-items: center;
    background-color: #f3f4f6;
    color: #1f2937;
    padding: 0.25rem 0.5rem;
    border-radius: 0.375rem;
    margin-left: 0.25rem !important;
    margin-right: 0.25rem !important;
}
.tag button {
    /* margin-left: 0.5rem; */
    background: none;
    border: none;
    color: #ef4444;
    cursor: pointer;
}
.tag button:hover {
    color: #b91c1c;
}
.bg-blue-500 {
    background-color: #3b82f6;
}

.bg-gray-200 {
    background-color: #e5e7eb;
}

.text-white {
    color: white;
}

/* Add dark mode styles */
:global(html.dark) {
    --bg-color: #1f2937; /* Dark background color */
    --text-color: #f3f4f6; /* Light text color */
    --border-color: #374151; /* Dark border color */
}

:global(html.dark body) {
    background-color: var(--bg-color) !important;
    color: var(--text-color) !important;
}

:global(html.dark .color-white-100) {
    color: #f3f4f6; /* Dark equivalent of gray-100 */
}

:global(html.dark .bg-gray-100) {
    background-color: #1f2937; /* Dark equivalent of gray-100 */
}

:global(html.dark .bg-gray-50) {
    background-color: #2d3748; /* Dark equivalent of gray-50 */
}

:global(html.dark .text-gray-900) {
    color: #f3f4f6; /* Light equivalent of gray-900 */
}

:global(html.dark .text-gray-800) {
    color: #e2e8f0; /* Light equivalent of gray-800 */
}

:global(html.dark .border) {
    border-color: var(--border-color);
}

:global(html.dark button) {
    background-color: #374151; /* Dark button background */
    color: var(--text-color);
}

:global(html.dark p, html.dark h2, html.dark h3, html.dark h4, html.dark h1, html.dark h5) {
    color: var(--text-color);
}

:global(html.dark button:hover) {
    background-color: #4b5563; /* Dark button hover */
}

:global(html.dark header) {
    background-color: #1f2937 !important; /* Equivalent to dark:bg-gray-800 */
    color: #f3f4f6 !important; /* Equivalent to dark:text-white */
}
:global(html.dark a, html.dark textarea, html.dark input, html.dark .tag, html.dark ul, html.dark ol, html.dark li, html.dark li::marker, html.dark strong) {
    background-color: #2d3748 !important; /* Equivalent to dark:bg-gray-800 */
    color: #f3f4f6 !important; /* Equivalent to dark:text-white */
}
strong {
    font-weight: 700;
}

:root {
  --sidebar-width: 25%; /* Default sidebar width */
}

aside {
  width: var(--sidebar-width);
  max-width: var(--sidebar-width);
}

.w-1 {
  width: 4px; /* Handle width */
  background-color: rgb(75 85 99); /* Gray-600 */
  cursor: col-resize;
  user-select: none;
  position: relative; /* Keep handle positioned correctly */
  z-index: 10; /* Ensure handle is above other elements */
}

.w-1:hover {
  background-color: rgb(55 65 81); /* Gray-700 */
}

ul.autocomplete {
    max-height: 200px; /* Limit dropdown height */
    overflow-y: auto; /* Add a scroll bar if too many items */
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); /* Subtle shadow */
    background-color: white; /* Ensure it stands out */
}
ul.autocomplete li {
    font-size: 0.875rem; /* Adjust text size */
    line-height: 1.25rem; /* Adjust line height */
}
</style>

