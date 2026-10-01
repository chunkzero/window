const SMALL_CAPS: Record<string, string> = {
    a: "ᴀ",
    b: "ʙ",
    c: "ᴄ",
    d: "ᴅ",
    e: "ᴇ",
    f: "ꜰ",
    g: "ɢ",
    h: "ʜ",
    i: "ɪ",
    j: "ᴊ",
    k: "ᴋ",
    l: "ʟ",
    m: "ᴍ",
    n: "ɴ",
    o: "ᴏ",
    p: "ᴘ",
    q: "ǫ",
    r: "ʀ",
    s: "ѕ",
    t: "ᴛ",
    u: "ᴜ",
    v: "ᴠ",
    w: "ᴡ",
    x: "x",
    y: "ʏ",
    z: "ᴢ",
};

/** Converts ASCII letters to small-caps glyphs; other characters are unchanged. */
export function smallCaps(text: string): string {
    return text.replace(/[A-Za-z]/g, (ch) => SMALL_CAPS[ch.toLowerCase()] ?? ch);
}

/** Like `smallCaps`, but leaves `<...>` MiniMessage tags untouched. */
export function smallCapsMinimessage(text: string): string {
    let out = "";
    let index = 0;
    while (index < text.length) {
        const tagStart = text.indexOf("<", index);
        if (tagStart === -1) {
            out += smallCaps(text.slice(index));
            break;
        }
        out += smallCaps(text.slice(index, tagStart));
        const tagEnd = text.indexOf(">", tagStart);
        if (tagEnd === -1) {
            out += text.slice(tagStart);
            break;
        }
        out += text.slice(tagStart, tagEnd + 1);
        index = tagEnd + 1;
    }
    return out;
}

export const text: {
    smallCaps: typeof smallCaps;
    smallCapsMinimessage: typeof smallCapsMinimessage;
} = { smallCaps, smallCapsMinimessage };
