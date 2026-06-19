import { defineConfig } from 'vitest/config';

// Unit tests for the pure TS logic. UI components, Svelte/state glue, IPC
// wrappers, and bootstrap are exercised end-to-end, not here.
export default defineConfig({
    test: {
        include: ['src/**/*.test.ts'],
        coverage: {
            provider: 'v8',
            reporter: ['text', 'lcov'],
            reportsDirectory: 'coverage',
            include: [
                'src/lib/utils/stringExtensions.ts',
                'src/lib/models/settings.ts',
            ],
        },
    },
});
