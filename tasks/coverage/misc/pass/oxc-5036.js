Parent: {
    Child1: {
        Child2: console.log("Child1");
        Child2: console.log("Child2");
    }
    Child1: {
        Child2: console.log("Child3");
        Child2: console.log("Child4");
    }
    Child2: {
        Child1: console.log("Child5");
        Child1: console.log("Child6");
    }
}

Parent: {
    Child1: {
        Child2: console.log("Child7");
        Child2: console.log("Child8");
    }
}

Parent: {
    Child1: {
        Child12: {
            Child3: {
                Child4: {
                    Child5: console.log("Child9");
                }
            }
        }
    }
    Child1: console.log("Child10");
    Child2: console.log("Child11");
    Child1: {
        Child2: {
            Child3: console.log("Child12");
        }
    }
    Child3: console.log("Child13");
}
