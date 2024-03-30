import { parseSync } from "@oxc-parser/wasm"

const sampleSourceCode = `
    function myFn(){
        console.log("Here")
    }
`
const { program } = parseSync(sampleSourceCode)
console.log(program)