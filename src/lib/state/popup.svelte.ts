import { ErrorPopup, NotificationPopup, type Popup } from '$lib/types/popup';
import { error as logError } from '@tauri-apps/plugin-log';

let popups = $state<Popup<any>[]>([]);

export function usePopup() {
    return {
        get isShown(): boolean {
            return popups.length > 0;
        },

        get currentPopup(): Popup<any> {
            const last = popups.at(-1);
            if (!last) {
                throw new Error("currentPopup accessed with no popups shown");
            }
            return last;
        },

        async show<T = void>(popup: Popup<T>): Promise<T> {
            // Log every error shown to the user. This is a desktop app with no
            // browser devtools, so the log is where errors are reviewed; caught
            // errors that only surface as popups would otherwise never be logged.
            if (popup instanceof ErrorPopup) {
                logError(`${popup.message}: ${popup.errorMessage}`);
            } else if (popup instanceof NotificationPopup && popup.kind === 'error') {
                logError(popup.message);
            }
            popups.push(popup);
            try {
                return await popup.promise;
            } finally {
                popups = popups.filter((p) => p !== popup);
            }
        }
    }
}