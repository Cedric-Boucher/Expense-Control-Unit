import { format } from 'date-fns';

export function formatTimestampLocal(isoString: string): string {
    const date = new Date(isoString); // converts to local time
    return format(date, "yyyy-MM-dd'T'HH:mm:ss");
}

export function formatTimestampLocalForDisplay(isoString: string): string {
    const date = new Date(isoString); // converts to local time
    return format(date, "yyyy-MM-dd HH:mm:ss");
}
