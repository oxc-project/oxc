function myFunction() {
    for (const element in array) {
        if ('if cond') {
            'if block'
            break
        } else if ('else cond') {
            'else block'
            continue
        }
        'for loop after if-else so this is unreachable'
        i++
    }
    'after for loop'
}
