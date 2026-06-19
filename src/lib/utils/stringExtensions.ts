// Fill `{name}` / `{0}` placeholders in a template from an object or array.
export function format(template: string, args: Record<string, unknown> | unknown[]): string {
    return template.replace(/\{(\w+)\}/g, (_, k) => {
        const value = Array.isArray(args) ? args[Number.parseInt(k)] : args[k];
        if (value === undefined || value === null) return `{${k}}`;
        if (typeof value === "string") return value;
        if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
            return value.toString();
        }
        return JSON.stringify(value);
    });
}
