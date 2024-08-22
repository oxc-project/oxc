Parent: {
    Child: {
        Child1: console.log("Child1");
        Child1: console.log("Child2");
    }
    Child: {
        Child1: console.log("Child3");
        Child1: console.log("Child4");
    }
    Child: {
        Child1: console.log("Child5");
        Child1: console.log("Child6");
    }
}

Parent: {
    Child: {
        Child1: console.log("Child7");
        Child1: console.log("Child8");
    }
}

Parent: {
    Child: {
        Child1: {
            Child2: {
                Child3: console.log("Child9");
            }
        }
    }
    Child: console.log("Child10");
    Child: console.log("Child11");
    Child: {
        Child1: {
            Child2: {
                Child3: console.log("Child12");
            }
        }
    }
    Child: console.log("Child13");
}
