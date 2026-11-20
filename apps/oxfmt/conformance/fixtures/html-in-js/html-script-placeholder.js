// An HTML `<script>` body reads the template's placeholders as the template does
const a = html`<script type="text/html"><div a=${v} class="x ${c}"></div></script>`;
const b = html`<script type="text/html"><${Tag} a=${v} /><p>x</p></script>`;
