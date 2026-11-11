# Input

`````ts
export const BaseStyles = html`
  <style>
    :root {
      --ln-font-fallback:
        -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif,
        Apple Color Emoji, Segoe UI Emoji;
      --ln-font-body: "Geist", var(--ln-font-fallback);
    }
  </style>
`;

`````

# Output

## `{}`

`````ts
export const BaseStyles = html`
  <style>
    :root {
      --ln-font-fallback:
        -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif,
        Apple Color Emoji, Segoe UI Emoji;
      --ln-font-body: "Geist", var(--ln-font-fallback);
    }
  </style>
`;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````ts
export const BaseStyles = html`
  <style>
    :root {
      --ln-font-fallback:
        -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif,
        Apple Color Emoji, Segoe UI Emoji;
      --ln-font-body: "Geist", var(--ln-font-fallback);
    }
  </style>
`;

`````

## `{"endOfLine":"crlf"}`

`````ts
export const BaseStyles = html`<CRLF>
  <style><CRLF>
    :root {<CRLF>
      --ln-font-fallback:<CRLF>
        -apple-system, BlinkMacSystemFont, Segoe UI, Helvetica, Arial, sans-serif,<CRLF>
        Apple Color Emoji, Segoe UI Emoji;<CRLF>
      --ln-font-body: "Geist", var(--ln-font-fallback);<CRLF>
    }<CRLF>
  </style><CRLF>
`;<CRLF>

`````
