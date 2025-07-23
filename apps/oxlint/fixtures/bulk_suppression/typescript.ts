let unusedTsVar: string = "test";
console.log("typescript file");

interface UnusedInterface {
    prop: string;
}

function tsFunction() {
    debugger;
    var redeclaredVar = 1;
    let redeclaredVar = 2; // This should cause an error
}