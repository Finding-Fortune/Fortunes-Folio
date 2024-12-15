<script>
    import '../app.css';
    import TitleBar from "../components/TitleBar.svelte"
</script>

<div class="min-h-screen max-h-screen flex flex-col overflow-hidden">
    <!-- Fixed Title Bar -->
    <TitleBar />

    <!-- Main Content -->
    <div style="max-width: 100vw;" class="mt-8 flex flex-grow flex-1 h-auto overflow-auto">
        <slot />
    </div>
</div>

<style>
</style>