export function log(message, context = '', ...data) {
    console.log(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}

export function error(message, context = '', ...data) {
    console.error(`[${new Date().toISOString()}] [${context}] ${message}`, ...data);
}
