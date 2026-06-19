<script lang="ts">
    import "../app.css";
    import Titlebar from "$lib/components/Titlebar.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import Statusbar from "$lib/components/Statusbar.svelte";
    import Popup from "$lib/components/Popup.svelte";
    import { onMount } from "svelte";
    import { initLocalization } from "$lib/state/localization.svelte";
    import { checkSettings } from "$lib/utils/commands";
    import { goto } from "$app/navigation";
    import { error as logError } from "@tauri-apps/plugin-log";

    onMount(async () => {
        // This is a desktop app with no browser devtools, so route uncaught
        // frontend errors to the log (hd2mm.log + stdout) where they can be
        // seen. Without this they vanish silently.
        window.addEventListener("error", (e) => {
            logError(`uncaught error: ${e.message} (${e.filename}:${e.lineno}:${e.colno})`);
        });
        window.addEventListener("unhandledrejection", (e) => {
            const r = e.reason;
            logError(`unhandled rejection: ${r?.stack ?? r?.message ?? String(r)}`);
        });

        await initLocalization("en");

        if (!await checkSettings()) {
            goto("/settings");
        }
    });
</script>

<div
    class="flex flex-col h-dvh overflow-hidden bg-zinc-900 border-zinc-500 border-4"
>
    <Titlebar />
    <div class="relative flex-1 flex flex-row">
        <Sidebar />
        <main class="flex-1 p-2 overflow-hidden min-w-0">
            <slot />
        </main>
        <Popup />
    </div>
    <Statusbar />
</div>
