declare global {
    interface String {
        format(args: Record<string, unknown> | unknown[]): string;
    }
}

String.prototype.format = function (args): string {
    return this.replace(/\{(\w+)\}/g, (_, k) => {
        const value = Array.isArray(args) ? args[parseInt(k)] : args[k];
        if (value === undefined || value === null) return `{${k}}`;
        if (typeof value === "string") return value;
        if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
            return value.toString();
        }
        return JSON.stringify(value);
    });
}

export {};