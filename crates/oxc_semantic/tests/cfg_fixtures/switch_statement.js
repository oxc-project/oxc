function myFunction() {
    switch (true) {
        case 1:
            'case 1'
            break;
        case 2:
            'case 2'
        case 3:
            'case 3'
            break;
        case 4: {
            'case 4'
            let i = 10;
            i++;
            return i;
        }
        case 5:
            'case 5'
            j++;
            foo()
        default:
            'default'
            return 10;
    }
    'function scope'
    return 5;
}
