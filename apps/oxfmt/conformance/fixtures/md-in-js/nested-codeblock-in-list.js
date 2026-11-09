md`
- item1

  \`\`\`js
  console.log("hello");
  \`\`\`

- item2
`

function f() {
    return md`
        - outer item

          \`\`\`js
          const x = 1;
          \`\`\`

        - another
          - nested list
            \`\`\`bash
            npm install
            \`\`\`
    `;
}
