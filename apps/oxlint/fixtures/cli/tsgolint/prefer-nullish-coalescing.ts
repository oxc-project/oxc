declare const nullableString: string | null;
const nullishResult = nullableString !== null && nullableString !== undefined ? nullableString : 'default';
