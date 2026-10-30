const tag = () => {};

tag`\1`;
tag`\2`;
tag`\3`;
tag`\4`;
tag`\5`;
tag`\6`;
tag`\7`;
tag`\8`;
tag`\9`;
tag`\00`;
tag`\01`;
tag`\02`;
tag`\03`;
tag`\04`;
tag`\05`;
tag`\06`;
tag`\07`;
tag`\08`;
tag`\09`;

tag`\x`;
tag`\x0`;
tag`\u`;
tag`\u000`;
tag`\u{}`;
tag`\u{110000}`;

new RegExp(`\0`);
new RegExp(`\x30`);
new RegExp(`\u0030`);
new RegExp(`\u{30}`);
new RegExp(`\z`);
new RegExp(`\$`);
new RegExp(`\``);
new RegExp(`\\`);
